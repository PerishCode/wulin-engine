use anyhow::{Context, Result, ensure};
use windows::Win32::Graphics::Direct3D12::{
    D3D12_RESOURCE_STATE_COPY_SOURCE, D3D12_RESOURCE_STATE_PRESENT,
    D3D12_RESOURCE_STATE_RENDER_TARGET, ID3D12CommandList,
};
use windows::Win32::Graphics::Dxgi::DXGI_PRESENT;
use windows::core::Interface;

use super::super::device::transition;
use super::super::meshlet_scene::SkeletalFrame;
use super::super::terrain::TerrainFrame;
use super::{CapturedFrame, RenderFrame, RenderOutcome, Renderer};

impl Renderer {
    pub unsafe fn render(&mut self, frame: RenderFrame<'_>) -> Result<RenderOutcome> {
        let RenderFrame {
            color,
            capture,
            capture_object_ids,
            probe,
            presentation_tick,
            presentation_status,
            simulation_status,
            actor,
            object_target_feedback,
            object_suppression,
            scene,
        } = frame;
        debug_assert!(!capture_object_ids || capture);
        debug_assert!(!probe || self.composition_enabled());
        debug_assert!(!probe || presentation_status.is_some());
        super::object_target::validate(object_target_feedback)?;
        if let Some(identity) = object_suppression {
            ensure!(
                identity.authored_local_id < crate::runtime::CANONICAL_OBJECTS_PER_REGION,
                "object suppression authored local ID is outside the canonical region capacity"
            );
        }
        if self.composition_enabled()
            && let Some(actor) = actor
        {
            self.preflight_actor(actor)?.require()?;
        }
        unsafe { self.drive_composition_traversal(scene.camera())? };
        if self.composition_enabled()
            && let Some(actor) = actor
        {
            self.preflight_actor(actor)?.require()?;
        }
        unsafe { self.poll_cooked_object_completion()? };
        let terrain_outcome = unsafe { self.poll_terrain_completion()? };
        let index = unsafe { self.swap_chain.GetCurrentBackBufferIndex() } as usize;
        unsafe { self.wait_for_buffer(index)? };
        unsafe { self.allocators[index].Reset() }.context("command allocator reset failed")?;
        unsafe { self.command_list.Reset(&self.allocators[index], None) }
            .context("command list reset failed")?;
        if self.composition.has_pending() {
            unsafe {
                self.poll_composition_publication(&self.command_list.clone(), terrain_outcome)?
            };
        }
        if let Some(delta) = self.take_composition_camera_shift() {
            scene.translate_camera_regions(delta)?;
        }
        let actor_projection = if self.composition_enabled() {
            actor.map(|actor| self.project_actor(actor)).transpose()?
        } else {
            None
        };
        let mut rendered_object_target = None;
        let mut rendered_object_suppression = None;

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
                let projection = self.terrain_renderer.projection()?;
                rendered_object_target = super::object_target::project(
                    object_target_feedback,
                    snapshot.object_source_namespace,
                    snapshot.global_config,
                    projection,
                )?;
                rendered_object_suppression = super::object_target::project_suppression(
                    object_suppression,
                    snapshot.object_source_namespace,
                    snapshot.global_config,
                )?;
                self.terrain_renderer.record(
                    &self.command_list,
                    TerrainFrame {
                        scene,
                        render_targets: [handle, self.frame_targets.semantic_handle()],
                        depth_target: self.frame_targets.depth_handle(),
                        probe,
                        clear_depth_semantic: true,
                    },
                )?;
                self.skeletal_scene_renderer.record(
                    &self.command_list,
                    SkeletalFrame {
                        snapshot,
                        scene,
                        back_buffer: &self.back_buffers[index],
                        render_targets: [handle, self.frame_targets.semantic_handle()],
                        depth_target: self.frame_targets.depth_handle(),
                        background_color: color,
                        probe,
                        terrain_slots: Some(&terrain_slots),
                        grounding_mode: self.composition_grounding_mode(),
                        projection,
                        presentation_tick,
                        actor: actor_projection,
                        object_target: rendered_object_target,
                        object_suppression: rendered_object_suppression,
                        frame_slot: index as u32,
                    },
                )?;
            } else {
                self.frame_targets.clear_idle(&self.command_list);
            }
            if probe {
                self.async_resident_renderer
                    .record_active_payload_readback(&self.command_list)?;
            }
            if capture_object_ids {
                transition(
                    &self.command_list,
                    self.frame_targets.semantic_resource(),
                    D3D12_RESOURCE_STATE_RENDER_TARGET,
                    D3D12_RESOURCE_STATE_COPY_SOURCE,
                );
                self.object_id_capture
                    .record(&self.command_list, self.frame_targets.semantic_resource());
                transition(
                    &self.command_list,
                    self.frame_targets.semantic_resource(),
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
        let projected_object_target_feedback = rendered_object_target.map(|_| {
            object_target_feedback
                .expect("projected object target must preserve its validated frame feedback")
        });
        let projected_object_suppression = rendered_object_suppression.map(|_| {
            object_suppression.expect("projected object suppression must preserve its frame input")
        });
        let outcome = if capture || probe {
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
            let composition_probe = if probe {
                let presentation_status = presentation_status
                    .context("composition probe requires a presentation status snapshot")?;
                Some(unsafe {
                    self.read_composition_probe(
                        crate::rendering::composition::CompositionFrameProbeInput {
                            scene,
                            background_color: color,
                            presentation_tick,
                            presentation_status,
                            simulation_status: simulation_status.context(
                                "composition probe requires a simulation status snapshot",
                            )?,
                            actor: actor_projection,
                            object_target: rendered_object_target,
                            object_suppression: rendered_object_suppression,
                        },
                    )
                }?)
            } else {
                None
            };
            RenderOutcome {
                capture: captured_frame,
                composition_probe,
                object_target_feedback: projected_object_target_feedback,
                object_suppression: projected_object_suppression,
            }
        } else {
            RenderOutcome {
                capture: None,
                composition_probe: None,
                object_target_feedback: projected_object_target_feedback,
                object_suppression: projected_object_suppression,
            }
        };
        Ok(outcome)
    }
}
