use anyhow::{Context, Result, ensure};
use serde::Serialize;

use crate::terrain_query::{TerrainBodyAdvance, TerrainBodyMotion};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainBodyHandle {
    generation: u64,
}

impl TerrainBodyHandle {
    pub fn new(generation: u64) -> Result<Self> {
        ensure!(generation != 0, "terrain body generation must be nonzero");
        Ok(Self { generation })
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetainedTerrainBody {
    pub handle: TerrainBodyHandle,
    pub motion: TerrainBodyMotion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetainedTerrainBodyAdvance {
    pub input: RetainedTerrainBody,
    pub advance: TerrainBodyAdvance,
    pub output: RetainedTerrainBody,
}

pub(crate) struct TerrainBodySlot {
    generation: u64,
    retained: Option<RetainedTerrainBody>,
}

impl TerrainBodySlot {
    pub const fn new() -> Self {
        Self {
            generation: 0,
            retained: None,
        }
    }

    pub fn spawn(&mut self, motion: TerrainBodyMotion) -> Result<RetainedTerrainBody> {
        ensure!(
            self.retained.is_none(),
            "retained terrain body slot is occupied"
        );
        let generation = self
            .generation
            .checked_add(1)
            .context("retained terrain body generation exhausted")?;
        let retained = RetainedTerrainBody {
            handle: TerrainBodyHandle { generation },
            motion,
        };
        self.generation = generation;
        self.retained = Some(retained);
        Ok(retained)
    }

    pub fn read(&self, handle: TerrainBodyHandle) -> Result<RetainedTerrainBody> {
        let retained = self.retained.context("no retained terrain body is live")?;
        ensure!(retained.handle == handle, "terrain body handle is stale");
        Ok(retained)
    }

    pub fn despawn(&mut self, handle: TerrainBodyHandle) -> Result<RetainedTerrainBody> {
        let retained = self.read(handle)?;
        self.retained = None;
        Ok(retained)
    }

    pub fn replace(
        &mut self,
        handle: TerrainBodyHandle,
        motion: TerrainBodyMotion,
    ) -> Result<RetainedTerrainBody> {
        let input = self.read(handle)?;
        let output = RetainedTerrainBody {
            handle: input.handle,
            motion,
        };
        self.retained = Some(output);
        Ok(output)
    }
}

#[cfg(test)]
#[path = "../../tests/private/retained_body.rs"]
mod tests;
