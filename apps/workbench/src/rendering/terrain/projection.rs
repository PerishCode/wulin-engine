use anyhow::{Result, ensure};

use crate::async_resident::ObjectSourceNamespace;
use crate::load::{LoadConfig, MAX_REGION_SIDE};
use crate::scene::Camera;
use crate::terrain::TerrainSourceNamespace;

const PROJECTION_CENTER: i32 = (MAX_REGION_SIDE / 2) as i32;
const REGION_SIDE_METERS: f32 = terrain_format::CELL_SIDE as f32 * 0.5;

#[derive(Clone, Copy)]
pub(in crate::rendering) struct TerrainProjection {
    config: LoadConfig,
    canonical: bool,
}

impl TerrainProjection {
    pub fn for_terrain(config: LoadConfig, source: Option<TerrainSourceNamespace>) -> Result<Self> {
        Self::new(config, source.is_some())
    }

    pub fn for_objects(config: LoadConfig, source: Option<ObjectSourceNamespace>) -> Result<Self> {
        Self::new(config, source.is_some())
    }

    fn new(config: LoadConfig, canonical: bool) -> Result<Self> {
        let side = config.active_radius * 2 + 1;
        ensure!(
            side * side == config.active_region_count(),
            "terrain projection shape is invalid"
        );
        ensure!(
            config.active_radius < MAX_REGION_SIDE / 2,
            "terrain projection radius exceeds the semantic window"
        );
        Ok(Self { config, canonical })
    }

    pub fn is_canonical(self) -> bool {
        self.canonical
    }

    pub fn active_count(self) -> usize {
        self.config.active_region_count() as usize
    }

    pub fn camera(self, mut camera: Camera) -> Camera {
        if !self.canonical {
            return camera;
        }
        let offset = self.alias_offset_meters();
        camera.position[0] -= offset[0];
        camera.position[2] -= offset[1];
        camera.target[0] -= offset[0];
        camera.target[2] -= offset[1];
        camera
    }

    pub fn region_id(self, active_index: usize, local_region_id: u32) -> Result<u32> {
        if !self.canonical {
            return Ok(local_region_id);
        }
        let [x, z] = self.render_offset(active_index)?;
        let region_x = PROJECTION_CENTER + x;
        let region_z = PROJECTION_CENTER + z;
        ensure!(
            (0..MAX_REGION_SIDE as i32).contains(&region_x)
                && (0..MAX_REGION_SIDE as i32).contains(&region_z),
            "terrain semantic projection is outside the bounded region grid"
        );
        Ok(region_z as u32 * MAX_REGION_SIDE + region_x as u32)
    }

    pub fn render_offset(self, active_index: usize) -> Result<[i32; 2]> {
        let side = (self.config.active_radius * 2 + 1) as usize;
        ensure!(
            active_index < self.config.active_region_count() as usize,
            "terrain projection active index is outside the window"
        );
        Ok([
            (active_index % side) as i32 - self.config.active_radius as i32,
            (active_index / side) as i32 - self.config.active_radius as i32,
        ])
    }

    pub fn position(self, active_index: usize, mut local: [f32; 3]) -> Result<[f32; 3]> {
        if !self.canonical {
            return Ok(local);
        }
        let offset = self.render_offset(active_index)?;
        local[0] += offset[0] as f32 * REGION_SIDE_METERS;
        local[2] += offset[1] as f32 * REGION_SIDE_METERS;
        Ok(local)
    }

    pub fn alias_center(self) -> [u32; 2] {
        [self.config.active_center_x, self.config.active_center_z]
    }

    pub fn alias_offset_regions(self) -> [i32; 2] {
        [
            self.config.active_center_x as i32 - PROJECTION_CENTER,
            self.config.active_center_z as i32 - PROJECTION_CENTER,
        ]
    }

    pub fn alias_offset_meters(self) -> [f32; 2] {
        self.alias_offset_regions()
            .map(|value| value as f32 * REGION_SIDE_METERS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera(offset_regions: i32) -> Camera {
        let offset = offset_regions as f32 * REGION_SIDE_METERS;
        Camera {
            position: [9.0 + offset, 6.0, 12.0],
            target: [offset, 1.0, -3.0],
            vertical_fov_degrees: 60.0,
            near_plane_meters: 0.1,
        }
    }

    #[test]
    fn canonical_aliases_project_identically() {
        let source = TerrainSourceNamespace([7; 32]);
        let base = TerrainProjection::for_terrain(
            LoadConfig::new(MAX_REGION_SIDE, 64, 64, 2).unwrap(),
            Some(source),
        )
        .unwrap();
        let base_camera = base.camera(camera(0));
        let base_ids = (0..base.active_count())
            .map(|index| base.region_id(index, 0).unwrap())
            .collect::<Vec<_>>();
        for center in [2, 64, 96, 125] {
            let projection = TerrainProjection::for_terrain(
                LoadConfig::new(MAX_REGION_SIDE, center, 64, 2).unwrap(),
                Some(source),
            )
            .unwrap();
            assert_eq!(
                projection
                    .camera(camera(center as i32 - 64))
                    .position
                    .map(f32::to_bits),
                base_camera.position.map(f32::to_bits)
            );
            assert_eq!(
                projection
                    .camera(camera(center as i32 - 64))
                    .target
                    .map(f32::to_bits),
                base_camera.target.map(f32::to_bits)
            );
            assert_eq!(
                (0..projection.active_count())
                    .map(|index| projection.region_id(index, u32::MAX).unwrap())
                    .collect::<Vec<_>>(),
                base_ids
            );
        }
    }

    #[test]
    fn legacy_projection_is_passthrough() {
        let projection = TerrainProjection::for_terrain(
            LoadConfig::new(MAX_REGION_SIDE, 64, 64, 2).unwrap(),
            None,
        )
        .unwrap();
        let camera = camera(0);
        assert_eq!(projection.camera(camera).position, camera.position);
        assert_eq!(projection.region_id(0, 1234).unwrap(), 1234);
        assert!(!projection.is_canonical());
    }
}
