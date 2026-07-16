use std::path::PathBuf;

use anyhow::Result;
use serde_json::Value;
use windows::Win32::Foundation::HWND;

use crate::rendering::{RenderFrame, RenderOutcome, Renderer};
use crate::scene::SceneState;
use crate::streaming::address::GlobalRegionConfig;
use crate::terrain_query::{TerrainBodyMotion, TerrainHeight, TerrainPosition};
use crate::timeline::{PresentationTimeline, SimulationSchedule};

mod actor;
mod motion_batch;
mod object_query;
mod simulation_actor;

use actor::ActorSlot;
pub use actor::{ActorHandle, RuntimeActor};
pub use object_query::{
    CANONICAL_OBJECT_NEAREST_CANDIDATE_CAPACITY, CANONICAL_OBJECTS_PER_REGION, CanonicalObject,
    CanonicalObjectIdentity, CanonicalObjectNearest, CanonicalObjectNearestQuery,
    CanonicalObjectPresentation, CanonicalObjectProximity, CanonicalObjectResolution,
    CanonicalObjectSnapshot, ObjectTargetFeedback, ObjectTargetFeedbackKind,
};
pub use region_format::PresentationRecord as ActorPresentation;
use simulation_actor::prepare_simulation_actor;
pub use simulation_actor::{
    ActorSimulationAdvance, ActorSimulationCommand, ActorSimulationOutcome,
    ActorSimulationRenderBlock, ActorStateTransition,
};

#[derive(Clone, Copy)]
pub struct FrameRequest {
    pub clear_color: [f32; 4],
    pub capture: bool,
    pub capture_object_ids: bool,
    pub probe: bool,
    pub object_target_feedback: Option<ObjectTargetFeedback>,
    pub object_suppression: Option<CanonicalObjectIdentity>,
}

pub struct Runtime {
    renderer: Renderer,
    scene: SceneState,
    presentation_timeline: PresentationTimeline,
    simulation_schedule: SimulationSchedule,
    actor: ActorSlot,
}

impl Runtime {
    /// Creates the native D3D12 runtime for one window.
    ///
    /// # Safety
    ///
    /// `hwnd` must identify a live window owned by the calling thread and must remain valid until
    /// this runtime is dropped. The extent must describe that window's renderable client area.
    pub unsafe fn new(hwnd: HWND, width: u32, height: u32) -> Result<Self> {
        Ok(Self {
            renderer: unsafe { Renderer::new(hwnd, width, height)? },
            scene: SceneState::new(),
            presentation_timeline: PresentationTimeline::new(),
            simulation_schedule: SimulationSchedule::new(),
            actor: ActorSlot::new(),
        })
    }

    /// Advances and presents one frame on the runtime's owning thread.
    ///
    /// # Safety
    ///
    /// The window supplied to [`Runtime::new`] must still be live, and this call must execute on
    /// the thread that created the runtime while no external code uses its native GPU objects.
    pub unsafe fn frame(&mut self, request: FrameRequest) -> Result<RenderOutcome> {
        let presentation_tick = self.presentation_timeline.tick();
        let actor = self.actor.current();
        let presentation_status = request
            .probe
            .then(|| self.presentation_timeline.status_json());
        let simulation_status = request
            .probe
            .then(|| self.simulation_schedule.status_json());
        let outcome = unsafe {
            self.renderer.render(RenderFrame {
                color: request.clear_color,
                capture: request.capture,
                capture_object_ids: request.capture_object_ids,
                probe: request.probe,
                presentation_tick,
                presentation_status: presentation_status.as_ref(),
                simulation_status: simulation_status.as_ref(),
                actor,
                object_target_feedback: request.object_target_feedback,
                object_suppression: request.object_suppression,
                scene: &mut self.scene,
            })
        }?;
        if self.renderer.composition_enabled() {
            self.presentation_timeline.commit_canonical_frame();
        }
        Ok(outcome)
    }

    /// Waits for all runtime-owned GPU and streaming work to become idle.
    ///
    /// # Safety
    ///
    /// This must execute on the runtime's owning thread while its native window and device remain
    /// valid.
    pub unsafe fn wait_idle(&mut self) -> Result<()> {
        unsafe { self.renderer.wait_idle() }
    }

    pub fn adapter_name(&self) -> &str {
        self.renderer.adapter_name()
    }

    pub fn debug_layer(&self) -> bool {
        self.renderer.debug_layer()
    }

    pub fn mesh_shader_tier(&self) -> u32 {
        self.renderer.mesh_shader_tier()
    }

