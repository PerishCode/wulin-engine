use animation_catalog::{BONE_COUNT, CLIP_COUNT};
use anyhow::Result;
use serde_json::{Value, json};

use crate::rendering::async_resident::PublishedSnapshot;
use crate::scene::SceneState;

use super::probe::{self, ProbeInput, SkeletalProbe};
use super::renderer::{SKELETAL_REVISION, SkeletalSceneRenderer};
use super::resources::{MAX_SHARED_POSES, MAX_SKELETAL_VISIBLE, PALETTE_BYTES};

impl SkeletalSceneRenderer {
    pub fn status_json(&self) -> Value {
        json!({
            "revision": SKELETAL_REVISION,
            "enabled": self.enabled,
            "settings": self.settings_json(),
            "catalog": {
                "meshletSha256": self.mesh_catalog_sha256,
                "animationSha256": self.animation_catalog_sha256,
                "boneCount": BONE_COUNT,
                "clipCount": CLIP_COUNT,
                "sampleCountPerClip": animation_catalog::SAMPLE_COUNT,
                "skinBindingCount": self.animation_catalog.skin_bindings.len(),
                "meshletGpuBytes": self.mesh_buffers.total_bytes,
                "animationGpuBytes": self.animation_buffers.total_bytes,
            },
            "resources": {
                "visibleCapacity": MAX_SKELETAL_VISIBLE,
                "sharedPoseCapacity": MAX_SHARED_POSES,
                "uniquePoseCapacity": MAX_SKELETAL_VISIBLE,
                "paletteBytes": PALETTE_BYTES,
                "executionBytes": self.resources.execution_bytes,
            },
            "submission": {
                "resetDispatchCount": 1,
                "cullDispatchCount": 1,
                "poseCompactDispatchCount": 1,
                "indirectPoseDispatchCount": 1,
                "indirectMeshDispatchCount": 1,
            }
        })
    }

    pub unsafe fn read_probe(
        &self,
        snapshot: &PublishedSnapshot,
        scene: &SceneState,
    ) -> Result<SkeletalProbe> {
        unsafe { self.read_probe_with_ground(snapshot, scene, None) }
    }

    pub(in crate::rendering) unsafe fn read_grounded_probe(
        &self,
        snapshot: &PublishedSnapshot,
        scene: &SceneState,
        ground_numerators: &[i32],
    ) -> Result<SkeletalProbe> {
        unsafe { self.read_probe_with_ground(snapshot, scene, Some(ground_numerators)) }
    }

    unsafe fn read_probe_with_ground(
        &self,
        snapshot: &PublishedSnapshot,
        scene: &SceneState,
        ground_numerators: Option<&[i32]>,
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
            "forcedLod": self.settings.forced_lod,
        })
    }
}
