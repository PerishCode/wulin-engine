use anyhow::{Result, bail};
use glam::{Mat4, Quat, Vec3, Vec4};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::load;
use crate::world::{self, RegionCoord, WorldSpace};

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
    world: WorldSpace,
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
            world: WorldSpace::default(),
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
        require_world_bound: bool,
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
        if require_world_bound {
            self.world.render_position(position)?;
            self.world.render_position(target)?;
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
        camera_view_projection(self.camera, aspect)
    }

    pub fn calibration_view_projection(&self, aspect: f32) -> Result<Mat4> {
        let camera = self.calibration_camera()?;
        let projection = Mat4::perspective_infinite_reverse_rh(
            camera.vertical_fov_degrees.to_radians(),
            aspect,
            camera.near_plane_meters,
        );
        let direction =
            (Vec3::from_array(camera.target) - Vec3::from_array(camera.position)).normalize();
        Ok(projection * Mat4::look_to_rh(Vec3::ZERO, direction, Vec3::Y))
    }

    pub fn calibration_model_matrix(&self, object: &SceneObject) -> Result<Mat4> {
        let camera_position = self.world.render_position(self.camera.position)?;
        let object_position = self.world.render_position(object.translation)?;
        Ok(Mat4::from_scale_rotation_translation(
            Vec3::from_array(object.scale),
            Quat::IDENTITY,
            Vec3::from_array(object_position) - Vec3::from_array(camera_position),
        ))
    }

    pub fn calibration_semantic_offset(&self, object: &SceneObject) -> Result<[f32; 3]> {
        let camera_position = self.world.render_position(self.camera.position)?;
        let object_position = self.world.render_position(object.translation)?;
        let camera_relative = Vec3::from_array(object_position) - Vec3::from_array(camera_position);
        Ok([
            object.translation[0] - camera_relative.x,
            object.translation[1] - camera_relative.y,
            object.translation[2] - camera_relative.z,
        ])
    }

    fn calibration_camera(&self) -> Result<Camera> {
        Ok(Camera {
            position: self.world.render_position(self.camera.position)?,
            target: self.world.render_position(self.camera.target)?,
            vertical_fov_degrees: self.camera.vertical_fov_degrees,
            near_plane_meters: self.camera.near_plane_meters,
        })
    }

    pub fn relocate_world(&mut self, anchor: RegionCoord) -> Result<()> {
        self.world.relocate(anchor, &self.world_positions())
    }

    pub fn rebase_world(&mut self, origin: RegionCoord) -> Result<()> {
        self.world.rebase(origin, &self.world_positions())
    }

    pub fn reset_world(&mut self) -> Result<()> {
        self.world.reset(&self.world_positions())
    }

    pub fn world_json(&self) -> Result<Value> {
        let camera_position = self.world.split_position(self.camera.position)?;
        let camera_target = self.world.split_position(self.camera.target)?;
        let render_camera = self.calibration_camera()?;
        let objects = OBJECTS
            .iter()
            .map(|object| {
                let global = self.world.split_position(object.translation)?;
                let render = self.world.render_position(object.translation)?;
                Ok(json!({
                    "id": object.id,
                    "name": object.name,
                    "global": global,
                    "renderRelativeMeters": render,
                }))
            })
            .collect::<Result<Vec<_>>>()?;
        let mut status = self.world.status_json();
        let object = status
            .as_object_mut()
            .expect("world status must serialize as an object");
        object.insert(
            "camera".into(),
            json!({
                "sceneLocal": self.camera,
                "globalPosition": camera_position,
                "globalTarget": camera_target,
                "renderRelative": render_camera,
            }),
        );
        object.insert("objects".into(), Value::Array(objects));
        Ok(status)
    }

    pub fn world_probe_json(&self) -> Result<Value> {
        let mut probe = self.world.probe()?;
        let aspect = 1280.0 / 720.0;
        let local_view_projection = self.view_projection(aspect);
        let render_view_projection = self.calibration_view_projection(aspect)?;
        let mut local_matrix_hash = Sha256::new();
        let mut render_matrix_hash = Sha256::new();
        let mut canonical_clip_space_hash = Sha256::new();
        let mut render_clip_space_hash = Sha256::new();
        let mut max_clip_error = 0.0_f32;
        hash_matrix(&mut local_matrix_hash, local_view_projection);
        hash_matrix(&mut render_matrix_hash, render_view_projection);
        for object in &OBJECTS {
            let local_model = object.model_matrix();
            let render_model = self.calibration_model_matrix(object)?;
            hash_matrix(&mut local_matrix_hash, local_model);
            hash_matrix(&mut render_matrix_hash, render_model);
            let canonical_mvp = local_view_projection * local_model;
            let render_mvp = render_view_projection * render_model;
            for point in CLIP_PROBE_POINTS {
                let canonical = canonical_mvp * point;
                let render = render_mvp * point;
                world::hash_f32_array(&mut canonical_clip_space_hash, canonical.to_array());
                world::hash_f32_array(&mut render_clip_space_hash, render.to_array());
                max_clip_error = max_clip_error.max(
                    (canonical - render)
                        .abs()
                        .to_array()
                        .into_iter()
                        .fold(0.0_f32, f32::max),
                );
            }
        }
        let object = probe
            .as_object_mut()
            .expect("world probe must serialize as an object");
        object.insert(
            "localMatrixHash".into(),
            Value::String(world::digest_hex(local_matrix_hash)),
        );
        object.insert(
            "renderMatrixHash".into(),
            Value::String(world::digest_hex(render_matrix_hash)),
        );
        object.insert(
            "canonicalClipSpaceHash".into(),
            Value::String(world::digest_hex(canonical_clip_space_hash)),
        );
        object.insert(
            "renderClipSpaceHash".into(),
            Value::String(world::digest_hex(render_clip_space_hash)),
        );
        object.insert(
            "maximumClipSpaceAbsoluteError".into(),
            json!(max_clip_error),
        );
        Ok(probe)
    }

    fn world_positions(&self) -> Vec<[f32; 3]> {
        let mut positions = Vec::with_capacity(OBJECTS.len() + 2);
        positions.push(self.camera.position);
        positions.push(self.camera.target);
        positions.extend(OBJECTS.iter().map(|object| object.translation));
        positions
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
            "renderCamera": self.calibration_camera().ok(),
            "world": self.world.status_json(),
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

fn camera_view_projection(camera: Camera, aspect: f32) -> Mat4 {
    let projection = Mat4::perspective_infinite_reverse_rh(
        camera.vertical_fov_degrees.to_radians(),
        aspect,
        camera.near_plane_meters,
    );
    let view = Mat4::look_at_rh(
        Vec3::from_array(camera.position),
        Vec3::from_array(camera.target),
        Vec3::Y,
    );
    projection * view
}

fn hash_matrix(hasher: &mut Sha256, matrix: Mat4) {
    world::hash_f32_array(hasher, matrix.to_cols_array());
}

const CLIP_PROBE_POINTS: [Vec4; 9] = [
    Vec4::new(-1.0, -1.0, -1.0, 1.0),
    Vec4::new(-1.0, -1.0, 1.0, 1.0),
    Vec4::new(-1.0, 1.0, -1.0, 1.0),
    Vec4::new(-1.0, 1.0, 1.0, 1.0),
    Vec4::new(1.0, -1.0, -1.0, 1.0),
    Vec4::new(1.0, -1.0, 1.0, 1.0),
    Vec4::new(1.0, 1.0, -1.0, 1.0),
    Vec4::new(1.0, 1.0, 1.0, 1.0),
    Vec4::W,
];

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_relative_clip_matches() {
        let scene = SceneState::new();
        let local = scene.view_projection(1280.0 / 720.0) * OBJECTS[0].model_matrix();
        let relative = scene.calibration_view_projection(1280.0 / 720.0).unwrap()
            * scene.calibration_model_matrix(&OBJECTS[0]).unwrap();
        for point in CLIP_PROBE_POINTS {
            let expected = local * point;
            let actual = relative * point;
            assert!((expected - actual).abs().max_element() <= 0.0001);
        }
    }
}
