use serde::Serialize;

use crate::region::RegionCoord;

pub const CANONICAL_OBJECTS_PER_REGION: u32 = region_format::RECORDS_PER_REGION;

pub use region_format::PresentationRecord as CanonicalObjectPresentation;

/// One exact authored triple from the current committed canonical object snapshot.
///
/// `authored_local_id` is unique only within `region` and the current object source. It is not a
/// persistent gameplay or network identifier.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalObject {
    pub region: RegionCoord,
    pub authored_local_id: u32,
    pub position: [f32; 3],
    pub height: f32,
    pub presentation: CanonicalObjectPresentation,
}
