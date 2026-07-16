use anyhow::{Context, Result};
use engine_runtime::{SIMULATION_MAX_STEPS_PER_ADVANCE, TerrainPosition};
use reference_host::bootstrap::PlayableRegionBounds;

use crate::locomotion::Command;

pub(crate) fn admit(
    position: TerrainPosition,
    bounds: PlayableRegionBounds,
    requested: Command,
) -> Result<Command> {
    let delta_x_q9 = admit_axis(position, bounds, requested.delta_x_q9, true)?;
    let delta_z_q9 = admit_axis(position, bounds, requested.delta_z_q9, false)?;
    Ok(Command {
        delta_x_q9,
        delta_z_q9,
        step_up_limit_q16: requested.step_up_limit_q16,
        running: requested.running && (delta_x_q9 != 0 || delta_z_q9 != 0),
    })
}

fn admit_axis(
    position: TerrainPosition,
    bounds: PlayableRegionBounds,
    delta_q9: i32,
    x_axis: bool,
) -> Result<i32> {
    let step_count = i32::try_from(SIMULATION_MAX_STEPS_PER_ADVANCE)
        .expect("maximum simulation steps must fit signed 32-bit");
    let maximum_delta_q9 = delta_q9
        .checked_mul(step_count)
        .context("prototype boundary maximum-batch displacement overflowed")?;
    let candidate = position
        .translated_q9(
            if x_axis { maximum_delta_q9 } else { 0 },
            if x_axis { 0 } else { maximum_delta_q9 },
        )
        .context("prototype boundary maximum-batch position failed")?;
    Ok(if bounds.contains(candidate.region()) {
        delta_q9
    } else {
        0
    })
}
