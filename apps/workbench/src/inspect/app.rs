use std::sync::mpsc::{Receiver, SyncSender};

use serde_json::json;
use windows::Win32::Foundation::HWND;

use crate::rendering::Renderer;
use crate::{PendingCapture, PendingOperations, WorkbenchState, load, perception, scene, window};

use super::protocol::{ControlKind, ControlResult, ProtocolError};
use super::server::ControlCommand;
use super::status::load_status;

pub(crate) fn handle_commands(
    hwnd: HWND,
    renderer: &mut Renderer,
    state: &mut WorkbenchState,
    scene: &mut scene::SceneState,
    commands: &Receiver<ControlCommand>,
    pending: &mut PendingOperations,
) {
    while let Ok(command) = commands.try_recv() {
        let ControlCommand { kind, response } = command;
        let result = match kind {
            ControlKind::Status => super::status::workbench(hwnd, renderer, state, scene),
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
            ControlKind::CameraStatus => Ok(scene.camera_json()),
            ControlKind::CameraReset => {
                scene.reset_camera();
                Ok(scene.camera_json())
            }
            ControlKind::CameraSetPose {
                position,
                target,
                vertical_fov_degrees,
            } => scene
                .set_camera(
                    position,
                    target,
                    vertical_fov_degrees,
                    renderer.calibration_mode_active(),
                )
                .map(|_| scene.camera_json())
                .map_err(|error| ProtocolError {
                    code: "invalid_camera",
                    message: error.to_string(),
                }),
            ControlKind::SceneListObjects => Ok(scene.objects_json()),
            world @ (ControlKind::WorldStatus
            | ControlKind::WorldRelocate { .. }
            | ControlKind::WorldRebase { .. }
            | ControlKind::WorldReset
            | ControlKind::WorldProbe) => super::world_control::dispatch(renderer, scene, world),
            ControlKind::LoadStatus | ControlKind::ResidentStatus => Ok(load_status(renderer)),
            ControlKind::AsyncResidentStatus => Ok(renderer.async_resident_status()),
            ControlKind::AsyncCopyGateArm => renderer
                .arm_async_copy_gate()
                .map(|fence| json!({"gateFence": fence}))
                .map_err(|error| ProtocolError {
                    code: "gate_failed",
                    message: error.to_string(),
                }),
            ControlKind::AsyncCopyGateRelease => unsafe { renderer.release_async_copy_gate() }
                .map(|fence| json!({"gateFence": fence}))
                .map_err(|error| ProtocolError {
                    code: "gate_failed",
                    message: error.to_string(),
                }),
            ControlKind::CookedStatus => Ok(renderer.cooked_status()),
            ControlKind::CookedOpen { path } => super::pack_control::validate_cooked(&path)
                .and_then(|path| renderer.open_cooked_pack(path))
                .and_then(|metadata| serde_json::to_value(metadata).map_err(Into::into))
                .map_err(|error| ProtocolError {
                    code: "pack_open_failed",
                    message: error.to_string(),
                }),
            ControlKind::CookedIoGateArm => renderer
                .arm_cooked_io_gate()
                .map(|fence| json!({"gateFence": fence}))
                .map_err(|error| ProtocolError {
                    code: "gate_failed",
                    message: error.to_string(),
                }),
            ControlKind::CookedIoGateRelease => renderer
                .release_cooked_io_gate()
                .map(|fence| json!({"gateFence": fence}))
                .map_err(|error| ProtocolError {
                    code: "gate_failed",
                    message: error.to_string(),
                }),
            object @ (ControlKind::CookedObjectStatus
            | ControlKind::CookedObjectOpen { .. }
            | ControlKind::CookedObjectDisable
            | ControlKind::CookedObjectIoGateArm
            | ControlKind::CookedObjectIoGateRelease) => {
                super::pack_control::dispatch_object(renderer, object)
            }
            terrain @ (ControlKind::TerrainStatus
            | ControlKind::TerrainOpen { .. }
            | ControlKind::TerrainEnable
            | ControlKind::TerrainDisable
            | ControlKind::TerrainIoGateArm
            | ControlKind::TerrainIoGateRelease
            | ControlKind::TerrainCopyGateArm
            | ControlKind::TerrainCopyGateRelease
            | ControlKind::TerrainLodStatus
            | ControlKind::TerrainLodConfigure { .. }
            | ControlKind::TerrainLodEnable
            | ControlKind::TerrainLodDisable
            | ControlKind::TerrainGlobalSchedule { .. }
            | ControlKind::TerrainSchedule { .. }) => {
                super::terrain_control::dispatch(renderer, terrain)
            }
            composition @ (ControlKind::CompositionStatus
            | ControlKind::CompositionSchedule { .. }
            | ControlKind::CompositionGlobalSchedule(..)
            | ControlKind::CompositionEnable
            | ControlKind::CompositionDisable
            | ControlKind::CompositionTraversalEnable
            | ControlKind::CompositionTraversalDisable
            | ControlKind::CompositionPrefetchEnable
            | ControlKind::CompositionPrefetchDisable
            | ControlKind::CompositionOrder { .. }
            | ControlKind::CompositionFixture { .. }) => {
                super::composition_control::dispatch(renderer, composition)
            }
            ControlKind::MeshletStatus => Ok(renderer.meshlet_scene_status()),
            ControlKind::MeshletConfigure {
                archetype_mask,
                forced_lod,
            } => renderer
                .configure_meshlet_scene(archetype_mask, forced_lod)
                .map(|()| renderer.meshlet_scene_status())
                .map_err(|error| ProtocolError {
                    code: "invalid_meshlet_config",
                    message: error.to_string(),
                }),
            ControlKind::MeshletEnable => renderer
                .enable_meshlet_scene()
                .map(|()| renderer.meshlet_scene_status())
                .map_err(|error| ProtocolError {
                    code: "meshlet_unavailable",
                    message: error.to_string(),
                }),
            ControlKind::MeshletDisable => {
                renderer.disable_meshlet_scene();
                Ok(renderer.meshlet_scene_status())
            }
            ControlKind::SkeletalStatus => Ok(renderer.skeletal_scene_status()),
            ControlKind::SkeletalConfigure {
                animated_percent,
                bone_count,
                phase_count,
                time_tick,
                unique_poses,
                forced_lod,
            } => renderer
                .configure_skeletal_scene(
                    animated_percent,
                    bone_count,
                    phase_count,
                    time_tick,
                    unique_poses,
                    forced_lod,
                )
                .map(|()| renderer.skeletal_scene_status())
                .map_err(|error| ProtocolError {
                    code: "invalid_skeletal_config",
                    message: error.to_string(),
                }),
            ControlKind::SkeletalEnable => renderer
                .enable_skeletal_scene()
                .map(|()| renderer.skeletal_scene_status())
                .map_err(|error| ProtocolError {
                    code: "skeletal_unavailable",
                    message: error.to_string(),
                }),
            ControlKind::SkeletalDisable => {
                renderer.disable_skeletal_scene();
                Ok(renderer.skeletal_scene_status())
            }
            ControlKind::SurfaceStatus => super::surface_control::status(renderer),
            ControlKind::SurfaceConfigure {
                material_count,
                mip_level,
            } => super::surface_control::configure(renderer, material_count, mip_level),
            ControlKind::SurfaceEnable => super::surface_control::enable(renderer),
            ControlKind::SurfaceDisable => super::surface_control::disable(renderer),
            ControlKind::SurfaceOcclusionEnable => {
                super::surface_control::enable_occlusion(renderer)
            }
            ControlKind::SurfaceOcclusionDisable => {
                super::surface_control::disable_occlusion(renderer)
            }
            ControlKind::SurfaceOcclusionReset => super::surface_control::reset_occlusion(renderer),
            ControlKind::LoadDisable => renderer
                .disable_load()
                .map(|()| load_status(renderer))
                .map_err(|error| ProtocolError {
                    code: "stream_busy",
                    message: error.to_string(),
                }),
            ControlKind::LoadConfigure {
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            } => load::LoadConfig::new(
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            )
            .and_then(|config| {
                renderer
                    .configure_load(config)
                    .map(|()| load_status(renderer))
            })
            .map_err(|error| ProtocolError {
                code: "invalid_load_config",
                message: error.to_string(),
            }),
            ControlKind::LoadProbe => {
                if renderer.load_config().is_none() {
                    Err(ProtocolError {
                        code: "load_disabled",
                        message: "load mode must be configured before probing".into(),
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
            ControlKind::ResidentStream {
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            } => match begin_resident_stream(
                renderer,
                pending,
                &response,
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            ) {
                Some(result) => result,
                None => continue,
            },
            ControlKind::AsyncResidentSchedule {
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            } => begin_async_stream(
                renderer,
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            ),
            ControlKind::CookedSchedule {
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            } => begin_cooked_stream(
                renderer,
                world_region_side,
                active_center_x,
                active_center_z,
                active_radius,
            ),
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
                    message: "a capture request is already pending".into(),
                })
            }
            ControlKind::PerceptionCapture {
                id,
                collection,
                region,
                samples,
            } => match perception::Request::new(region, samples, window::WIDTH, window::HEIGHT) {
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
                    message: "a capture request is already pending".into(),
                }),
                Err(error) => Err(ProtocolError {
                    code: "invalid_region",
                    message: error.to_string(),
                }),
            },
        };
        let _ = response.send(result);
    }
}

