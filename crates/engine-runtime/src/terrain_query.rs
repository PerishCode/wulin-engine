use anyhow::{Context, Result, ensure};
use serde::Serialize;

use crate::streaming::address::GlobalRegionConfig;
use crate::terrain::TerrainAssignment;
use crate::world::RegionCoord;

pub const TERRAIN_QUERY_POSITION_DENOMINATOR: i32 = 512;
pub const TERRAIN_QUERY_HEIGHT_DENOMINATOR: u32 = 65_536;
pub const TERRAIN_QUERY_LOCAL_MIN_Q9: i32 = -4096;
pub const TERRAIN_QUERY_LOCAL_MAX_Q9_EXCLUSIVE: i32 = 4096;

const TERRAIN_CELL_SIDE_Q9: i32 = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainQueryPosition {
    region: RegionCoord,
    local_x_q9: i32,
    local_z_q9: i32,
}

impl TerrainQueryPosition {
    pub fn new(region: RegionCoord, local_x_q9: i32, local_z_q9: i32) -> Result<Self> {
        ensure_local_coordinate("X", local_x_q9)?;
        ensure_local_coordinate("Z", local_z_q9)?;
        Ok(Self {
            region,
            local_x_q9,
            local_z_q9,
        })
    }

    pub const fn region(self) -> RegionCoord {
        self.region
    }

    pub const fn local_x_q9(self) -> i32 {
        self.local_x_q9
    }

