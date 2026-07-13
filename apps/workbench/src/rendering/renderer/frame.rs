use anyhow::{Context, Result};
use windows::Win32::Graphics::Direct3D12::{
    D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_PRESENT,
    D3D12_RESOURCE_STATE_RENDER_TARGET, ID3D12CommandList,
};
use windows::Win32::Graphics::Dxgi::DXGI_PRESENT;
use windows::core::Interface;

use crate::scene::SceneState;

use super::super::composition::CompositionOrder;
use super::super::device::transition;
use super::super::meshlet_scene::{MeshletFrame, SkeletalFrame};
use super::super::terrain::TerrainFrame;
use super::{CapturedFrame, RenderOutcome, Renderer};

impl Renderer {
    pub unsafe fn render(
        &mut self,
        color: [f32; 4],
        capture: bool,
        capture_object_ids: bool,
        probe_load: bool,
        scene: &SceneState,
    ) -> Result<RenderOutcome> {
        debug_assert!(!capture_object_ids || capture);
        debug_assert!(!probe_load || self.load_config().is_some());
        unsafe { self.drive_composition_traversal(scene.camera())? };
        unsafe { self.poll_cooked_completion()? };
        let terrain_outcome = unsafe { self.poll_terrain_completion()? };
        let stream_resident = self.resident_renderer.has_pending_stream();
        let index = unsafe { self.swap_chain.GetCurrentBackBufferIndex() } as usize;
        unsafe { self.wait_for_buffer(index)? };
        unsafe { self.allocators[index].Reset() }.context("command allocator reset failed")?;
        unsafe { self.command_list.Reset(&self.allocators[index], None) }
            .context("command list reset failed")?;
        if self.composition.has_pending() {
            unsafe {
                self.poll_composition_publication(&self.command_list.clone(), terrain_outcome)?
            };
        } else {
            let publication = unsafe {
                self.async_resident_renderer
                    .prepare_frame(&self.command_list)
            };
            if let Some(report) = publication.as_ref()
                && self.cooked_streamer.has_pending()
            {
                self.cooked_streamer.mark_published(report)?;
            }
            if let Some(report) =
                unsafe { self.terrain_renderer.prepare_frame(&self.command_list) }?
            {
                self.terrain_streamer.mark_published(&report)?;
            }
        }

        unsafe {
            transition(
                &self.command_list,
                &self.back_buffers[index],
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            );
        }
        let handle = self.rtv_handle(index);
        unsafe {
            self.command_list
                .OMSetRenderTargets(1, Some(&handle), true, None);
            self.command_list
                .ClearRenderTargetView(handle, &color, None);
            if self.composition_enabled() {
                let terrain_slots = self
                    .terrain_renderer
                    .active_assignments()
                    .context("composition has no terrain mapping")?
                    .iter()
                    .map(|value| value.slot)
                    .collect::<Vec<_>>();
                let snapshot = self
                    .async_resident_renderer
                    .snapshot()
                    .context("composition has no resident snapshot")?;
                let grounding_mode = self.composition_grounding_mode();
                let projection = self.terrain_renderer.projection()?;
                match self.composition_order() {
                    CompositionOrder::TerrainFirst => {
                        self.terrain_renderer.record(
                            &self.command_list,
                            TerrainFrame {
                                scene,
                                render_targets: [handle, self.scene_renderer.object_id_handle()],
                                depth_target: self.scene_renderer.depth_handle(),
                                probe: probe_load,
                                clear_depth_semantic: true,
                            },
                        )?;
                        self.skeletal_scene_renderer.record(
                            &self.command_list,
                            SkeletalFrame {
                                snapshot,
                                scene,
                                back_buffer: &self.back_buffers[index],
                                render_targets: [handle, self.scene_renderer.object_id_handle()],
                                depth_target: self.scene_renderer.depth_handle(),
                                background_color: color,
                                probe: probe_load,
                                terrain_slots: Some(&terrain_slots),
                                grounding_mode,
                                projection,
                                clear_depth_semantic: false,
                            },
                        )?;
                    }
                    CompositionOrder::ObjectFirst => {
                        self.skeletal_scene_renderer.record(
                            &self.command_list,
                            SkeletalFrame {
                                snapshot,
                                scene,
                                back_buffer: &self.back_buffers[index],
                                render_targets: [handle, self.scene_renderer.object_id_handle()],
                                depth_target: self.scene_renderer.depth_handle(),
                                background_color: color,
                                probe: probe_load,
                                terrain_slots: Some(&terrain_slots),
                                grounding_mode,
                                projection,
                                clear_depth_semantic: true,
                            },
                        )?;
                        self.terrain_renderer.record(
                            &self.command_list,
                            TerrainFrame {
                                scene,
                                render_targets: [handle, self.scene_renderer.object_id_handle()],
                                depth_target: self.scene_renderer.depth_handle(),
                                probe: probe_load,
                                clear_depth_semantic: false,
                            },
                        )?;
                    }
                }
            } else if self.terrain_renderer.is_enabled() {
                self.terrain_renderer.record(
                    &self.command_list,
                    TerrainFrame {
                        scene,
                        render_targets: [handle, self.scene_renderer.object_id_handle()],
                        depth_target: self.scene_renderer.depth_handle(),
                        probe: probe_load,
                        clear_depth_semantic: true,
                    },
                )?;
            } else if self.skeletal_scene_renderer.is_enabled() {
                let snapshot = self
                    .async_resident_renderer
                    .snapshot()
                    .context("skeletal scene has no published resident snapshot")?;
                let projection = snapshot.projection()?;
                self.skeletal_scene_renderer.record(
                    &self.command_list,
                    SkeletalFrame {
                        snapshot,
                        scene,
                        back_buffer: &self.back_buffers[index],
                        render_targets: [handle, self.scene_renderer.object_id_handle()],
                        depth_target: self.scene_renderer.depth_handle(),
                        background_color: color,
                        probe: probe_load,
                        terrain_slots: None,
                        grounding_mode: 0,
                        projection,
                        clear_depth_semantic: true,
                    },
                )?;
            } else if self.meshlet_scene_renderer.is_enabled() {
                let snapshot = self
                    .async_resident_renderer
                    .snapshot()
                    .context("meshlet scene has no published resident snapshot")?;
                self.meshlet_scene_renderer.record(
                    &self.command_list,
                    MeshletFrame {
                        snapshot,
                        region_heap: self.async_resident_renderer.descriptor_heap(),
                        scene,
                        render_targets: [handle, self.scene_renderer.object_id_handle()],
                        depth_target: self.scene_renderer.depth_handle(),
                        probe: probe_load,
                    },
                )?;
            } else if self.async_resident_renderer.config().is_some() {
                self.async_resident_renderer.record(
                    &self.command_list,
                    scene,
                    [handle, self.scene_renderer.object_id_handle()],
                    self.scene_renderer.depth_handle(),
                    probe_load,
                )?;
            } else if self.resident_renderer.config().is_some() {
                self.resident_renderer.record(
                    &self.command_list,
                    scene,
                    [handle, self.scene_renderer.object_id_handle()],
                    self.scene_renderer.depth_handle(),
                    probe_load,
                )?;
            } else if self.load_renderer.config().is_some() {
                self.load_renderer.record(
                    &self.command_list,
                    scene,
                    [handle, self.scene_renderer.object_id_handle()],
                    self.scene_renderer.depth_handle(),
                    probe_load,
                )?;
            } else {
                self.scene_renderer
                    .record(&self.command_list, scene, handle)?;
            }
            if capture_object_ids {
                transition(
                    &self.command_list,
                    self.scene_renderer.object_id_resource(),
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                self.object_id_capture
                    .record(&self.command_list, self.scene_renderer.object_id_resource());
                transition(
                    &self.command_list,
                    self.scene_renderer.object_id_resource(),
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                );
            }
            if capture {
                transition(
                    &self.command_list,
                    &self.back_buffers[index],
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                self.capture
                    .record(&self.command_list, &self.back_buffers[index]);
                transition(
                    &self.command_list,
                    &self.back_buffers[index],
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                    D3D12_RESOURCE_STATE_PRESENT,
                );
            } else {
                transition(
                    &self.command_list,
                    &self.back_buffers[index],
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_PRESENT,
                );
            }
            self.command_list.Close()
        }
        .context("command list close failed")?;

        let list: ID3D12CommandList = self.command_list.cast()?;
        unsafe {
            self.queue.ExecuteCommandLists(&[Some(list)]);
            self.swap_chain.Present(1, DXGI_PRESENT(0)).ok()
        }
        .context("Present failed")?;

        let signal = self.next_fence_value;
        self.next_fence_value += 1;
        unsafe { self.queue.Signal(&self.fence, signal) }.context("queue signal failed")?;
        self.fence_values[index] = signal;
        if capture || probe_load || stream_resident {
            unsafe { self.wait_for_value(signal)? };
            let captured_frame = if capture {
                let color = unsafe { self.capture.read() }?;
                let object_ids = if capture_object_ids {
                    Some(unsafe { self.object_id_capture.read() }?)
                } else {
                    None
                };
                Some(CapturedFrame { color, object_ids })
            } else {
                None
            };
            let meshlet_probe = if probe_load && self.meshlet_scene_renderer.is_enabled() {
                let snapshot = self
                    .async_resident_renderer
                    .snapshot()
                    .context("meshlet probe has no published resident snapshot")?;
                Some(unsafe { self.meshlet_scene_renderer.read_probe(snapshot, scene) }?)
            } else {
                None
            };
            let composition_probe = if probe_load && self.composition_enabled() {
                Some(unsafe { self.read_composition_probe(scene) }?)
            } else {
                None
            };
            let terrain_probe = if probe_load
                && self.terrain_renderer.is_enabled()
                && !self.composition_enabled()
            {
                Some(unsafe { self.terrain_renderer.read_probe(scene) }?)
            } else {
                None
            };
            let surface_probe = if probe_load
                && self.skeletal_scene_renderer.is_enabled()
                && self.skeletal_scene_renderer.surface_is_enabled()
            {
                let snapshot = self
                    .async_resident_renderer
                    .snapshot()
                    .context("surface probe has no published resident snapshot")?;
                Some(unsafe {
                    self.skeletal_scene_renderer
                        .read_surface_probe(snapshot, scene, color)
                }?)
            } else {
                None
            };
            let skeletal_probe = if probe_load
                && self.skeletal_scene_renderer.is_enabled()
                && !self.skeletal_scene_renderer.surface_is_enabled()
                && !self.composition_enabled()
            {
                let snapshot = self
                    .async_resident_renderer
                    .snapshot()
                    .context("skeletal probe has no published resident snapshot")?;
                Some(unsafe { self.skeletal_scene_renderer.read_probe(snapshot, scene) }?)
            } else {
                None
            };
            let load_probe = if probe_load
                && (self.terrain_renderer.is_enabled()
                    || self.meshlet_scene_renderer.is_enabled()
                    || self.skeletal_scene_renderer.is_enabled())
            {
                None
            } else if probe_load && self.async_resident_renderer.config().is_some() {
                Some(unsafe { self.async_resident_renderer.read_probe() }?)
            } else if probe_load && self.resident_renderer.config().is_some() {
                Some(unsafe { self.resident_renderer.read_probe() }?)
            } else if probe_load {
                Some(unsafe { self.load_renderer.read_probe() }?)
            } else {
                None
            };
            let resident_stream = if stream_resident {
                Some(self.resident_renderer.complete_stream()?)
            } else {
                None
            };
            return Ok(RenderOutcome {
                capture: captured_frame,
                load_probe,
                meshlet_probe,
                skeletal_probe,
                surface_probe,
                terrain_probe,
                composition_probe,
                resident_stream,
            });
        }
        Ok(RenderOutcome {
            capture: None,
            load_probe: None,
            meshlet_probe: None,
            skeletal_probe: None,
            surface_probe: None,
            terrain_probe: None,
            composition_probe: None,
            resident_stream: None,
        })
    }
}
