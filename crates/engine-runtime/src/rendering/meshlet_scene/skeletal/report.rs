use anyhow::Result;
use serde_json::{Value, json};

use crate::rendering::async_resident::PublishedSnapshot;
use crate::scene::SceneState;

use super::probe::{self, ProbeInput, SkeletalProbe};
use super::renderer::{SkeletalSceneRenderer, SkeletalSettings};

pub(in crate::rendering) struct CompositionProbeInput<'a> {
    pub snapshot: &'a PublishedSnapshot,
    pub scene: &'a SceneState,
    pub presentation_tick: u32,
    pub ground_numerators: &'a [i32],
    pub ground_denominator: u32,
    pub instance_records: &'a [Vec<crate::resident::InstanceRecord>],
    pub local_ids: &'a [Vec<u32>],
    pub presentations: &'a [Vec<crate::resident::PresentationRecord>],
    pub actor: Option<crate::rendering::ActorRenderProjection>,
    pub object_suppression: Option<crate::rendering::ProjectedObjectSuppression>,
}

impl SkeletalSceneRenderer {
    pub(in crate::rendering) unsafe fn read_composition_probe(
        &self,
        input: CompositionProbeInput<'_>,
    ) -> Result<SkeletalProbe> {
        let settings = SkeletalSettings::for_tick(input.presentation_tick);
        unsafe {
            probe::read(ProbeInput {
                resources: &self.resources,
                mesh_catalog: &self.mesh_catalog,
                animation_catalog: &self.animation_catalog,
                mesh_catalog_sha256: &self.mesh_catalog_sha256,
                animation_catalog_sha256: &self.animation_catalog_sha256,
                settings,
                settings_json: settings_json(settings),
                timestamp_frequency: self.timestamp_frequency,
                width: self.width,
                height: self.height,
                snapshot: input.snapshot,
                scene: input.scene,
                ground_numerators: input.ground_numerators,
                ground_denominator: input.ground_denominator,
                instance_records: input.instance_records,
                local_ids: input.local_ids,
                presentations: input.presentations,
                actor: input.actor,
                object_suppression: input.object_suppression,
            })
        }
    }

    pub(in crate::rendering) unsafe fn read_ground_numerators(
        &self,
        count: usize,
    ) -> Result<Vec<i32>> {
        unsafe { crate::rendering::resident::read_values(&self.resources.ground_readback, count) }
    }
}

fn settings_json(settings: SkeletalSettings) -> Value {
    json!({
        "boneCount": settings.bone_count,
        "phaseCount": settings.phase_count,
        "timeTick": settings.time_tick,
    })
}
