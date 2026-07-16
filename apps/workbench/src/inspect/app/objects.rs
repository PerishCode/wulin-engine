use engine_runtime::{RegionCoord, Runtime};
use serde_json::json;

use super::{ControlResult, protocol_error};

pub(super) fn query(
    runtime: &Runtime,
    region_x: i64,
    region_z: i64,
    authored_local_id: u32,
) -> ControlResult {
    runtime
        .query_canonical_object(RegionCoord::new(region_x, region_z), authored_local_id)
        .and_then(|object| {
            let terrain_position = object.terrain_position()?;
            Ok(json!({
                "revision": "exact-canonical-object-position-v1",
                "object": object,
                "terrainPosition": terrain_position,
                "perQueryAllocationBytes": 0,
                "sourceReadCount": 0,
                "gpuCopyCount": 0,
                "gpuReadbackCount": 0,
                "fenceWaitCount": 0,
                "synchronizationCount": 0,
            }))
        })
        .map_err(|error| protocol_error("canonical_object_query_failed", error))
}
