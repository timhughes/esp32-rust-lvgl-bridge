//! # Headless Visual Snapshot Test for Minimal Starter Example
//!
//! Renders the minimal 3-widget UI schema in host memory and exports
//! a high-resolution 480×480 PNG snapshot to `artifacts/00_minimal_starter.png`.

use bridge_harness::HeadlessHarness;
use minimal_app::MINIMAL_SCHEMA;
use std::sync::Mutex;

static TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_minimal_visual_snapshot() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let mut harness = HeadlessHarness::new(480, 480);
    harness.build_schema(MINIMAL_SCHEMA);
    harness.step(50);

    // Save to both local artifacts and workspace root artifacts if accessible
    let dirs = ["artifacts", "../../../../artifacts"];
    let mut saved = false;

    for dir in dirs {
        if let Ok(_) = std::fs::create_dir_all(dir) {
            let file_path = format!("{}/00_minimal_starter.png", dir);
            if harness.save_png(&file_path).is_ok() {
                saved = true;
            }
        }
    }

    assert!(saved, "Failed to save minimal starter snapshot to artifacts directory");
}
