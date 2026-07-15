use anyhow::{Context, Result, ensure};
use serde::Serialize;

use crate::address::GlobalRegionConfig;
use crate::runtime::RuntimeActor;
use crate::terrain_query::TERRAIN_POSITION_REGION_SIDE_Q9;
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

    pub(crate) fn preflight_actor(&self, actor: RuntimeActor) -> Result<()> {
        self.project_actor(actor)?;
        if let Some((global_config, config)) = self.composition.pending_actor_projection_config() {
            project(global_config, config, actor)
                .context("runtime actor is outside the pending render window")?;
        }
        Ok(())
    }

    pub(crate) fn actor_scene_center(&self, actor: RuntimeActor) -> Result<[f32; 3]> {
        scene_center(self.project_actor(actor)?)
    }
}

fn scene_center(projection: ActorRenderProjection) -> Result<[f32; 3]> {
    let terrain = TerrainProjection::for_objects(projection.global_config.local_config()?)?;
    let alias = terrain.alias_offset_regions();
    let scene_position_q9 = [
        alias[0]
            .checked_mul(TERRAIN_POSITION_REGION_SIDE_Q9)
            .and_then(|value| value.checked_add(projection.window_position_q9[0]))
            .context("actor scene-center X overflowed")?,
        alias[1]
            .checked_mul(TERRAIN_POSITION_REGION_SIDE_Q9)
            .and_then(|value| value.checked_add(projection.window_position_q9[1]))
            .context("actor scene-center Z overflowed")?,
    ];
    Ok([
        scene_position_q9[0] as f32 / TERRAIN_POSITION_DENOMINATOR as f32,
        projection.center_height_q16 as f32 / TERRAIN_BODY_HEIGHT_DENOMINATOR as f32,
        scene_position_q9[1] as f32 / TERRAIN_POSITION_DENOMINATOR as f32,
    ])
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