    pub fn shader_model(&self) -> &str {
        self.renderer.shader_model()
    }

    pub fn barycentrics_supported(&self) -> bool {
        self.renderer.barycentrics_supported()
    }

    pub fn rasterizer_ordered_views_supported(&self) -> bool {
        self.renderer.rasterizer_ordered_views_supported()
    }

    pub fn visibility_format_supported(&self) -> bool {
        self.renderer.visibility_format_supported()
    }

    pub fn color_uav_format_supported(&self) -> bool {
        self.renderer.color_uav_format_supported()
    }

    /// Returns the current native device-removal reason, if any.
    ///
    /// # Safety
    ///
    /// The runtime's native device must not be used concurrently outside this facade.
    pub unsafe fn device_removed_reason(&self) -> Option<String> {
        unsafe { self.renderer.device_removed_reason() }
    }

    pub fn camera_json(&self) -> Value {
        self.scene.camera_json()
    }

    pub fn reset_camera(&mut self) -> Value {
        self.scene.reset_camera();
        self.scene.camera_json()
    }

    pub fn set_camera(
        &mut self,
        position: [f32; 3],
        target: [f32; 3],
        vertical_fov_degrees: f32,
    ) -> Result<Value> {
        self.scene
            .set_camera(position, target, vertical_fov_degrees)?;
        Ok(self.scene.camera_json())
    }

    pub fn set_actor_relative_camera(
        &mut self,
        handle: ActorHandle,
        position_offset: [f32; 3],
        target_offset: [f32; 3],
        vertical_fov_degrees: f32,
    ) -> Result<()> {
        let actor = self.actor.read(handle)?;
        let anchor = self.renderer.actor_scene_center(actor)?;
        self.scene.set_camera_from_anchor(
            anchor,
            position_offset,
            target_offset,
            vertical_fov_degrees,
        )?;
        Ok(())
    }

    pub fn spatial_json(&self) -> Value {
        self.scene.spatial_json()
    }

    pub fn open_terrain_pack(&mut self, path: PathBuf) -> Result<Value> {
        self.renderer.open_terrain_pack(path)
    }

    pub fn open_cooked_object_pack(&mut self, path: PathBuf) -> Result<Value> {
        self.renderer.open_cooked_object_pack(path)
    }

    pub fn composition_status(&self) -> Value {
        let mut status = self.renderer.composition_status();
        status["presentationClock"] = self.presentation_timeline.status_json();
        status["simulationSchedule"] = self.simulation_schedule.status_json();
        status
    }

    pub fn composition_enabled(&self) -> bool {
        self.renderer.composition_enabled()
    }

    pub fn query_terrain_height(&self, position: TerrainPosition) -> Result<TerrainHeight> {
        self.renderer.query_terrain_height(position)
    }

    pub fn resolve_canonical_object(
        &self,
        identity: CanonicalObjectIdentity,
    ) -> Result<CanonicalObjectResolution> {
        self.renderer.resolve_canonical_object(identity)
    }

    pub fn canonical_object_snapshot(&self) -> Result<CanonicalObjectSnapshot> {
        self.renderer.canonical_object_snapshot()
    }

    pub fn query_nearest_canonical_object(
        &self,
        origin: TerrainPosition,
        max_distance_q9: u32,
        excluded_identity: Option<CanonicalObjectIdentity>,
    ) -> Result<CanonicalObjectNearestQuery> {
        self.renderer
            .query_nearest_canonical_object(origin, max_distance_q9, excluded_identity)
    }

    pub fn spawn_actor(
        &mut self,
        motion: TerrainBodyMotion,
        presentation: ActorPresentation,
    ) -> Result<RuntimeActor> {
        self.actor
            .spawn(motion, presentation, self.presentation_timeline.tick())
    }

    pub fn read_actor(&self, handle: ActorHandle) -> Result<RuntimeActor> {
        self.actor.read(handle)
    }

    pub fn despawn_actor(&mut self, handle: ActorHandle) -> Result<RuntimeActor> {
        self.actor.despawn(handle)
    }

