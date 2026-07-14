use anyhow::Result;
use serde_json::{Value, json};

use crate::rendering::async_resident::PublishedSnapshot;
use crate::scene::SceneState;

use super::probe::{self, ProbeInput, SkeletalProbe};
use super::renderer::SkeletalSceneRenderer;

impl SkeletalSceneRenderer {
    pub(in crate::rendering) unsafe fn read_composition_probe(
        &self,
        snapshot: &PublishedSnapshot,
        scene: &SceneState,
        ground_numerators: &[i32],
        ground_denominator: u32,
        instance_records: &[Vec<crate::resident::InstanceRecord>],
        local_ids: &[Vec<u32>],
    ) -> Result<SkeletalProbe> {
        unsafe {
            probe::read(ProbeInput {
                resources: &self.resources,
                mesh_catalog: &self.mesh_catalog,
                animation_catalog: &self.animation_catalog,
                mesh_catalog_sha256: &self.mesh_catalog_sha256,
                animation_catalog_sha256: &self.animation_catalog_sha256,
                settings: self.settings,
                settings_json: self.settings_json(),
                timestamp_frequency: self.timestamp_frequency,
                width: self.width,
                height: self.height,
                snapshot,
                scene,
                ground_numerators,
                ground_denominator,
                instance_records,
                local_ids,
            })
        }
    }

    pub(in crate::rendering) unsafe fn read_ground_numerators(
        &self,
        count: usize,
    ) -> Result<Vec<i32>> {
        unsafe { crate::rendering::resident::read_values(&self.resources.ground_readback, count) }
    }

    fn settings_json(&self) -> Value {
        json!({
            "animatedPercent": self.settings.animated_percent,
            "boneCount": self.settings.bone_count,
            "phaseCount": self.settings.phase_count,
            "timeTick": self.settings.time_tick,
            "uniquePoses": self.settings.unique_poses,
        })
    }
}
