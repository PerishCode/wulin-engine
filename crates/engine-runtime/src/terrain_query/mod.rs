use anyhow::{Context, Result, ensure};
use serde::Serialize;

use crate::streaming::address::GlobalRegionConfig;
use crate::terrain::TerrainAssignment;

mod motion;
mod position;

pub(crate) use motion::integrate_terrain_body_step;
pub use motion::{TerrainBodyMotion, TerrainBodyStep};
pub use position::{
    TERRAIN_POSITION_DENOMINATOR, TERRAIN_POSITION_LOCAL_MAX_Q9_EXCLUSIVE,
    TERRAIN_POSITION_LOCAL_MIN_Q9, TERRAIN_POSITION_REGION_SIDE_Q9, TerrainPosition,
};

pub const TERRAIN_QUERY_HEIGHT_DENOMINATOR: u32 = 65_536;
pub const TERRAIN_BODY_HEIGHT_DENOMINATOR: u32 = TERRAIN_QUERY_HEIGHT_DENOMINATOR;

const TERRAIN_CELL_SIDE_Q9: i32 = 256;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainBody {
    position: TerrainPosition,
    center_height_numerator: i32,
    half_height_numerator: i32,
}

impl TerrainBody {
    pub fn new(
        position: TerrainPosition,
        center_height_numerator: i32,
        half_height_numerator: i32,
    ) -> Result<Self> {
        ensure!(
            half_height_numerator > 0,
            "terrain body half-height numerator must be positive"
        );
        Ok(Self {
            position,
            center_height_numerator,
            half_height_numerator,
        })
    }

    pub const fn position(self) -> TerrainPosition {
        self.position
    }

    pub const fn center_height_numerator(self) -> i32 {
        self.center_height_numerator
    }

