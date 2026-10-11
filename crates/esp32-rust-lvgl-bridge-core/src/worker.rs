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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::{ParamId, ParamVal};

    struct MockController {
        speed: i32,
        powered: bool,
        steps: u32,
    }

    impl MockController {
        fn new() -> Self {
            Self {
                speed: 0,
                powered: false,
                steps: 0,
            }
        }
    }

    impl Controller for MockController {
        fn process_command(&mut self, cmd: UiCommand) {
            match cmd.param_id {
                ParamId(1) => self.powered = unsafe { cmd.value.bool_val },
                ParamId(2) => self.speed = unsafe { cmd.value.i32_val },
                _ => {}
            }
        }

        fn step(&mut self) {
            self.steps += 1;
        }

        fn emit_telemetry<F>(&self, mut send_fn: F)
        where
            F: FnMut(&DeviceTelemetry),
        {
            send_fn(&DeviceTelemetry {
                param_id: ParamId(1),
                ack_seq_id: 1,
                value: ParamVal::from_bool(self.powered),
            });
            send_fn(&DeviceTelemetry {
                param_id: ParamId(2),
                ack_seq_id: 2,
                value: ParamVal::from_i32(self.speed),
            });
        }
    }

    #[test]
    fn test_controller_lifecycle() {
        let mut ctrl = MockController::new();
        assert!(!ctrl.powered);
        assert_eq!(ctrl.speed, 0);
        assert_eq!(ctrl.steps, 0);

        ctrl.process_command(UiCommand {
            param_id: ParamId(1),
            seq_id: 10,
            value: ParamVal::from_bool(true),
        });
        ctrl.process_command(UiCommand {
            param_id: ParamId(2),
            seq_id: 11,
            value: ParamVal::from_i32(75),
        });
        assert!(ctrl.powered);
        assert_eq!(ctrl.speed, 75);

        ctrl.step();
        assert_eq!(ctrl.steps, 1);

        let mut emitted = Vec::new();
        ctrl.emit_telemetry(|telem| {
            emitted.push((telem.param_id, telem.ack_seq_id));
        });
        assert_eq!(emitted.len(), 2);
        assert_eq!(emitted[0], (ParamId(1), 1));
        assert_eq!(emitted[1], (ParamId(2), 2));
    }
}
