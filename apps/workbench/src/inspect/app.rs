use std::sync::mpsc::Receiver;

use engine_runtime::{GlobalRegionConfig, RegionCoord, Runtime, TerrainBody, TerrainPosition};
use reference_host::{
    bootstrap::{PackKind, validate_pack_path},
    window,
};
use serde_json::json;
use windows::Win32::Foundation::HWND;

use crate::{
    PendingCapture, PendingOperations, WINDOW_HEIGHT, WINDOW_WIDTH, WorkbenchState, perception,
};

use super::protocol::{ControlKind, ControlResult, ProtocolError};
use super::server::ControlCommand;

mod retained_body;

pub(crate) fn handle_commands(
    hwnd: HWND,
    runtime: &mut Runtime,
    state: &mut WorkbenchState,
    commands: &Receiver<ControlCommand>,
    pending: &mut PendingOperations,
) {
    while let Ok(command) = commands.try_recv() {
        let ControlCommand { kind, response } = command;
        let result = match kind {
            ControlKind::Status => super::status::workbench(hwnd, runtime, state),
            ControlKind::SetClearColor(color) => {
                state.clear_color = color;
                Ok(json!({"clearColor": color}))
            }
            ControlKind::Pause => {
                state.paused = true;
                Ok(json!({"paused": true}))
            }
            ControlKind::Resume => {
                state.paused = false;
                Ok(json!({"paused": false}))
            }
            ControlKind::InputStatus => Ok(state.input.status_json()),
            ControlKind::InputRecordStart => state
                .input
                .start_recording()
                .map_err(|error| protocol_error("input_record_failed", error)),
            ControlKind::InputRecordStop => state
                .input
                .stop_recording()
                .map_err(|error| protocol_error("input_record_failed", error)),
            ControlKind::InputReplay => state
                .input
                .replay()
                .map_err(|error| protocol_error("input_replay_failed", error)),
            ControlKind::SimulationStatus => Ok(runtime.simulation_status()),
            ControlKind::SimulationProbe => runtime
                .simulation_schedule_probe()
                .map_err(|error| protocol_error("simulation_probe_failed", error)),
            ControlKind::SimulationAdvance {
                elapsed_nanoseconds,
            } => runtime
                .advance_simulation(elapsed_nanoseconds)
                .map(|advance| {
                    json!({
                        "revision": "deterministic-fixed-simulation-schedule-v1",
                        "advance": advance,
                        "perAdvanceAllocationBytes": 0,
                        "sourceReadCount": 0,
                        "gpuCopyCount": 0,
                        "gpuReadbackCount": 0,
                        "fenceWaitCount": 0,
                        "synchronizationCount": 0,
                    })
                })
                .map_err(|error| protocol_error("simulation_advance_failed", error)),
            ControlKind::InputPost { messages } => window::post_input(hwnd, &messages)
                .map(|()| json!({"postedMessageCount": messages.len()}))
                .map_err(|error| protocol_error("native_input_failed", error)),
            ControlKind::CameraStatus => Ok(runtime.camera_json()),
            ControlKind::CameraReset => Ok(runtime.reset_camera()),
            ControlKind::CameraSetPose {
                position,
                target,
                vertical_fov_degrees,
            } => runtime
                .set_camera(position, target, vertical_fov_degrees)
                .map_err(|error| protocol_error("invalid_camera", error)),
            ControlKind::TerrainSourceOpen { path } => validate_pack_path(&path, PackKind::Terrain)
                .and_then(|path| runtime.open_terrain_pack(path))
                .map_err(|error| protocol_error("pack_open_failed", error)),
            ControlKind::ObjectSourceOpen { path } => validate_pack_path(&path, PackKind::Objects)
                .and_then(|path| runtime.open_cooked_object_pack(path))
                .map_err(|error| protocol_error("pack_open_failed", error)),
            ControlKind::CanonicalStatus => Ok(runtime.composition_status()),
            ControlKind::CanonicalTimeStatus => Ok(runtime.presentation_time_status()),
            ControlKind::CanonicalTimePause => Ok(runtime.pause_presentation_time()),
            ControlKind::CanonicalTimeResume => Ok(runtime.resume_presentation_time()),
            ControlKind::CanonicalTimeSet { tick } => runtime
                .set_presentation_time(tick)
                .map_err(|error| protocol_error("invalid_presentation_time", error)),
            ControlKind::CanonicalTimeStep { ticks } => runtime
                .step_presentation_time(ticks)
                .map_err(|error| protocol_error("invalid_presentation_time", error)),
            ControlKind::CanonicalSchedule {
                origin_x,
                origin_z,
                center_x,
                center_z,
                active_radius,
            } => GlobalRegionConfig::new(origin_x, origin_z, center_x, center_z, active_radius)
                .map_err(|error| protocol_error("invalid_global_config", error))
                .and_then(|config| {
                    unsafe { runtime.schedule_global_composition(config) }.map_err(stream_error)
                }),
            ControlKind::CanonicalTraversalEnable => runtime
                .enable_composition_traversal()
                .map(|()| runtime.composition_status())
                .map_err(stream_error),
            ControlKind::CanonicalTraversalDisable => {
                runtime.disable_composition_traversal();
                Ok(runtime.composition_status())
            }
            ControlKind::CanonicalPrefetchEnable => runtime
                .enable_composition_prefetch()
                .map(|()| runtime.composition_status())
                .map_err(stream_error),
            ControlKind::CanonicalPrefetchDisable => runtime
                .disable_composition_prefetch()
                .map(|()| runtime.composition_status())
                .map_err(stream_error),
            ControlKind::CanonicalProbe => {
                if !runtime.composition_enabled() {
                    Err(ProtocolError {
                        code: "canonical_unavailable",
                        message: "canonical composition must be published before probing".into(),
                    })
                } else if pending.is_idle() {
                    pending.probe = Some(response);
                    continue;
                } else {
                    Err(ProtocolError {
                        code: "capture_busy",
                        message: "a capture or probe request is already pending".into(),
                    })
                }
            }
            ControlKind::CanonicalTerrainHeight {
                region_x,
                region_z,
                local_x_q9,
                local_z_q9,
            } => TerrainPosition::new(RegionCoord::new(region_x, region_z), local_x_q9, local_z_q9)
                .and_then(|position| {
                    runtime.query_terrain_height(position).map(|height| {
                        json!({
                            "revision": "exact-canonical-terrain-query-v1",
                            "position": position,
                            "height": height,
                            "perQueryAllocationBytes": 0,
                            "sourceReadCount": 0,
                            "gpuCopyCount": 0,
                            "gpuReadbackCount": 0,
                            "fenceWaitCount": 0,
                            "synchronizationCount": 0,
                        })
                    })
                })
                .map_err(|error| protocol_error("terrain_query_failed", error)),
            ControlKind::CanonicalTerrainContact {
                region_x,
                region_z,
                local_x_q9,
                local_z_q9,
                center_height_numerator,
                half_height_numerator,
            } => TerrainPosition::new(RegionCoord::new(region_x, region_z), local_x_q9, local_z_q9)
                .and_then(|position| {
                    TerrainBody::new(position, center_height_numerator, half_height_numerator)
                })
                .and_then(|body| {
                    runtime.resolve_terrain_contact(body).map(|contact| {
                        json!({
                            "revision": "exact-terrain-body-contact-v1",
                            "inputBody": body,
                            "contact": contact,
                            "perResolutionAllocationBytes": 0,
                            "sourceReadCount": 0,
                            "gpuCopyCount": 0,
                            "gpuReadbackCount": 0,
                            "fenceWaitCount": 0,
                            "synchronizationCount": 0,
                        })
                    })
                })
                .map_err(|error| protocol_error("terrain_contact_failed", error)),
            ControlKind::CanonicalTerrainBodySpawn {
                region_x,
                region_z,
                local_x_q9,
                local_z_q9,
                center_height_numerator,
                half_height_numerator,
                step_velocity_q16,
            } => retained_body::spawn(
                runtime,
                retained_body::MotionPayload {
                    region_x,
                    region_z,
                    local_x_q9,
                    local_z_q9,
                    center_height_numerator,
                    half_height_numerator,
                    step_velocity_q16,
                },
            ),
            ControlKind::CanonicalTerrainBodyRead { generation } => {
                retained_body::read(runtime, generation)
            }
            ControlKind::CanonicalTerrainBodyDespawn { generation } => {
                retained_body::despawn(runtime, generation)
            }
            ControlKind::CanonicalTerrainBodyRetainedAdvance {
                generation,
                delta_x_q9,
                delta_z_q9,
                step_up_limit_q16,
                step_acceleration_q16,
            } => retained_body::advance(
                runtime,
                generation,
                delta_x_q9,
                delta_z_q9,
                step_up_limit_q16,
                step_acceleration_q16,
            ),
            ControlKind::CanonicalTerrainBodyRetainedBatch {
                generation,
                step_count,
                delta_x_q9,
                delta_z_q9,
                step_up_limit_q16,
                step_acceleration_q16,
            } => retained_body::batch(
                runtime,
                generation,
                step_count,
                delta_x_q9,
                delta_z_q9,
                step_up_limit_q16,
                step_acceleration_q16,
            ),
            ControlKind::SimulationTerrainBodyAdvance {
                generation,
                elapsed_nanoseconds,
                delta_x_q9,
                delta_z_q9,
                step_up_limit_q16,
                step_acceleration_q16,
            } => retained_body::simulation_advance(
                runtime,
                generation,
                elapsed_nanoseconds,
                delta_x_q9,
                delta_z_q9,
                step_up_limit_q16,
                step_acceleration_q16,
            ),
            ControlKind::ObjectIoGateArm => gate(runtime.arm_object_io_gate()),
            ControlKind::ObjectIoGateRelease => gate(runtime.release_object_io_gate()),
            ControlKind::ObjectCopyGateArm => gate(runtime.arm_object_copy_gate()),
            ControlKind::ObjectCopyGateRelease => {
                gate(unsafe { runtime.release_object_copy_gate() })
            }
            ControlKind::TerrainIoGateArm => gate(runtime.arm_terrain_io_gate()),
            ControlKind::TerrainIoGateRelease => gate(runtime.release_terrain_io_gate()),
            ControlKind::TerrainCopyGateArm => gate(runtime.arm_terrain_copy_gate()),
            ControlKind::TerrainCopyGateRelease => {
                gate(unsafe { runtime.release_terrain_copy_gate() })
            }
            ControlKind::Capture { id, collection } => {
                if pending.is_idle() {
                    pending.capture = Some(PendingCapture {
                        id,
                        collection,
                        perception: None,
                        response,
                    });
                    continue;
                }
                Err(ProtocolError {
                    code: "capture_busy",
                    message: "a capture or probe request is already pending".into(),
                })
            }
            ControlKind::PerceptionCapture {
                id,
                collection,
                region,
                samples,
            } => match perception::Request::new(region, samples, WINDOW_WIDTH, WINDOW_HEIGHT) {
                Ok(perception) if pending.is_idle() => {
                    pending.capture = Some(PendingCapture {
                        id,
                        collection,
                        perception: Some(perception),
                        response,
                    });
                    continue;
                }
                Ok(_) => Err(ProtocolError {
                    code: "capture_busy",
                    message: "a capture or probe request is already pending".into(),
                }),
                Err(error) => Err(protocol_error("invalid_region", error)),
            },
        };
        let _ = response.send(result);
    }
}

fn gate(result: anyhow::Result<u64>) -> ControlResult {
    result
        .map(|fence| json!({"gateFence": fence}))
        .map_err(|error| protocol_error("gate_failed", error))
}

fn stream_error(error: anyhow::Error) -> ProtocolError {
    let code = if error.to_string().contains("busy") {
        "stream_busy"
    } else {
        "stream_failed"
    };
    protocol_error(code, error)
}

fn protocol_error(code: &'static str, error: impl std::fmt::Display) -> ProtocolError {
    ProtocolError {
        code,
        message: error.to_string(),
    }
}
