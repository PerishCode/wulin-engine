use anyhow::{Context, Result};
use engine_runtime::{ActorSimulationAdvance, ActorSimulationOutcome};
use reference_host::HostElapsedSample;

pub(crate) const fn admitted_elapsed(sample: HostElapsedSample) -> Option<u64> {
    match sample {
        HostElapsedSample::Ready {
            elapsed_nanoseconds,
        } => Some(elapsed_nanoseconds),
        HostElapsedSample::Reset
        | HostElapsedSample::Stalled { .. }
        | HostElapsedSample::Suspended => None,
    }
}

pub(crate) fn consume_actor_outcome(
    outcome: ActorSimulationOutcome,
    render_block_count: &mut u64,
) -> Result<Option<ActorSimulationAdvance>> {
    match outcome {
        ActorSimulationOutcome::Advanced(advance) => Ok(Some(advance)),
        ActorSimulationOutcome::RenderBlocked(_) => {
            *render_block_count = render_block_count
                .checked_add(1)
                .context("prototype actor render-block count overflowed")?;
            Ok(None)
        }
    }
}
