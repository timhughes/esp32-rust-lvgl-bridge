use crate::abi::{DeviceTelemetry, UiCommand};

pub trait Controller {
    fn process_command(&mut self, cmd: UiCommand);
    fn step(&mut self);
    fn emit_telemetry<F>(&self, send_fn: F)
    where
        F: FnMut(&DeviceTelemetry);
}

pub trait WorkerRunner {
    fn init(&mut self);
    fn run_step(&mut self);
}
