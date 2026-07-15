use anyhow::{Context, Result, ensure};
use serde::Serialize;

use super::ActorPresentation;
use crate::terrain_query::TerrainBodyMotion;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorHandle {
    generation: u64,
}

impl ActorHandle {
    pub fn new(generation: u64) -> Result<Self> {
        ensure!(generation != 0, "actor generation must be nonzero");
        Ok(Self { generation })
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeActor {
    pub handle: ActorHandle,
    pub motion: TerrainBodyMotion,
    pub presentation: ActorPresentation,
    pub animation_epoch_tick: u32,
}

pub(crate) struct ActorSlot {
    generation: u64,
    actor: Option<RuntimeActor>,
}

impl ActorSlot {
    pub const fn new() -> Self {
        Self {
            generation: 0,
            actor: None,
        }
    }

    pub fn spawn(
        &mut self,
        motion: TerrainBodyMotion,
        presentation: ActorPresentation,
        animation_epoch_tick: u32,
    ) -> Result<RuntimeActor> {
        presentation.validate()?;
        validate_animation_epoch(animation_epoch_tick)?;
        ensure!(self.actor.is_none(), "runtime actor slot is occupied");
        let generation = self
            .generation
            .checked_add(1)
            .context("runtime actor generation exhausted")?;
        let actor = RuntimeActor {
            handle: ActorHandle { generation },
            motion,
            presentation,
            animation_epoch_tick,
        };
        self.generation = generation;
        self.actor = Some(actor);
        Ok(actor)
    }

    pub fn read(&self, handle: ActorHandle) -> Result<RuntimeActor> {
        let actor = self.actor.context("no runtime actor is live")?;
        ensure!(actor.handle == handle, "actor handle is stale");
        Ok(actor)
    }

    pub const fn current(&self) -> Option<RuntimeActor> {
        self.actor
    }

    pub fn despawn(&mut self, handle: ActorHandle) -> Result<RuntimeActor> {
        let actor = self.read(handle)?;
        self.actor = None;
        Ok(actor)
    }

    pub fn replace_state(
        &mut self,
        handle: ActorHandle,
        replacement: RuntimeActor,
    ) -> Result<RuntimeActor> {
        replacement.presentation.validate()?;
        validate_animation_epoch(replacement.animation_epoch_tick)?;
        let input = self.read(handle)?;
        ensure!(
            replacement.handle == input.handle,
            "replacement actor handle diverged"
        );
        self.actor = Some(replacement);
        Ok(replacement)
    }
}

pub(crate) fn transition_animation_epoch(
    input: RuntimeActor,
    output: ActorPresentation,
    current_tick: u32,
) -> Result<u32> {
    validate_animation_epoch(current_tick)?;
    Ok(if animation_stream_changed(input.presentation, output) {
        current_tick
    } else {
        input.animation_epoch_tick
    })
}

fn animation_stream_changed(input: ActorPresentation, output: ActorPresentation) -> bool {
    match (input.animation_clip(), output.animation_clip()) {
        (Some(input_clip), Some(output_clip)) => {
            input.archetype != output.archetype || input_clip != output_clip
        }
        (None, None) => false,
        _ => true,
    }
}

fn validate_animation_epoch(tick: u32) -> Result<()> {
    ensure!(
        tick < animation_catalog::PRESENTATION_CLOCK_FRAME_PERIOD,
        "actor animation epoch must be below the presentation clock period"
    );
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/private/actor.rs"]
mod tests;