fn begin_resident_stream(
    renderer: &mut Renderer,
    pending: &mut PendingOperations,
    response: &SyncSender<ControlResult>,
    world_region_side: u32,
    active_center_x: u32,
    active_center_z: u32,
    active_radius: u32,
) -> Option<ControlResult> {
    let config = match load::LoadConfig::new(
        world_region_side,
        active_center_x,
        active_center_z,
        active_radius,
    ) {
        Ok(config) => config,
        Err(error) => {
            return Some(Err(ProtocolError {
                code: "invalid_load_config",
                message: error.to_string(),
            }));
        }
    };
    if !pending.is_idle() {
        return Some(Err(ProtocolError {
            code: "capture_busy",
            message: "a capture, probe, or stream request is already pending".into(),
        }));
    }
    match unsafe { renderer.stream_resident(config) } {
        Ok(()) => {
            pending.stream = Some(response.clone());
            None
        }
        Err(error) => Some(Err(ProtocolError {
            code: "stream_failed",
            message: error.to_string(),
        })),
    }
}

fn begin_async_stream(
    renderer: &mut Renderer,
    world_region_side: u32,
    active_center_x: u32,
    active_center_z: u32,
    active_radius: u32,
) -> ControlResult {
    let config = load::LoadConfig::new(
        world_region_side,
        active_center_x,
        active_center_z,
        active_radius,
    )
    .map_err(|error| ProtocolError {
        code: "invalid_load_config",
        message: error.to_string(),
    })?;
    let report = unsafe { renderer.stream_async_resident(config) }.map_err(|error| {
        let message = error.to_string();
        ProtocolError {
            code: if message.contains("stream_busy") {
                "stream_busy"
            } else {
                "stream_failed"
            },
            message,
        }
    })?;
    serde_json::to_value(report).map_err(|error| ProtocolError {
        code: "stream_failed",
        message: error.to_string(),
    })
}

fn begin_cooked_stream(
    renderer: &mut Renderer,
    world_region_side: u32,
    active_center_x: u32,
    active_center_z: u32,
    active_radius: u32,
) -> ControlResult {
    let config = load::LoadConfig::new(
        world_region_side,
        active_center_x,
        active_center_z,
        active_radius,
    )
    .map_err(|error| ProtocolError {
        code: "invalid_load_config",
        message: error.to_string(),
    })?;
    let report = renderer.stream_cooked_resident(config).map_err(|error| {
        let message = error.to_string();
        ProtocolError {
            code: if message.contains("stream_busy") {
                "stream_busy"
            } else {
                "stream_failed"
            },
            message,
        }
    })?;
    serde_json::to_value(report).map_err(|error| ProtocolError {
        code: "stream_failed",
        message: error.to_string(),
    })
}
