use anyhow::{Result, ensure};
use serde::Serialize;

use crate::region::RegionCoord;
use crate::streaming::async_resident::ObjectSourceNamespace;
use crate::terrain_query::{
    TERRAIN_POSITION_DENOMINATOR, TERRAIN_POSITION_LOCAL_MAX_Q9_EXCLUSIVE,
    TERRAIN_POSITION_LOCAL_MIN_Q9, TerrainPosition,
};

pub const CANONICAL_OBJECTS_PER_REGION: u32 = region_format::RECORDS_PER_REGION;
pub const CANONICAL_OBJECT_NEAREST_CANDIDATE_CAPACITY: u32 =
    crate::resident::ACTIVE_REGION_CAPACITY as u32 * CANONICAL_OBJECTS_PER_REGION;

pub use region_format::PresentationRecord as CanonicalObjectPresentation;

/// One exact address inside a specific committed canonical object source.
///
/// The source namespace prevents an address from silently aliasing content after source
/// replacement. It is not a persistent gameplay or network identifier.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalObjectIdentity {
    pub source_namespace: ObjectSourceNamespace,
    pub region: RegionCoord,
    pub authored_local_id: u32,
}

/// One exact authored triple from the current committed canonical object snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalObject {
    pub identity: CanonicalObjectIdentity,
    pub position: [f32; 3],
    pub height: f32,
    pub presentation: CanonicalObjectPresentation,
}

/// Typed lifetime result for one source-qualified canonical object identity.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(tag = "outcome", content = "object", rename_all = "kebab-case")]
pub enum CanonicalObjectResolution {
    Resolved(CanonicalObject),
    SourceReplaced,
    OutsidePublishedWindow,
}

/// One exact nearest-object candidate from the current committed snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalObjectNearest {
    pub object: CanonicalObject,
    pub terrain_position: TerrainPosition,
    pub delta_x_q9: i64,
    pub delta_z_q9: i64,
    pub distance_squared_q18: u64,
}

/// Bounded nearest-object query output over every committed active CPU object page.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalObjectNearestQuery {
    pub candidate_count: u32,
    pub nearest: Option<CanonicalObjectNearest>,
}

impl CanonicalObject {
    /// Converts the authored planar position to the sole canonical terrain-position domain.
    ///
    /// The identity owner region remains unchanged. An authored `+8m` edge normalizes spatially
    /// into the adjacent region because [`TerrainPosition`] uses half-open local axes.
    pub fn terrain_position(self) -> Result<TerrainPosition> {
        let local_x_q9 = exact_local_q9("X", self.position[0])?;
        let local_z_q9 = exact_local_q9("Z", self.position[2])?;
        TerrainPosition::new(
            self.identity.region,
            TERRAIN_POSITION_LOCAL_MIN_Q9,
            TERRAIN_POSITION_LOCAL_MIN_Q9,
        )?
        .translated_q9(
            local_x_q9 - TERRAIN_POSITION_LOCAL_MIN_Q9,
            local_z_q9 - TERRAIN_POSITION_LOCAL_MIN_Q9,
        )
    }
}

fn exact_local_q9(axis: &str, value: f32) -> Result<i32> {
    ensure!(
        value.is_finite(),
        "canonical object local {axis} position is not finite"
    );
    let scaled = value * TERRAIN_POSITION_DENOMINATOR as f32;
    ensure!(
        (TERRAIN_POSITION_LOCAL_MIN_Q9 as f32..=TERRAIN_POSITION_LOCAL_MAX_Q9_EXCLUSIVE as f32)
            .contains(&scaled),
        "canonical object local {axis} position is outside the closed authored region"
    );
    ensure!(
        scaled == scaled.round(),
        "canonical object local {axis} position is outside the exact Q9 lattice"
    );
    Ok(scaled as i32)
}
