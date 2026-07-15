mod camera;

pub use camera::Camera;
pub(crate) use camera::view_projection;

use anyhow::Result;
use serde_json::{Value, json};

pub struct SceneState {
    camera: Camera,
}

impl SceneState {
    pub fn new() -> Self {
        Self {
            camera: Camera::default(),
        }
    }

    pub fn reset_camera(&mut self) {
        self.camera = Camera::default();
    }

    pub fn set_camera(
        &mut self,
        position: [f32; 3],
        target: [f32; 3],
        vertical_fov_degrees: f32,
    ) -> Result<()> {
        self.camera = Camera::new(position, target, vertical_fov_degrees)?;
        Ok(())
    }

    pub(crate) fn set_camera_from_anchor(
        &mut self,
        anchor: [f32; 3],
        position_offset: [f32; 3],
        target_offset: [f32; 3],
        vertical_fov_degrees: f32,
    ) -> Result<()> {
        let add = |offset: [f32; 3]| {
            [
                anchor[0] + offset[0],
                anchor[1] + offset[1],
                anchor[2] + offset[2],
            ]
        };
        let candidate = Camera::new(
            add(position_offset),
            add(target_offset),
            vertical_fov_degrees,
        )?;
        self.camera = candidate;
        Ok(())
    }

    pub(crate) fn translate_camera_regions(&mut self, delta: [i32; 2]) -> Result<()> {
        self.camera = self.camera.translated_regions(delta)?;
        Ok(())
    }

    pub fn camera(&self) -> Camera {
        self.camera
    }

    pub fn camera_json(&self) -> Value {
        serde_json::to_value(self.camera).expect("camera serialization should not fail")
    }

    pub fn spatial_json(&self) -> Value {
        json!({
            "revision": "canonical-camera-space-v1",
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
        })
    }
}