    pub fn advance_simulation_actor(
        &mut self,
        handle: ActorHandle,
        elapsed_nanoseconds: u64,
        command: ActorSimulationCommand,
    ) -> Result<ActorSimulationOutcome> {
        let input = self.actor.read(handle)?;
        let prepared = prepare_simulation_actor(
            self.simulation_schedule,
            input.motion,
            elapsed_nanoseconds,
            command,
            |position| self.query_terrain_height(position),
        )?;
        let presentation = if prepared.simulation.step_count == 0 {
            input.presentation
        } else {
            command.presentation
        };
        let animation_epoch_tick = if prepared.simulation.step_count == 0 {
            input.animation_epoch_tick
        } else {
            actor::transition_animation_epoch(
                input,
                presentation,
                self.presentation_timeline.tick(),
            )?
        };
        let candidate = RuntimeActor {
            motion: prepared.motion.output,
            presentation,
            animation_epoch_tick,
            ..input
        };
        if self.renderer.composition_enabled() {
            let admission = self.renderer.preflight_actor(candidate)?;
            if admission.pending_blocked() {
                return Ok(ActorSimulationOutcome::RenderBlocked(
                    ActorSimulationRenderBlock {
                        prepared_step_count: prepared.simulation.step_count,
                        terrain_query_count: prepared.motion.terrain_query_count,
                    },
                ));
            }
            admission.require()?;
        }
        let output = self.actor.replace_state(handle, candidate)?;
        self.simulation_schedule = prepared.schedule;
        Ok(ActorSimulationOutcome::Advanced(ActorSimulationAdvance {
            simulation: prepared.simulation,
            actor: ActorStateTransition {
                input,
                output,
                step_count: prepared.simulation.step_count,
                terrain_query_count: prepared.motion.terrain_query_count,
                last_step_grounded: prepared.motion.last_step_grounded,
            },
        }))
    }

    pub fn pause_presentation_time(&mut self) -> Value {
        self.presentation_timeline.pause();
        self.presentation_timeline.status_json()
    }

    pub fn resume_presentation_time(&mut self) -> Value {
        self.presentation_timeline.resume();
        self.presentation_timeline.status_json()
    }

    pub fn set_presentation_time(&mut self, tick: u32) -> Result<Value> {
        self.presentation_timeline.set(tick)?;
        Ok(self.presentation_timeline.status_json())
    }

    pub fn step_presentation_time(&mut self, ticks: u32) -> Result<Value> {
        self.presentation_timeline.step(ticks)?;
        Ok(self.presentation_timeline.status_json())
    }

    /// Schedules one canonical terrain/object pair publication.
    ///
    /// # Safety
    ///
    /// This must execute on the runtime's owning thread while its native device and source
    /// streamers are live.
    pub unsafe fn schedule_global_composition(
        &mut self,
        config: GlobalRegionConfig,
    ) -> Result<Value> {
        unsafe { self.renderer.schedule_global_composition(config) }
    }

    pub fn enable_composition_traversal(&mut self) -> Result<()> {
        self.renderer.enable_composition_traversal()
    }

    pub fn disable_composition_traversal(&mut self) {
        self.renderer.disable_composition_traversal();
    }

    pub fn enable_composition_prefetch(&mut self) -> Result<()> {
        self.renderer.enable_composition_prefetch()
    }

    pub fn disable_composition_prefetch(&mut self) -> Result<()> {
        self.renderer.disable_composition_prefetch()
    }

    pub fn arm_object_io_gate(&mut self) -> Result<u64> {
        self.renderer.arm_object_io_gate()
    }

    pub fn release_object_io_gate(&mut self) -> Result<u64> {
        self.renderer.release_object_io_gate()
    }

    pub fn arm_object_copy_gate(&mut self) -> Result<u64> {
        self.renderer.arm_async_copy_gate()
    }

    /// Releases the deliberately armed object-copy gate.
    ///
    /// # Safety
    ///
    /// The matching gate must have been armed on this runtime, and release must occur on the
    /// runtime's owning thread.
    pub unsafe fn release_object_copy_gate(&mut self) -> Result<u64> {
        unsafe { self.renderer.release_async_copy_gate() }
    }

    pub fn arm_terrain_io_gate(&mut self) -> Result<u64> {
        self.renderer.arm_terrain_io_gate()
    }

    pub fn release_terrain_io_gate(&mut self) -> Result<u64> {
        self.renderer.release_terrain_io_gate()
    }

    pub fn arm_terrain_copy_gate(&mut self) -> Result<u64> {
        self.renderer.arm_terrain_copy_gate()
    }

    /// Releases the deliberately armed terrain-copy gate.
    ///
    /// # Safety
    ///
    /// The matching gate must have been armed on this runtime, and release must occur on the
    /// runtime's owning thread.
    pub unsafe fn release_terrain_copy_gate(&mut self) -> Result<u64> {
        unsafe { self.renderer.release_terrain_copy_gate() }
    }
}
