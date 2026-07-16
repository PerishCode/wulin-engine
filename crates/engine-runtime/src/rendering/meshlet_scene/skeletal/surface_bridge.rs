use animation_catalog::Catalog as AnimationCatalog;
use anyhow::Result;
use meshlet_catalog::Catalog as MeshletCatalog;
use windows::Win32::Graphics::Direct3D12::{ID3D12CommandQueue, ID3D12Device};

use crate::scene::SceneState;

use super::renderer::{SkeletalSceneRenderer, SkeletalSettings};
use super::resources::ExecutionResources;
use super::surface::{SurfaceProbe, SurfaceProbeContext, SurfaceRenderer, SurfaceRendererInput};

pub(in crate::rendering) struct CompositionSurfaceInput<'a> {
    pub scene: &'a SceneState,
    pub presentation_tick: u32,
    pub background_color: [f32; 4],
    pub instance_records: &'a [Vec<crate::resident::InstanceRecord>],
    pub local_ids: &'a [Vec<u32>],
    pub presentations: &'a [Vec<crate::resident::PresentationRecord>],
    pub projection: crate::rendering::terrain::TerrainProjection,
    pub ground_numerators: &'a [i32],
    pub ground_denominator: u32,
    pub actor: Option<crate::rendering::ActorRenderProjection>,
    pub object_target: Option<crate::rendering::ProjectedObjectTarget>,
    pub object_suppression: Option<crate::rendering::ProjectedObjectSuppression>,
}

pub(super) unsafe fn create_surface(
    device: &ID3D12Device,
    queue: &ID3D12CommandQueue,
    resources: &ExecutionResources,
    mesh: &MeshletCatalog,
    animation: &AnimationCatalog,
    extent: [u32; 2],
) -> Result<SurfaceRenderer> {
    unsafe {
        SurfaceRenderer::new(
            device,
            SurfaceRendererInput {
                queue,
                source_heap: &resources.heap,
                source_visible: &resources.visible,
                source_counters: &resources.counters,
                mesh,
                animation,
                extent,
            },
        )
    }
}

impl SkeletalSceneRenderer {
    pub(in crate::rendering) fn enable_canonical_surface(&mut self) {
        self.surface.activate_canonical();
    }

    pub(in crate::rendering) unsafe fn read_composition_surface_probe(
        &self,
        skeletal: super::probe::SkeletalProbe,
        input: CompositionSurfaceInput<'_>,
    ) -> Result<SurfaceProbe> {
        unsafe {
            self.surface.read_probe(SurfaceProbeContext {
                skeletal,
                animation_catalog: &self.animation_catalog,
                mesh_catalog: &self.mesh_catalog,
                scene: input.scene,
                skeletal_settings: SkeletalSettings::for_tick(input.presentation_tick),
                instance_records: input.instance_records,
                local_ids: input.local_ids,
                presentations: input.presentations,
                projection: input.projection,
                ground_numerators: input.ground_numerators,
                ground_denominator: input.ground_denominator,
                background_color: input.background_color,
                timestamp_readback: &self.resources.timestamp_readback,
                timestamp_frequency: self.timestamp_frequency,
                actor: input.actor,
                object_target: input.object_target,
                object_suppression: input.object_suppression,
            })
        }
    }
}
