use engine_runtime::ActorPresentation;

use crate::locomotion::Command;

const SURVEY_CLIP: u32 = 0;
const WALK_CLIP: u32 = 1;

pub(crate) struct Policy {
    committed_yaw_q16: u32,
}

impl Policy {
    pub(crate) const fn new() -> Self {
        Self {
            committed_yaw_q16: 0,
        }
    }

    pub(crate) const fn command(&self, command: Command) -> ActorPresentation {
        let moving = command.delta_x_q9 != 0 || command.delta_z_q9 != 0;
        let clip = if moving { WALK_CLIP } else { SURVEY_CLIP };
        let yaw_q16 = if moving {
            locomotion_yaw_q16(command)
        } else {
            self.committed_yaw_q16
        };
        animated(yaw_q16, clip)
    }

    pub(crate) fn observe_advance(&mut self, step_count: u32, output: ActorPresentation) {
        if step_count != 0 {
            self.committed_yaw_q16 = output.yaw_q16;
        }
    }
}

pub(crate) const fn initial() -> ActorPresentation {
    animated(0, SURVEY_CLIP)
}

const fn locomotion_yaw_q16(command: Command) -> u32 {
    match (command.delta_x_q9.signum(), command.delta_z_q9.signum()) {
        (1, 0) => 0,
        (1, 1) => 8_192,
        (0, 1) => 16_384,
        (-1, 1) => 24_576,
        (-1, 0) => 32_768,
        (-1, -1) => 40_960,
        (0, -1) => 49_152,
        (1, -1) => 57_344,
        _ => unreachable!(),
    }
}

const fn animated(yaw_q16: u32, clip: u32) -> ActorPresentation {
    ActorPresentation::animated(7, 63, yaw_q16, clip, 0, 0)
}
