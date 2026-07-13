use anyhow::{Result, bail};
use glam::{Mat4, Quat, Vec3};
use serde::Serialize;
use serde_json::{Value, json};

use crate::load;

pub const SCENE_REVISION: &str = "calibration-v1";
pub const NEAR_PLANE_METERS: f32 = 0.1;
const DEFAULT_FOV_DEGREES: f32 = 60.0;
const DEFAULT_POSITION: [f32; 3] = [9.0, 6.0, 12.0];
const DEFAULT_TARGET: [f32; 3] = [0.0, 1.0, -3.0];

#[derive(Clone, Copy)]
pub enum MeshKind {
    Plane,
    Cube,
}

#[derive(Clone, Copy)]
pub struct SceneObject {
    pub id: u32,
    pub name: &'static str,
    pub kind: &'static str,
    pub mesh: MeshKind,
    pub translation: [f32; 3],
    pub scale: [f32; 3],
    pub color: [f32; 4],
    pub material: u32,
}

pub struct SemanticObject {
    pub name: String,
    pub kind: String,
    pub color: [f32; 4],
}

pub const OBJECTS: [SceneObject; 8] = [
    SceneObject {
        id: 1,
        name: "ground.reference",
        kind: "ground",
        mesh: MeshKind::Plane,
        translation: [0.0, 0.0, -3.0],
        scale: [24.0, 1.0, 24.0],
        color: [0.32, 0.35, 0.38, 1.0],
        material: 1,
    },
    SceneObject {
        id: 10,
        name: "axis.positive_x",
        kind: "axis",
        mesh: MeshKind::Cube,
        translation: [2.0, 0.06, 0.0],
        scale: [4.0, 0.12, 0.12],
        color: [0.95, 0.12, 0.1, 1.0],
        material: 0,
    },
    SceneObject {
        id: 11,
        name: "axis.positive_y",
        kind: "axis",
        mesh: MeshKind::Cube,
        translation: [0.0, 2.0, 0.0],
        scale: [0.12, 4.0, 0.12],
        color: [0.12, 0.9, 0.2, 1.0],
        material: 0,
    },
    SceneObject {
        id: 12,
        name: "axis.positive_z",
        kind: "axis",
        mesh: MeshKind::Cube,
        translation: [0.0, 0.06, 2.0],
        scale: [0.12, 0.12, 4.0],
        color: [0.12, 0.32, 0.95, 1.0],
        material: 0,
    },
    SceneObject {
        id: 100,
        name: "marker.near",
        kind: "marker",
        mesh: MeshKind::Cube,
        translation: [-3.0, 1.0, -2.0],
        scale: [2.0, 2.0, 2.0],
        color: [0.9, 0.2, 0.12, 1.0],
        material: 0,
    },
    SceneObject {
        id: 101,
        name: "marker.center",
        kind: "marker",
        mesh: MeshKind::Cube,
        translation: [0.0, 1.0, -5.0],
        scale: [2.0, 2.0, 2.0],
        color: [0.15, 0.75, 0.3, 1.0],
        material: 0,
    },
    SceneObject {
        id: 102,
        name: "marker.far",
        kind: "marker",
        mesh: MeshKind::Cube,
        translation: [3.0, 1.0, -8.0],
        scale: [2.0, 2.0, 2.0],
        color: [0.18, 0.35, 0.9, 1.0],
        material: 0,
    },
    SceneObject {
        id: 110,
        name: "block.occluder",
        kind: "occluder",
        mesh: MeshKind::Cube,
        translation: [0.0, 2.0, -2.5],
        scale: [1.5, 4.0, 1.5],
        color: [0.78, 0.68, 0.18, 1.0],
        material: 0,
    },
];

