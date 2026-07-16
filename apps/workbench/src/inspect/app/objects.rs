use engine_runtime::{
    CANONICAL_OBJECT_NEAREST_CANDIDATE_CAPACITY, RegionCoord, Runtime, TerrainPosition,
};
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

pub(super) fn nearest(
    runtime: &Runtime,
    region_x: i64,
    region_z: i64,
    local_x_q9: i32,
    local_z_q9: i32,
    max_distance_q9: u32,
) -> ControlResult {
    TerrainPosition::new(RegionCoord::new(region_x, region_z), local_x_q9, local_z_q9)
        .and_then(|origin| {
            runtime
                .query_nearest_canonical_object(origin, max_distance_q9)
                .map(|query| {
                    json!({
                        "revision": "exact-canonical-object-nearest-v1",
                        "origin": origin,
                        "maxDistanceQ9": max_distance_q9,
                        "query": query,
                        "maximumCandidateCount": CANONICAL_OBJECT_NEAREST_CANDIDATE_CAPACITY,
                        "perQueryAllocationBytes": 0,
                        "sourceReadCount": 0,
                        "gpuCopyCount": 0,
                        "gpuReadbackCount": 0,
                        "fenceWaitCount": 0,
                        "synchronizationCount": 0,
                    })
                })
        })
        .map_err(|error| protocol_error("canonical_object_nearest_failed", error))
}
