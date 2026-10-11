use std::path::{Path, PathBuf};

fn collect_c_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_c_files(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("c") {
                files.push(path);
            }
        }
    }
}

fn resolve_or_fetch_lvgl(manifest_dir: &Path) -> PathBuf {
    // 1. Explicit LVGL_DIR environment variable
    if let Ok(dir) = std::env::var("LVGL_DIR").map(PathBuf::from) {
        if dir.exists() && dir.join("src").exists() {
            return dir;
        }
    }

    // 2. Espressif Component Manager cache or parent project managed_components
    let home = std::env::var("HOME").unwrap_or_default();
    let candidates = [
        format!("{}/.cache/Espressif/ComponentManager/service_d92d8f1e/lvgl__lvgl_9.4.0_17e68bfd", home),
        format!("{}/../../managed_components/lvgl__lvgl", manifest_dir.display()),
        format!("{}/../../../managed_components/lvgl__lvgl", manifest_dir.display()),
        format!("{}/../../components/lvgl", manifest_dir.display()),
    ];

    for c in &candidates {
        let p = PathBuf::from(c);
        if p.exists() && p.join("src").exists() {
            return p;
        }
    }

    // 3. Fallback: Automatically clone/download LVGL v9.4.0 into OUT_DIR
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let lvgl_target = out_dir.join("lvgl-9.4.0");
    if lvgl_target.join("src").exists() {
        return lvgl_target;
    }

    println!("cargo:warning=LVGL v9 not found locally. Auto-fetching LVGL v9.4.0 into {}...", lvgl_target.display());
    let clone_ok = std::process::Command::new("git")
        .args(&["clone", "--depth", "1", "--branch", "v9.4.0", "https://github.com/lvgl/lvgl.git", lvgl_target.to_str().unwrap()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !clone_ok || !lvgl_target.join("src").exists() {
        let _ = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!(
                "mkdir -p '{}' && curl -sL https://codeload.github.com/lvgl/lvgl/tar.gz/refs/tags/v9.4.0 | tar -xz --strip-components=1 -C '{}'",
                lvgl_target.display(),
                lvgl_target.display()
            ))
            .status();
    }

    if lvgl_target.join("src").exists() {
        lvgl_target
    } else {
        panic!("Failed to locate or download LVGL v9 sources! Set LVGL_DIR environment variable.");
    }
}

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let c_src_dir = manifest_dir.join("c_src");
    let include_dir = c_src_dir.join("include");

    let lvgl_dir = resolve_or_fetch_lvgl(&manifest_dir);

    let mut c_files = Vec::new();
    collect_c_files(&lvgl_dir.join("src"), &mut c_files);

    // Bridge component sources and headers
    let bridge_dir = manifest_dir.join("../../components/esp_lvgl_bridge");
    let bridge_src = bridge_dir.join("src");
    let bridge_inc = bridge_dir.join("include");
    let bsp_inc = manifest_dir.join("../../components/bsp_guition_s3_4848s040/include");

    // Unified C bridge & BSP files from components + host harness test wrappers
    c_files.push(bridge_src.join("ui_binding.c"));
    c_files.push(bridge_src.join("p4_c6_link.c"));
    c_files.push(bridge_src.join("bridge_transport.c"));
    let bsp_src = manifest_dir.join("../../components/bsp_guition_s3_4848s040/src");
    c_files.push(bsp_src.join("bsp_4848s040.c"));
    c_files.push(c_src_dir.join("harness.c"));

    let mut build = cc::Build::new();
    build
        .opt_level(2)
        .define("LV_CONF_INCLUDE_SIMPLE", None)
        .include(&include_dir)
        .include(&bridge_inc)
        .include(&bsp_inc)
        .include(&lvgl_dir)
        .include(lvgl_dir.join("src"))
        .flag("-Wno-unused-function")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-deprecated-declarations");

    for file in c_files {
        build.file(file);
    }

    build.compile("lvgl_bridge_harness");

    println!("cargo:rerun-if-changed=../../components/esp_lvgl_bridge/src/ui_binding.c");
    println!("cargo:rerun-if-changed=../../components/esp_lvgl_bridge/src/p4_c6_link.c");
    println!("cargo:rerun-if-changed=../../components/esp_lvgl_bridge/include/bridge_protocol.h");
    println!("cargo:rerun-if-changed=../../components/esp_lvgl_bridge/include/ui_binding.h");
    println!("cargo:rerun-if-changed=c_src/harness.c");
    println!("cargo:rerun-if-changed=c_src/host_transport.c");
    println!("cargo:rerun-if-changed=c_src/include/lv_conf.h");
}