pub struct SceneState {
    camera: Camera,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Camera {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub vertical_fov_degrees: f32,
    pub near_plane_meters: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ObjectSnapshot {
    id: u32,
    name: &'static str,
    kind: &'static str,
    translation: [f32; 3],
    scale: [f32; 3],
    color: [f32; 4],
}

impl SceneState {
    pub fn new() -> Self {
        Self {
            camera: default_camera(),
        }
    }

    pub fn reset_camera(&mut self) {
        self.camera = default_camera();
    }

    pub fn set_camera(
        &mut self,
        position: [f32; 3],
        target: [f32; 3],
        vertical_fov_degrees: f32,
    ) -> Result<()> {
        let position_vector = Vec3::from_array(position);
        let target_vector = Vec3::from_array(target);
        let forward = target_vector - position_vector;
        if !position_vector.is_finite()
            || !target_vector.is_finite()
            || !vertical_fov_degrees.is_finite()
        {
            bail!("camera values must be finite");
        }
        if forward.length_squared() < 0.01 {
            bail!("camera position and target must be distinct");
        }
        if forward.normalize().cross(Vec3::Y).length_squared() < 0.0001 {
            bail!("camera forward cannot be parallel to world +Y");
        }
        if !(20.0..=100.0).contains(&vertical_fov_degrees) {
            bail!("verticalFovDegrees must be in the range 20..=100");
        }
        self.camera = Camera {
            position,
            target,
            vertical_fov_degrees,
            near_plane_meters: NEAR_PLANE_METERS,
        };
        Ok(())
    }

    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        let projection = Mat4::perspective_infinite_reverse_rh(
            self.camera.vertical_fov_degrees.to_radians(),
            aspect,
            self.camera.near_plane_meters,
        );
        let view = Mat4::look_at_rh(
            Vec3::from_array(self.camera.position),
            Vec3::from_array(self.camera.target),
            Vec3::Y,
        );
        projection * view
    }

    pub fn camera(&self) -> Camera {
        self.camera
    }

    pub fn camera_json(&self) -> Value {
        serde_json::to_value(self.camera).expect("camera serialization should not fail")
    }

    pub fn objects_json(&self) -> Value {
        json!({
            "sceneRevision": SCENE_REVISION,
            "objects": OBJECTS.iter().map(object_snapshot).collect::<Vec<_>>()
        })
    }

    pub fn spatial_json(&self) -> Value {
        json!({
            "sceneRevision": SCENE_REVISION,
            "coordinateSystem": {
                "handedness": "right",
                "rightAxis": "+X",
                "upAxis": "+Y",
                "cameraForwardAxis": "-Z",
                "worldUnit": "meter",
                "transformSemantics": "column-vector",
                "clipExpression": "projection * view * model * position"
            },
            "depth": {
                "ndcRange": [0.0, 1.0],
                "reverseZ": true,
                "infiniteFarPlane": true,
                "clearValue": 0.0,
                "comparison": "GREATER"
            },
            "camera": self.camera,
            "objects": OBJECTS.iter().map(object_snapshot).collect::<Vec<_>>()
        })
    }
}

impl SceneObject {
    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::from_array(self.scale),
            Quat::IDENTITY,
            Vec3::from_array(self.translation),
        )
    }
}

pub fn semantic_object(id: u32) -> Option<SemanticObject> {
    OBJECTS
        .iter()
        .find(|object| object.id == id)
        .map(|object| SemanticObject {
            name: object.name.into(),
            kind: object.kind.into(),
            color: object.color,
        })
        .or_else(|| {
            load::terrain_semantic(id).map(|region| SemanticObject {
                name: region.name,
                kind: region.kind,
                color: region.color,
            })
        })
        .or_else(|| {
            load::region_semantic(id).map(|region| SemanticObject {
                name: region.name,
                kind: region.kind,
                color: region.color,
            })
        })
}

fn default_camera() -> Camera {
    Camera {
        position: DEFAULT_POSITION,
        target: DEFAULT_TARGET,
        vertical_fov_degrees: DEFAULT_FOV_DEGREES,
        near_plane_meters: NEAR_PLANE_METERS,
    }
}

fn object_snapshot(object: &SceneObject) -> ObjectSnapshot {
    ObjectSnapshot {
        id: object.id,
        name: object.name,
        kind: object.kind,
        translation: object.translation,
        scale: object.scale,
        color: object.color,
    }
}
