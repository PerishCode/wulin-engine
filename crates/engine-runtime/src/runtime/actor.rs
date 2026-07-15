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
    ) -> Result<RuntimeActor> {
        presentation.validate()?;
        ensure!(self.actor.is_none(), "runtime actor slot is occupied");
        let generation = self
            .generation
            .checked_add(1)
            .context("runtime actor generation exhausted")?;
        let actor = RuntimeActor {
            handle: ActorHandle { generation },
            motion,
            presentation,
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

    pub fn replace_motion(
        &mut self,
        handle: ActorHandle,
        motion: TerrainBodyMotion,
    ) -> Result<RuntimeActor> {
        let input = self.read(handle)?;
        let output = RuntimeActor { motion, ..input };
        self.actor = Some(output);
        Ok(output)
    }
}

#[cfg(test)]
#[path = "../../tests/private/actor.rs"]
mod tests;
