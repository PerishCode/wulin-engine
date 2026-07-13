use serde::Deserialize;
use serde_json::Value;

use crate::perception::{PixelPoint, PixelRegion};

pub enum ControlKind {
    Status,
    SetClearColor([f32; 4]),
    Pause,
    Resume,
    Capture {
        id: String,
        collection: String,
    },
    PerceptionCapture {
        id: String,
        collection: String,
        region: Option<PixelRegion>,
        samples: Vec<PixelPoint>,
    },
    CameraStatus,
    CameraSetPose {
        position: [f32; 3],
        target: [f32; 3],
        vertical_fov_degrees: f32,
    },
    CameraReset,
    SceneListObjects,
    WorldStatus,
    WorldRelocate {
        region_x: i64,
        region_z: i64,
    },
    WorldRebase {
        region_x: i64,
        region_z: i64,
    },
    WorldReset,
    WorldProbe,
    LoadConfigure {
        world_region_side: u32,
        active_center_x: u32,
        active_center_z: u32,
        active_radius: u32,
    },
    LoadDisable,
    LoadStatus,
    LoadProbe,
    ResidentStream {
        world_region_side: u32,
        active_center_x: u32,
        active_center_z: u32,
        active_radius: u32,
    },
    ResidentStatus,
    AsyncResidentSchedule {
        world_region_side: u32,
        active_center_x: u32,
        active_center_z: u32,
        active_radius: u32,
    },
    AsyncResidentStatus,
    AsyncCopyGateArm,
    AsyncCopyGateRelease,
    CookedOpen {
        path: String,
    },
    CookedSchedule {
        world_region_side: u32,
        active_center_x: u32,
        active_center_z: u32,
        active_radius: u32,
    },
    CookedStatus,
    CookedIoGateArm,
    CookedIoGateRelease,
    MeshletStatus,
    MeshletConfigure {
        archetype_mask: u32,
        forced_lod: Option<u32>,
    },
    MeshletEnable,
    MeshletDisable,
    SkeletalStatus,
    SkeletalConfigure {
        animated_percent: u32,
        bone_count: u32,
        phase_count: u32,
        time_tick: u32,
        unique_poses: bool,
        forced_lod: Option<u32>,
    },
    SkeletalEnable,
    SkeletalDisable,
    SurfaceStatus,
    SurfaceConfigure {
        material_count: u32,
        mip_level: u32,
    },
    SurfaceEnable,
    SurfaceDisable,
    SurfaceOcclusionEnable,
    SurfaceOcclusionDisable,
    SurfaceOcclusionReset,
    TerrainStatus,
    TerrainOpen {
        path: String,
    },
    TerrainSchedule {
        world_region_side: u32,
        active_center_x: u32,
        active_center_z: u32,
        active_radius: u32,
    },
    TerrainGlobalSchedule {
        origin_x: i64,
        origin_z: i64,
        center_x: i64,
        center_z: i64,
        active_radius: u32,
    },
    TerrainEnable,
    TerrainDisable,
    TerrainIoGateArm,
    TerrainIoGateRelease,
    TerrainCopyGateArm,
    TerrainCopyGateRelease,
    TerrainLodStatus,
    TerrainLodConfigure {
        near_patch_radius: u32,
        middle_patch_radius: u32,
        forced_lod: Option<u32>,
    },
    TerrainLodEnable,
    TerrainLodDisable,
    CompositionStatus,
    CompositionSchedule {
        world_region_side: u32,
        active_center_x: u32,
        active_center_z: u32,
        active_radius: u32,
    },
    CompositionGlobalSchedule(i64, i64, i64, i64, u32),
    CompositionEnable,
    CompositionDisable,
    CompositionTraversalEnable,
    CompositionTraversalDisable,
    CompositionPrefetchEnable,
    CompositionPrefetchDisable,
    CompositionOrder {
        terrain_first: bool,
    },
    CompositionFixture {
        arbitrary_q8: bool,
    },
}

pub type ControlResult = std::result::Result<Value, ProtocolError>;
type ParsedControl = std::result::Result<ControlKind, ProtocolError>;

