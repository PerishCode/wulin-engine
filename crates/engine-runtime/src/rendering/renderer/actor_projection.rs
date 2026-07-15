use anyhow::{Context, Result, ensure};
use serde::Serialize;

use crate::address::GlobalRegionConfig;
use crate::runtime::RuntimeActor;
use crate::terrain_query::{TERRAIN_BODY_HEIGHT_DENOMINATOR, TERRAIN_POSITION_DENOMINATOR};

use super::super::terrain::TerrainProjection;
use super::Renderer;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorRenderProjection {
    pub actor: RuntimeActor,
    pub global_config: GlobalRegionConfig,
    pub active_region_index: u32,
    pub semantic_region: u32,
    pub window_position_q9: [i32; 2],
    pub center_height_q16: i32,
    pub half_height_q16: i32,
    pub position_denominator: i32,
    pub height_denominator: u32,
}

impl Renderer {
    pub(crate) fn project_actor(&self, actor: RuntimeActor) -> Result<ActorRenderProjection> {
        let (global_config, config) = self.composition.actor_projection_config()?;
        project(global_config, config, actor)
    }
}

fn project(
    global_config: GlobalRegionConfig,
    config: crate::load::LoadConfig,
    actor: RuntimeActor,
) -> Result<ActorRenderProjection> {
    ensure!(
        global_config.local_config()? == config,
        "actor render projection local/global configs diverged"
    );
    let body = actor.motion.body();
    let position = body.position();
    let active_index = global_config
        .active_index(position.region())
        .context("runtime actor is outside the active render window")?;
    let projection = TerrainProjection::for_objects(config)?;
    let window_position_q9 =
        projection.position_q9(active_index, [position.local_x_q9(), position.local_z_q9()])?;
    Ok(ActorRenderProjection {
        actor,
        global_config,
        active_region_index: active_index as u32,
        semantic_region: projection.region_id(active_index)?,
        window_position_q9,
        center_height_q16: body.center_height_numerator(),
        half_height_q16: body.half_height_numerator(),
        position_denominator: TERRAIN_POSITION_DENOMINATOR,
        height_denominator: TERRAIN_BODY_HEIGHT_DENOMINATOR,
    })
}

#[cfg(test)]
#[path = "../../../tests/private/actor_projection.rs"]
mod tests;