    pub const fn half_height_numerator(self) -> i32 {
        self.half_height_numerator
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TerrainContactClassification {
    Separated,
    Touching,
    Penetrating,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TerrainBodyContact {
    pub classification: TerrainContactClassification,
    pub terrain: TerrainHeight,
    pub separation_numerator: i64,
    pub correction_numerator: i64,
    pub resolved_body: TerrainBody,
    pub height_denominator: u32,
}

pub(crate) fn resolve_body_contact(
    body: TerrainBody,
    terrain: TerrainHeight,
) -> Result<TerrainBodyContact> {
    ensure!(
        terrain.height_denominator == TERRAIN_BODY_HEIGHT_DENOMINATOR,
        "terrain body contact height denominator disagrees with terrain query authority"
    );
    let foot_height_numerator = i64::from(body.center_height_numerator)
        .checked_sub(i64::from(body.half_height_numerator))
        .context("terrain body foot height overflowed signed 64-bit arithmetic")?;
    let separation_numerator = foot_height_numerator
        .checked_sub(i64::from(terrain.height_numerator))
        .context("terrain body separation overflowed signed 64-bit arithmetic")?;
    let classification = match separation_numerator.cmp(&0) {
        std::cmp::Ordering::Greater => TerrainContactClassification::Separated,
        std::cmp::Ordering::Equal => TerrainContactClassification::Touching,
        std::cmp::Ordering::Less => TerrainContactClassification::Penetrating,
    };
    let correction_numerator = if classification == TerrainContactClassification::Penetrating {
        separation_numerator
            .checked_neg()
            .context("terrain body correction overflowed signed 64-bit arithmetic")?
    } else {
        0
    };
    let resolved_center = i64::from(body.center_height_numerator)
        .checked_add(correction_numerator)
        .context("resolved terrain body center overflowed signed 64-bit arithmetic")?;
    let resolved_body = TerrainBody::new(
        body.position,
        i32::try_from(resolved_center)
            .context("resolved terrain body center is outside the signed 32-bit Q16 range")?,
        body.half_height_numerator,
    )?;

    Ok(TerrainBodyContact {
        classification,
        terrain,
        separation_numerator,
        correction_numerator,
        resolved_body,
        height_denominator: TERRAIN_BODY_HEIGHT_DENOMINATOR,
    })
}

pub(crate) fn query_published_height(
    config: GlobalRegionConfig,
    assignments: &[TerrainAssignment],
    tiles: &[terrain_format::TerrainTile],
    position: TerrainPosition,
) -> Result<TerrainHeight> {
    let expected_count = config.local_config()?.active_region_count() as usize;
    ensure!(
        assignments.len() == expected_count && tiles.len() == expected_count,
        "published terrain query snapshot has inconsistent active counts"
    );
    let addressed = config
        .addressed_region(position.region())?
        .context("terrain query region is outside the published active window")?;

    let mut matches = assignments
        .iter()
        .enumerate()
        .filter(|(_, assignment)| assignment.global_region == position.region());
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

    Ok(sample_tile(
        tile,
        position.local_x_q9(),
        position.local_z_q9(),
    ))
}

fn sample_tile(
    tile: &terrain_format::TerrainTile,
    local_x_q9: i32,
    local_z_q9: i32,
) -> TerrainHeight {
    let tile_x_q9 = local_x_q9 - TERRAIN_POSITION_LOCAL_MIN_Q9;
    let tile_z_q9 = local_z_q9 - TERRAIN_POSITION_LOCAL_MIN_Q9;
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
    use crate::region::RegionCoord;

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

    fn position() -> TerrainPosition {
        TerrainPosition::new(RegionCoord::ZERO, 0, 0).unwrap()
    }

    fn height(height_numerator: i32, triangle: TerrainTriangle) -> TerrainHeight {
        TerrainHeight {
            height_numerator,
            height_denominator: TERRAIN_QUERY_HEIGHT_DENOMINATOR,
            triangle,
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
        assert!(TerrainPosition::new(region, -4096, -4096).is_ok());
        assert!(TerrainPosition::new(region, 4095, 4095).is_ok());
        assert!(TerrainPosition::new(region, -4097, 0).is_err());
        assert!(TerrainPosition::new(region, 4096, 0).is_err());
        assert!(TerrainPosition::new(region, 0, -4097).is_err());
        assert!(TerrainPosition::new(region, 0, 4096).is_err());
    }

    #[test]
    fn seam_uses_adjacent_region() {
        let far = 1_i64 << 40;
        let config = GlobalRegionConfig::new(far, -far, far, -far, 1).unwrap();
        let (assignments, mut tiles) = snapshot(config, [256; 4]);
        for tile in &mut tiles {
            *tile = flat_tile(tile.region_id, 256);
        }
        let adjacent = TerrainPosition::new(RegionCoord::new(far + 1, -far), -4096, 0).unwrap();
        let height = query_published_height(config, &assignments, &tiles, adjacent).unwrap();
        assert_eq!(height.height_numerator, 65_536);
        assert!(TerrainPosition::new(RegionCoord::new(far, -far), 4096, 0).is_err());
    }

    #[test]
    fn rejects_snapshot_mismatch() {
        let config = GlobalRegionConfig::new(0, 0, 0, 0, 0).unwrap();
        let (mut assignments, mut tiles) = snapshot(config, [0; 4]);
        let position = TerrainPosition::new(RegionCoord::ZERO, 0, 0).unwrap();
        let outside = TerrainPosition::new(RegionCoord::new(1, 0), 0, 0).unwrap();
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

    #[test]
    fn resolves_contact_exactly() {
        let terrain = height(-100, TerrainTriangle::First);
        let separated = TerrainBody::new(position(), -89, 10).unwrap();
        let touching = TerrainBody::new(position(), -90, 10).unwrap();
        let penetrating = TerrainBody::new(position(), -91, 10).unwrap();

        let separated_contact = resolve_body_contact(separated, terrain).unwrap();
        assert_eq!(
            separated_contact.classification,
            TerrainContactClassification::Separated
        );
        assert_eq!(separated_contact.separation_numerator, 1);
        assert_eq!(separated_contact.correction_numerator, 0);
        assert_eq!(separated_contact.resolved_body, separated);

        let touching_contact = resolve_body_contact(touching, terrain).unwrap();
        assert_eq!(
            touching_contact.classification,
            TerrainContactClassification::Touching
        );
        assert_eq!(touching_contact.separation_numerator, 0);
        assert_eq!(touching_contact.correction_numerator, 0);
        assert_eq!(touching_contact.resolved_body, touching);

        let penetrating_contact = resolve_body_contact(penetrating, terrain).unwrap();
        assert_eq!(
            penetrating_contact.classification,
            TerrainContactClassification::Penetrating
        );
        assert_eq!(penetrating_contact.separation_numerator, -1);
        assert_eq!(penetrating_contact.correction_numerator, 1);
        assert_eq!(penetrating_contact.resolved_body, touching);
    }

    #[test]
    fn validates_shape_and_triangle() {
        for triangle in [
            TerrainTriangle::First,
            TerrainTriangle::Diagonal,
            TerrainTriangle::Second,
        ] {
            let body = TerrainBody::new(position(), 65_536, 65_536).unwrap();
            let contact = resolve_body_contact(body, height(0, triangle)).unwrap();
            assert_eq!(contact.terrain.triangle, triangle);
            assert_eq!(contact.height_denominator, 65_536);
        }
        assert!(TerrainBody::new(position(), 0, 0).is_err());
        assert!(TerrainBody::new(position(), 0, -1).is_err());
    }

    #[test]
    fn rejects_invalid_resolution() {
        let body = TerrainBody::new(position(), i32::MAX, 1).unwrap();
        let mismatch = TerrainHeight {
            height_numerator: 0,
            height_denominator: 1,
            triangle: TerrainTriangle::Diagonal,
        };
        assert!(resolve_body_contact(body, mismatch).is_err());

        let maximum = height(i32::MAX, TerrainTriangle::Second);
        assert!(resolve_body_contact(body, maximum).is_err());
        let extreme = TerrainBody::new(position(), i32::MIN, i32::MAX).unwrap();
        assert!(resolve_body_contact(extreme, maximum).is_err());
    }
}
