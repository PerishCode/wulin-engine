use anyhow::{Result, bail};
use glam::{Mat4, Vec3};
use serde::Serialize;

const DEFAULT_FOV_DEGREES: f32 = 60.0;
const DEFAULT_POSITION: [f32; 3] = [9.0, 6.0, 12.0];
const DEFAULT_TARGET: [f32; 3] = [0.0, 1.0, -3.0];
const NEAR_PLANE_METERS: f32 = 0.1;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Camera {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub vertical_fov_degrees: f32,
    pub near_plane_meters: f32,
}

impl Camera {
    pub(super) fn new(
        position: [f32; 3],
        target: [f32; 3],
        vertical_fov_degrees: f32,
    ) -> Result<Self> {
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
        Ok(Self {
            position,
            target,
            vertical_fov_degrees,
            near_plane_meters: NEAR_PLANE_METERS,
        })
    }

    pub(super) fn translated_regions(self, delta: [i32; 2]) -> Result<Self> {
        let region_meters = terrain_format::REGION_SIDE_METERS;
        let translation = [
            delta[0] as f32 * region_meters,
            0.0,
            delta[1] as f32 * region_meters,
        ];
        let translated = |value: [f32; 3]| {
            [
                value[0] + translation[0],
                value[1],
                value[2] + translation[2],
            ]
        };
        Self::new(
            translated(self.position),
            translated(self.target),
            self.vertical_fov_degrees,
        )
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: DEFAULT_POSITION,
            target: DEFAULT_TARGET,
            vertical_fov_degrees: DEFAULT_FOV_DEGREES,
            near_plane_meters: NEAR_PLANE_METERS,
        }
    }
}

pub(crate) fn view_projection(camera: Camera, aspect: f32) -> Mat4 {
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