    pub const fn local_z_q9(self) -> i32 {
        self.local_z_q9
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TerrainTriangle {
    First,
    Diagonal,
    Second,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainHeight {
    pub height_numerator: i32,
    pub height_denominator: u32,
    pub triangle: TerrainTriangle,
}

pub(crate) fn query_published_height(
    config: GlobalRegionConfig,
    assignments: &[TerrainAssignment],
    tiles: &[terrain_format::TerrainTile],
    position: TerrainQueryPosition,
) -> Result<TerrainHeight> {
    let expected_count = config.local_config()?.active_region_count() as usize;
    ensure!(
        assignments.len() == expected_count && tiles.len() == expected_count,
        "published terrain query snapshot has inconsistent active counts"
    );
    let addressed = config
        .addressed_region(position.region)?
        .context("terrain query region is outside the published active window")?;

    let mut matches = assignments
        .iter()
        .enumerate()
        .filter(|(_, assignment)| assignment.global_region == position.region);
    let (index, assignment) = matches
        .next()
        .context("published terrain query snapshot is missing the addressed region")?;
    ensure!(
        matches.next().is_none(),
        "published terrain query snapshot contains duplicate global-region identity"
    );
    ensure!(
        assignment.region_id == addressed.local_region_id,
        "published terrain query assignment disagrees with canonical addressing"
    );
    let tile = &tiles[index];
    ensure!(
        tile.region_id == assignment.region_id,
        "published terrain query tile identity disagrees with its assignment"
    );

    Ok(sample_tile(tile, position.local_x_q9, position.local_z_q9))
}

fn ensure_local_coordinate(axis: &str, value: i32) -> Result<()> {
    ensure!(
        (TERRAIN_QUERY_LOCAL_MIN_Q9..TERRAIN_QUERY_LOCAL_MAX_Q9_EXCLUSIVE).contains(&value),
        "terrain query local {axis} Q9 coordinate must be in [{TERRAIN_QUERY_LOCAL_MIN_Q9}, {TERRAIN_QUERY_LOCAL_MAX_Q9_EXCLUSIVE})"
    );
    Ok(())
}

fn sample_tile(
    tile: &terrain_format::TerrainTile,
    local_x_q9: i32,
    local_z_q9: i32,
) -> TerrainHeight {
    let tile_x_q9 = local_x_q9 - TERRAIN_QUERY_LOCAL_MIN_Q9;
    let tile_z_q9 = local_z_q9 - TERRAIN_QUERY_LOCAL_MIN_Q9;
    let cell_x = (tile_x_q9 / TERRAIN_CELL_SIDE_Q9) as usize;
    let cell_z = (tile_z_q9 / TERRAIN_CELL_SIDE_Q9) as usize;
    let u = tile_x_q9 - cell_x as i32 * TERRAIN_CELL_SIDE_Q9;
    let v = tile_z_q9 - cell_z as i32 * TERRAIN_CELL_SIDE_Q9;
    let sum = u + v;
    let at = |x: usize, z: usize| i32::from(tile.heights[z * terrain_format::SAMPLE_SIDE + x]);
    let height_numerator = if sum <= TERRAIN_CELL_SIDE_Q9 {
        at(cell_x, cell_z) * (TERRAIN_CELL_SIDE_Q9 - sum)
            + at(cell_x + 1, cell_z) * u
            + at(cell_x, cell_z + 1) * v
    } else {
        at(cell_x + 1, cell_z) * (TERRAIN_CELL_SIDE_Q9 - v)
            + at(cell_x, cell_z + 1) * (TERRAIN_CELL_SIDE_Q9 - u)
            + at(cell_x + 1, cell_z + 1) * (sum - TERRAIN_CELL_SIDE_Q9)
    };
    let triangle = match sum.cmp(&TERRAIN_CELL_SIDE_Q9) {
        std::cmp::Ordering::Less => TerrainTriangle::First,
        std::cmp::Ordering::Equal => TerrainTriangle::Diagonal,
        std::cmp::Ordering::Greater => TerrainTriangle::Second,
    };
    TerrainHeight {
        height_numerator,
        height_denominator: TERRAIN_QUERY_HEIGHT_DENOMINATOR,
        triangle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile(region_id: u32, corners: [i16; 4]) -> terrain_format::TerrainTile {
        let mut heights = [0; terrain_format::HEIGHT_COUNT];
        heights[0] = corners[0];
        heights[1] = corners[1];
        heights[terrain_format::SAMPLE_SIDE] = corners[2];
        heights[terrain_format::SAMPLE_SIDE + 1] = corners[3];
        terrain_format::TerrainTile {
            region_id,
            heights,
            materials: [0; terrain_format::MATERIAL_COUNT],
        }
    }

    fn flat_tile(region_id: u32, height: i16) -> terrain_format::TerrainTile {
        terrain_format::TerrainTile {
            region_id,
            heights: [height; terrain_format::HEIGHT_COUNT],
            materials: [0; terrain_format::MATERIAL_COUNT],
        }
    }

    fn snapshot(
        config: GlobalRegionConfig,
        corners: [i16; 4],
    ) -> (Vec<TerrainAssignment>, Vec<terrain_format::TerrainTile>) {
        let addressed = config.addressed_regions().unwrap();
        let assignments = addressed
            .iter()
            .enumerate()
            .map(|(index, value)| TerrainAssignment {
                slot: index as u32,
                region_id: value.local_region_id,
                global_region: value.global_region,
            })
            .collect::<Vec<_>>();
        let tiles = addressed
            .iter()
            .map(|value| tile(value.local_region_id, corners))
            .collect();
        (assignments, tiles)
    }

    #[test]
    fn samples_triangles_exactly() {
        let tile = tile(0, [-256, 256, 512, -512]);
        let first = sample_tile(&tile, -4032, -4032);
        let diagonal = sample_tile(&tile, -3968, -3968);
        let second = sample_tile(&tile, -3904, -3904);
        assert_eq!(first.triangle, TerrainTriangle::First);
        assert_eq!(first.height_numerator, 16_384);
        assert_eq!(diagonal.triangle, TerrainTriangle::Diagonal);
        assert_eq!(diagonal.height_numerator, 98_304);
        assert_eq!(second.triangle, TerrainTriangle::Second);
        assert_eq!(second.height_numerator, -16_384);
        assert_eq!(second.height_denominator, 65_536);
    }

    #[test]
    fn enforces_half_open_coordinates() {
        let region = RegionCoord::new(7, -9);
        assert!(TerrainQueryPosition::new(region, -4096, -4096).is_ok());
        assert!(TerrainQueryPosition::new(region, 4095, 4095).is_ok());
        assert!(TerrainQueryPosition::new(region, -4097, 0).is_err());
        assert!(TerrainQueryPosition::new(region, 4096, 0).is_err());
        assert!(TerrainQueryPosition::new(region, 0, -4097).is_err());
        assert!(TerrainQueryPosition::new(region, 0, 4096).is_err());
    }

    #[test]
    fn seam_uses_adjacent_region() {
        let far = 1_i64 << 40;
        let config = GlobalRegionConfig::new(far, -far, far, -far, 1).unwrap();
        let (assignments, mut tiles) = snapshot(config, [256; 4]);
        for tile in &mut tiles {
            *tile = flat_tile(tile.region_id, 256);
        }
        let adjacent =
            TerrainQueryPosition::new(RegionCoord::new(far + 1, -far), -4096, 0).unwrap();
        let height = query_published_height(config, &assignments, &tiles, adjacent).unwrap();
        assert_eq!(height.height_numerator, 65_536);
        assert!(TerrainQueryPosition::new(RegionCoord::new(far, -far), 4096, 0).is_err());
    }

    #[test]
    fn rejects_snapshot_mismatch() {
        let config = GlobalRegionConfig::new(0, 0, 0, 0, 0).unwrap();
        let (mut assignments, mut tiles) = snapshot(config, [0; 4]);
        let position = TerrainQueryPosition::new(RegionCoord::ZERO, 0, 0).unwrap();
        let outside = TerrainQueryPosition::new(RegionCoord::new(1, 0), 0, 0).unwrap();
        assert!(query_published_height(config, &assignments, &tiles, outside).is_err());

        assignments[0].region_id += 1;
        assert!(query_published_height(config, &assignments, &tiles, position).is_err());
        assignments[0].region_id -= 1;
        tiles[0].region_id += 1;
        assert!(query_published_height(config, &assignments, &tiles, position).is_err());
    }

    #[test]
    fn bounds_extreme_interpolation() {
        let maximum = flat_tile(0, i16::MAX);
        let minimum = flat_tile(0, i16::MIN);
        assert_eq!(
            sample_tile(&maximum, 4095, 4095).height_numerator,
            8_388_352
        );
        assert_eq!(
            sample_tile(&minimum, 4095, 4095).height_numerator,
            -8_388_608
        );
    }
}
