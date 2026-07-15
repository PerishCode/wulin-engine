use engine_runtime::ActorPresentation;

use crate::locomotion::Command;

const SURVEY_CLIP: u32 = 0;
const WALK_CLIP: u32 = 1;

pub(crate) const fn initial() -> ActorPresentation {
    animated(SURVEY_CLIP)
}

pub(crate) const fn for_locomotion(command: Command) -> ActorPresentation {
    let clip = if command.delta_x_q9 == 0 && command.delta_z_q9 == 0 {
        SURVEY_CLIP
    } else {
        WALK_CLIP
    };
    animated(clip)
}

const fn animated(clip: u32) -> ActorPresentation {
    ActorPresentation::animated(7, 63, 0, clip, 0, 0)
}
