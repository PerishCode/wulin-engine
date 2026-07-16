use engine_runtime::{
    CANONICAL_OBJECT_NEAREST_CANDIDATE_CAPACITY, CanonicalObjectIdentity,
    CanonicalObjectResolution, ObjectSourceNamespace, RegionCoord, Runtime, TerrainPosition,
};
use serde_json::json;

use super::{ControlResult, protocol_error};

pub(super) fn resolve(
    runtime: &Runtime,
    source_namespace: [u8; 32],
    region_x: i64,
    region_z: i64,
    authored_local_id: u32,
) -> ControlResult {
    runtime
        .resolve_canonical_object(CanonicalObjectIdentity {
            source_namespace: ObjectSourceNamespace::from_bytes(source_namespace),
            region: RegionCoord::new(region_x, region_z),
            authored_local_id,
        })
        .and_then(|resolution| {
            let terrain_position = match resolution {
                CanonicalObjectResolution::Resolved(object) => Some(object.terrain_position()?),
                CanonicalObjectResolution::SourceReplaced
                | CanonicalObjectResolution::OutsidePublishedWindow => None,
            };
            Ok(json!({
                "revision": "typed-canonical-object-resolution-v1",
                "resolution": resolution,
                "terrainPosition": terrain_position,
                "perResolutionAllocationBytes": 0,
                "sourceReadCount": 0,
                "gpuCopyCount": 0,
                "gpuReadbackCount": 0,
                "fenceWaitCount": 0,
                "synchronizationCount": 0,
            }))
        })
        .map_err(|error| protocol_error("canonical_object_resolution_failed", error))
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
                        "revision": "source-qualified-canonical-object-nearest-v1",
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