#[derive(Debug)]
pub struct ProtocolError {
    pub code: &'static str,
    pub message: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ColorPayload {
    rgba: [f32; 4],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapturePayload {
    id: String,
    collection: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PerceptionPayload {
    id: String,
    collection: Option<String>,
    region: Option<PixelRegion>,
    #[serde(default)]
    samples: Vec<PixelPoint>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CameraPayload {
    position: [f32; 3],
    target: [f32; 3],
    vertical_fov_degrees: f32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LoadPayload {
    world_region_side: u32,
    active_center_x: u32,
    active_center_z: u32,
    active_radius: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CookedPayload {
    path: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MeshletPayload {
    archetype_mask: u32,
    forced_lod: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkeletalPayload {
    animated_percent: u32,
    bone_count: u32,
    phase_count: u32,
    time_tick: u32,
    unique_poses: bool,
    forced_lod: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SurfacePayload {
    material_count: u32,
    mip_level: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TerrainLodPayload {
    near_patch_radius: u32,
    middle_patch_radius: u32,
    forced_lod: Option<u32>,
}

pub fn parse_control(verb: &str, payload: Value) -> ParsedControl {
    match verb {
        "workbench.status" => Ok(ControlKind::Status),
        "workbench.pause" => Ok(ControlKind::Pause),
        "workbench.resume" => Ok(ControlKind::Resume),
        "camera.status" => Ok(ControlKind::CameraStatus),
        "camera.reset" => Ok(ControlKind::CameraReset),
        "scene.list_objects" => Ok(ControlKind::SceneListObjects),
        "world.status" => Ok(ControlKind::WorldStatus),
        "world.relocate" => super::world_control::parse_relocate(payload),
        "world.rebase" => super::world_control::parse_rebase(payload),
        "world.reset" => Ok(ControlKind::WorldReset),
        "world.probe" => Ok(ControlKind::WorldProbe),
        "load.disable" => Ok(ControlKind::LoadDisable),
        "load.status" => Ok(ControlKind::LoadStatus),
        "load.probe" => Ok(ControlKind::LoadProbe),
        "resident.status" => Ok(ControlKind::ResidentStatus),
        "async.status" => Ok(ControlKind::AsyncResidentStatus),
        "async.gate.arm" => Ok(ControlKind::AsyncCopyGateArm),
        "async.gate.release" => Ok(ControlKind::AsyncCopyGateRelease),
        "cooked.status" => Ok(ControlKind::CookedStatus),
        "cooked.gate.arm" => Ok(ControlKind::CookedIoGateArm),
        "cooked.gate.release" => Ok(ControlKind::CookedIoGateRelease),
        "meshlet.status" => Ok(ControlKind::MeshletStatus),
        "meshlet.enable" => Ok(ControlKind::MeshletEnable),
        "meshlet.disable" => Ok(ControlKind::MeshletDisable),
        "meshlet.configure" => parse_meshlet(payload),
        "skeletal.status" => Ok(ControlKind::SkeletalStatus),
        "skeletal.enable" => Ok(ControlKind::SkeletalEnable),
        "skeletal.disable" => Ok(ControlKind::SkeletalDisable),
        "skeletal.configure" => parse_skeletal(payload),
        "surface.status" => Ok(ControlKind::SurfaceStatus),
        "surface.enable" => Ok(ControlKind::SurfaceEnable),
        "surface.disable" => Ok(ControlKind::SurfaceDisable),
        "surface.configure" => parse_surface(payload),
        "surface.occlusion.enable" => Ok(ControlKind::SurfaceOcclusionEnable),
        "surface.occlusion.disable" => Ok(ControlKind::SurfaceOcclusionDisable),
        "surface.occlusion.reset" => Ok(ControlKind::SurfaceOcclusionReset),
        "terrain.status" => Ok(ControlKind::TerrainStatus),
        "terrain.enable" => Ok(ControlKind::TerrainEnable),
        "terrain.disable" => Ok(ControlKind::TerrainDisable),
        "terrain.io_gate.arm" => Ok(ControlKind::TerrainIoGateArm),
        "terrain.io_gate.release" => Ok(ControlKind::TerrainIoGateRelease),
        "terrain.copy_gate.arm" => Ok(ControlKind::TerrainCopyGateArm),
        "terrain.copy_gate.release" => Ok(ControlKind::TerrainCopyGateRelease),
        "terrain.lod.status" => Ok(ControlKind::TerrainLodStatus),
        "terrain.lod.enable" => Ok(ControlKind::TerrainLodEnable),
        "terrain.lod.disable" => Ok(ControlKind::TerrainLodDisable),
        "terrain.lod.configure" => parse_terrain_lod(payload),
        "composition.status" => Ok(ControlKind::CompositionStatus),
        "composition.enable" => Ok(ControlKind::CompositionEnable),
        "composition.disable" => Ok(ControlKind::CompositionDisable),
        "composition.traversal.enable" => Ok(ControlKind::CompositionTraversalEnable),
        "composition.traversal.disable" => Ok(ControlKind::CompositionTraversalDisable),
        "composition.prefetch.enable" => Ok(ControlKind::CompositionPrefetchEnable),
        "composition.prefetch.disable" => Ok(ControlKind::CompositionPrefetchDisable),
        "composition.order" => super::composition_control::parse_order(payload),
        "composition.fixture" => super::composition_control::parse_fixture(payload),
        "composition.global.schedule" => super::composition_control::parse_global(payload),
        "terrain.open" => parse_terrain(payload),
        "cooked.open" => parse_cooked(payload),
        "load.configure" => parse_load(payload, LoadTarget::Procedural),
        "resident.stream" => parse_load(payload, LoadTarget::Resident),
        "async.schedule" => parse_load(payload, LoadTarget::Async),
        "cooked.schedule" => parse_load(payload, LoadTarget::Cooked),
        "terrain.schedule" => parse_load(payload, LoadTarget::Terrain),
        "terrain.global.schedule" => super::terrain_control::parse_global(payload),
        "composition.schedule" => parse_load(payload, LoadTarget::Composition),
        "camera.set_pose" => parse_camera(payload),
        "workbench.capture" => parse_capture(payload),
        "perception.capture" => parse_perception(payload),
        "workbench.set_clear_color" => parse_color(payload),
        _ => Err(ProtocolError {
            code: "unknown_event",
            message: format!("unsupported event {verb:?}"),
        }),
    }
}

enum LoadTarget {
    Procedural,
    Resident,
    Async,
    Cooked,
    Terrain,
    Composition,
}

fn parse_load(value: Value, target: LoadTarget) -> ParsedControl {
    let payload: LoadPayload = decode(value)?;
    let LoadPayload {
        world_region_side,
        active_center_x,
        active_center_z,
        active_radius,
    } = payload;
    Ok(match target {
        LoadTarget::Procedural => ControlKind::LoadConfigure {
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        },
        LoadTarget::Resident => ControlKind::ResidentStream {
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        },
        LoadTarget::Async => ControlKind::AsyncResidentSchedule {
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        },
        LoadTarget::Cooked => ControlKind::CookedSchedule {
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        },
        LoadTarget::Terrain => ControlKind::TerrainSchedule {
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        },
        LoadTarget::Composition => ControlKind::CompositionSchedule {
            world_region_side,
            active_center_x,
            active_center_z,
            active_radius,
        },
    })
}

fn parse_meshlet(value: Value) -> ParsedControl {
    let payload: MeshletPayload = decode(value)?;
    Ok(ControlKind::MeshletConfigure {
        archetype_mask: payload.archetype_mask,
        forced_lod: payload.forced_lod,
    })
}

fn parse_skeletal(value: Value) -> ParsedControl {
    let payload: SkeletalPayload = decode(value)?;
    Ok(ControlKind::SkeletalConfigure {
        animated_percent: payload.animated_percent,
        bone_count: payload.bone_count,
        phase_count: payload.phase_count,
        time_tick: payload.time_tick,
        unique_poses: payload.unique_poses,
        forced_lod: payload.forced_lod,
    })
}

fn parse_surface(value: Value) -> ParsedControl {
    let payload: SurfacePayload = decode(value)?;
    Ok(ControlKind::SurfaceConfigure {
        material_count: payload.material_count,
        mip_level: payload.mip_level,
    })
}

fn parse_cooked(value: Value) -> ParsedControl {
    let payload: CookedPayload = decode(value)?;
    Ok(ControlKind::CookedOpen { path: payload.path })
}

fn parse_terrain(value: Value) -> ParsedControl {
    let payload: CookedPayload = decode(value)?;
    Ok(ControlKind::TerrainOpen { path: payload.path })
}

fn parse_terrain_lod(value: Value) -> ParsedControl {
    let payload: TerrainLodPayload = decode(value)?;
    Ok(ControlKind::TerrainLodConfigure {
        near_patch_radius: payload.near_patch_radius,
        middle_patch_radius: payload.middle_patch_radius,
        forced_lod: payload.forced_lod,
    })
}

fn parse_camera(value: Value) -> ParsedControl {
    let payload: CameraPayload = decode(value)?;
    Ok(ControlKind::CameraSetPose {
        position: payload.position,
        target: payload.target,
        vertical_fov_degrees: payload.vertical_fov_degrees,
    })
}

fn parse_capture(value: Value) -> ParsedControl {
    let payload: CapturePayload = decode(value)?;
    validate_name("capture id", &payload.id)?;
    let collection = payload.collection.unwrap_or_else(|| "operator".into());
    validate_name("collection", &collection)?;
    Ok(ControlKind::Capture {
        id: payload.id,
        collection,
    })
}

fn parse_perception(value: Value) -> ParsedControl {
    let payload: PerceptionPayload = decode(value)?;
    validate_name("capture id", &payload.id)?;
    let collection = payload.collection.unwrap_or_else(|| "operator".into());
    validate_name("collection", &collection)?;
    Ok(ControlKind::PerceptionCapture {
        id: payload.id,
        collection,
        region: payload.region,
        samples: payload.samples,
    })
}

fn parse_color(value: Value) -> ParsedControl {
    let payload: ColorPayload = decode(value)?;
    if payload.rgba.iter().any(|value| !value.is_finite())
        || payload
            .rgba
            .iter()
            .any(|value| !(0.0..=1.0).contains(value))
    {
        return Err(ProtocolError {
            code: "invalid_payload",
            message: "rgba values must be finite numbers in the range 0..=1".into(),
        });
    }
    Ok(ControlKind::SetClearColor(payload.rgba))
}

fn decode<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, ProtocolError> {
    serde_json::from_value(value).map_err(|error| ProtocolError {
        code: "invalid_payload",
        message: error.to_string(),
    })
}

fn validate_name(label: &str, value: &str) -> Result<(), ProtocolError> {
    let valid = !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid {
        Ok(())
    } else {
        Err(ProtocolError {
            code: "invalid_payload",
            message: format!("{label} must contain 1..=64 ASCII letters, digits, '-' or '_'"),
        })
    }
}
