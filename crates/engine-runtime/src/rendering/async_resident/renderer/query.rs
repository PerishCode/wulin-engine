use anyhow::{Context, Result, ensure};

use crate::async_resident::canonical_stable_seed;
use crate::runtime::{
    CANONICAL_OBJECT_NEAREST_CANDIDATE_CAPACITY, CANONICAL_OBJECTS_PER_REGION, CanonicalObject,
    CanonicalObjectIdentity, CanonicalObjectNearest, CanonicalObjectNearestQuery,
    CanonicalObjectResolution,
};
use crate::terrain_query::TerrainPosition;

use super::{AsyncResidentRenderer, PublishedSnapshot};
use crate::rendering::async_resident::transfer::CpuObjectPage;

impl AsyncResidentRenderer {
    pub(in crate::rendering) fn resolve_canonical_object(
        &self,
        identity: CanonicalObjectIdentity,
    ) -> Result<CanonicalObjectResolution> {
        let snapshot = self
            .published
            .as_ref()
            .context("canonical object resolution requires a published snapshot")?;
        resolve_snapshot(snapshot, identity)
    }

    pub(in crate::rendering) fn query_nearest_canonical_object(
        &self,
        origin: TerrainPosition,
        max_distance_q9: u32,
        excluded_identity: Option<CanonicalObjectIdentity>,
    ) -> Result<CanonicalObjectNearestQuery> {
        let snapshot = self
            .published
            .as_ref()
            .context("canonical object nearest query requires a published snapshot")?;
        query_nearest_snapshot(snapshot, origin, max_distance_q9, excluded_identity)
    }
}

fn resolve_snapshot(
    snapshot: &PublishedSnapshot,
    identity: CanonicalObjectIdentity,
) -> Result<CanonicalObjectResolution> {
    ensure!(
        identity.authored_local_id < CANONICAL_OBJECTS_PER_REGION,
        "canonical object authored local ID is outside the fixed region capacity"
    );
    require_snapshot_shape(snapshot)?;
    if identity.source_namespace != snapshot.object_source_namespace {
        return Ok(CanonicalObjectResolution::SourceReplaced);
    }
    let Some(active_index) = snapshot.global_config.active_index(identity.region) else {
        return Ok(CanonicalObjectResolution::OutsidePublishedWindow);
    };
    let page = &snapshot.active_cpu_pages[active_index];
    require_page_shape(snapshot, page, active_index)?;
    let mut matched_index = None;
    for (index, local_id) in page.local_ids.iter().copied().enumerate() {
        if local_id == identity.authored_local_id {
            ensure!(
                matched_index.replace(index).is_none(),
                "published canonical object CPU page contains duplicate authored local IDs"
            );
        }
    }
    let index = matched_index.context("published canonical object CPU page is missing local ID")?;
    let object = object_at(snapshot, page, index, identity.authored_local_id)?;
    ensure!(
        object.identity == identity,
        "published canonical object identity diverged from the requested address"
    );
    Ok(CanonicalObjectResolution::Resolved(object))
}

fn query_nearest_snapshot(
    snapshot: &PublishedSnapshot,
    origin: TerrainPosition,
    max_distance_q9: u32,
    excluded_identity: Option<CanonicalObjectIdentity>,
) -> Result<CanonicalObjectNearestQuery> {
    require_snapshot_shape(snapshot)?;
    if let Some(identity) = excluded_identity {
        ensure!(
            identity.source_namespace == snapshot.object_source_namespace,
            "canonical object nearest exclusion belongs to a replaced source"
        );
        ensure!(
            identity.authored_local_id < CANONICAL_OBJECTS_PER_REGION,
            "canonical object nearest exclusion local ID is outside the fixed region capacity"
        );
    }
    ensure!(
        snapshot
            .global_config
            .active_index(origin.region())
            .is_some(),
        "canonical object nearest origin is outside the published active window"
    );
    let mut candidate_count = 0_u32;
    let mut nearest: Option<((u64, i64, i64, u32), CanonicalObjectNearest)> = None;

    for (active_index, page) in snapshot.active_cpu_pages.iter().enumerate() {
        require_page_shape(snapshot, page, active_index)?;
        let mut seen_local_ids = [0_u64; CANONICAL_OBJECTS_PER_REGION as usize / 64];
        for (index, authored_local_id) in page.local_ids.iter().copied().enumerate() {
            ensure!(
                authored_local_id < CANONICAL_OBJECTS_PER_REGION,
                "published canonical object CPU page contains an out-of-range authored local ID"
            );
            let word = authored_local_id as usize / 64;
            let bit = 1_u64 << (authored_local_id % 64);
            ensure!(
                seen_local_ids[word] & bit == 0,
                "published canonical object CPU page contains duplicate authored local IDs"
            );
            seen_local_ids[word] |= bit;

            let object = object_at(snapshot, page, index, authored_local_id)?;
            candidate_count = candidate_count
                .checked_add(1)
                .context("canonical object nearest candidate count overflowed")?;
            if Some(object.identity) == excluded_identity {
                continue;
            }
            let Some(proximity) = object.proximity_from(origin, max_distance_q9)? else {
                continue;
            };
            let candidate = CanonicalObjectNearest {
                object,
                terrain_position: proximity.terrain_position,
                delta_x_q9: proximity.delta_x_q9,
                delta_z_q9: proximity.delta_z_q9,
                distance_squared_q18: proximity.distance_squared_q18,
            };
            let key = (
                candidate.distance_squared_q18,
                object.identity.region.x,
                object.identity.region.z,
                object.identity.authored_local_id,
            );
            if nearest
                .as_ref()
                .is_none_or(|(nearest_key, _)| key < *nearest_key)
            {
                nearest = Some((key, candidate));
            }
        }
    }

    ensure!(
        candidate_count <= CANONICAL_OBJECT_NEAREST_CANDIDATE_CAPACITY,
        "published canonical object CPU snapshot exceeds the nearest-query candidate capacity"
    );
    Ok(CanonicalObjectNearestQuery {
        candidate_count,
        nearest: nearest.map(|(_, candidate)| candidate),
    })
}

fn require_snapshot_shape(snapshot: &PublishedSnapshot) -> Result<()> {
    let expected_count = snapshot.global_config.local_config()?.active_region_count() as usize;
    ensure!(
        snapshot.active_cpu_pages.len() == expected_count,
        "published canonical object CPU snapshot has an inconsistent active count"
    );
    Ok(())
}

fn require_page_shape(
    snapshot: &PublishedSnapshot,
    page: &CpuObjectPage,
    active_index: usize,
) -> Result<()> {
    ensure!(
        snapshot.global_config.active_index(page.global_region) == Some(active_index),
        "published canonical object CPU page has the wrong signed region"
    );
    ensure!(
        page.records.len() == CANONICAL_OBJECTS_PER_REGION as usize
            && page.local_ids.len() == page.records.len()
            && page.presentations.len() == page.records.len(),
        "published canonical object CPU page has inconsistent triple planes"
    );
    Ok(())
}

fn object_at(
    snapshot: &PublishedSnapshot,
    page: &CpuObjectPage,
    index: usize,
    authored_local_id: u32,
) -> Result<CanonicalObject> {
    let record = page.records[index];
    ensure!(
        record.region_id
            == canonical_stable_seed(snapshot.object_stable_seed_namespace, page.global_region),
        "published canonical object CPU record has the wrong stable seed"
    );
    Ok(CanonicalObject {
        identity: CanonicalObjectIdentity {
            source_namespace: snapshot.object_source_namespace,
            region: page.global_region,
            authored_local_id,
        },
        position: record.position,
        height: record.height,
        presentation: page.presentations[index],
    })
}

#[cfg(test)]
#[path = "query_fixture.rs"]
mod test_fixture;

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::test_fixture::{snapshot, snapshot_at, snapshot_with_radius};
    use super::*;
    use crate::async_resident::ObjectSourceNamespace;
    use crate::region::RegionCoord;

    #[test]
    fn resolution_ignores_physical_order() {
        let first = snapshot(769, 73);
        let second = snapshot(641, 419);
        for local_id in [0, 511, 1023] {
            let identity = CanonicalObjectIdentity {
                source_namespace: first.object_source_namespace,
                region: first.global_config.global_center,
                authored_local_id: local_id,
            };
            let expected = resolve_snapshot(&first, identity).unwrap();
            let reordered = resolve_snapshot(&second, identity).unwrap();
            assert_eq!(reordered, expected);
            let CanonicalObjectResolution::Resolved(object) = expected else {
                panic!("current in-window identity did not resolve");
            };
            assert_eq!(object.identity, identity);
        }
    }

    #[test]
    fn typed_lifetime_outcomes() {
        let snapshot = snapshot(769, 73);
        let region = snapshot.global_config.global_center;
        let identity = |source_namespace, region, authored_local_id| CanonicalObjectIdentity {
            source_namespace,
            region,
            authored_local_id,
        };
        assert!(
            resolve_snapshot(
                &snapshot,
                identity(
                    snapshot.object_source_namespace,
                    region,
                    CANONICAL_OBJECTS_PER_REGION,
                ),
            )
            .is_err()
        );
        assert_eq!(
            resolve_snapshot(
                &snapshot,
                identity(
                    snapshot.object_source_namespace,
                    region.checked_offset(1, 0).unwrap(),
                    0,
                ),
            )
            .unwrap(),
            CanonicalObjectResolution::OutsidePublishedWindow,
        );
        assert_eq!(
            resolve_snapshot(
                &snapshot,
                identity(ObjectSourceNamespace::from_bytes([4; 32]), region, 0),
            )
            .unwrap(),
            CanonicalObjectResolution::SourceReplaced,
        );
    }

    #[test]
    fn malformed_pages_fail() {
        let mut mismatched = snapshot(769, 73);
        let region = mismatched.global_config.global_center;
        Arc::get_mut(&mut mismatched.active_cpu_pages[0])
            .unwrap()
            .global_region = region.checked_offset(1, 0).unwrap();
        let identity = CanonicalObjectIdentity {
            source_namespace: mismatched.object_source_namespace,
            region,
            authored_local_id: 0,
        };
        assert!(resolve_snapshot(&mismatched, identity).is_err());

        let mut malformed = snapshot(769, 73);
        Arc::get_mut(&mut malformed.active_cpu_pages[0])
            .unwrap()
            .presentations
            .pop();
        assert!(resolve_snapshot(&malformed, identity).is_err());

        let mut missing = snapshot(769, 73);
        let missing_ids = &mut Arc::get_mut(&mut missing.active_cpu_pages[0])
            .unwrap()
            .local_ids;
        *missing_ids
            .iter_mut()
            .find(|local_id| **local_id == 0)
            .unwrap() = 1;
        assert!(resolve_snapshot(&missing, identity).is_err());

        let mut duplicate = snapshot(769, 73);
        let duplicate_ids = &mut Arc::get_mut(&mut duplicate.active_cpu_pages[0])
            .unwrap()
            .local_ids;
        *duplicate_ids
            .iter_mut()
            .find(|local_id| **local_id == 1)
            .unwrap() = 0;
        assert!(resolve_snapshot(&duplicate, identity).is_err());

        let mut malformed_snapshot = snapshot(769, 73);
        malformed_snapshot.active_cpu_pages.pop();
        let stale = CanonicalObjectIdentity {
            source_namespace: ObjectSourceNamespace::from_bytes([4; 32]),
            ..identity
        };
        assert!(resolve_snapshot(&malformed_snapshot, stale).is_err());
    }

    #[test]
    fn nearest_order_and_seam() {
        let first = snapshot_with_radius(769, 73, 1);
        let second = snapshot_with_radius(641, 419, 1);
        let center = first.global_config.global_center;
        let origin = TerrainPosition::new(center, -4096, -4096).unwrap();
        let expected = query_nearest_snapshot(&first, origin, 0, None).unwrap();
        let reordered = query_nearest_snapshot(&second, origin, 0, None).unwrap();
        assert_eq!(reordered, expected);
        assert_eq!(expected.candidate_count, 9 * CANONICAL_OBJECTS_PER_REGION);
        let nearest = expected.nearest.unwrap();
        assert_eq!(nearest.distance_squared_q18, 0);
        assert_eq!((nearest.delta_x_q9, nearest.delta_z_q9), (0, 0));
        assert_eq!(
            nearest.object.identity.region,
            center.checked_offset(-1, -1).unwrap()
        );
        assert_eq!(nearest.object.identity.authored_local_id, 1023);
        assert_eq!(
            nearest.object.identity.source_namespace,
            first.object_source_namespace
        );
        assert_eq!(nearest.terrain_position, origin);
    }

    #[test]
    fn nearest_radius_and_none() {
        let snapshot = snapshot(769, 73);
        let region = snapshot.global_config.global_center;
        let exact = TerrainPosition::new(region, -4096, -4096).unwrap();
        let exact_query = query_nearest_snapshot(&snapshot, exact, 0, None).unwrap();
        assert_eq!(
            exact_query
                .nearest
                .unwrap()
                .object
                .identity
                .authored_local_id,
            0
        );

        let displaced = exact.translated_q9(1, 0).unwrap();
        assert_eq!(
            query_nearest_snapshot(&snapshot, displaced, 0, None)
                .unwrap()
                .nearest,
            None
        );
        let inclusive = query_nearest_snapshot(&snapshot, displaced, 1, None)
            .unwrap()
            .nearest
            .unwrap();
        assert_eq!(inclusive.object.identity.authored_local_id, 0);
        assert_eq!((inclusive.delta_x_q9, inclusive.delta_z_q9), (-1, 0));
        assert_eq!(inclusive.distance_squared_q18, 1);
        let proximity = inclusive
            .object
            .proximity_from(displaced, 1)
            .unwrap()
            .unwrap();
        assert_eq!(inclusive.terrain_position, proximity.terrain_position);
        assert_eq!(inclusive.delta_x_q9, proximity.delta_x_q9);
        assert_eq!(inclusive.delta_z_q9, proximity.delta_z_q9);
        assert_eq!(
            inclusive.distance_squared_q18,
            proximity.distance_squared_q18
        );
    }

    #[test]
    fn nearest_capacity_and_origin() {
        let snapshot = snapshot_with_radius(769, 73, 2);
        let center = snapshot.global_config.global_center;
        let local_origin = TerrainPosition::new(center, 0, 0).unwrap();
        assert_eq!(
            query_nearest_snapshot(&snapshot, local_origin, 4096, None)
                .unwrap()
                .candidate_count,
            CANONICAL_OBJECT_NEAREST_CANDIDATE_CAPACITY
        );

        let outside = TerrainPosition::new(RegionCoord::new(i64::MIN, i64::MAX), 0, 0).unwrap();
        assert!(query_nearest_snapshot(&snapshot, outside, u32::MAX, None).is_err());

        let edge_center = RegionCoord::new(i64::MIN + 2, i64::MAX - 3);
        let signed_edge = snapshot_at(769, 73, 2, edge_center);
        let edge_origin = TerrainPosition::new(edge_center, 0, 0).unwrap();
        let edge_query = query_nearest_snapshot(&signed_edge, edge_origin, u32::MAX, None).unwrap();
        assert_eq!(
            edge_query.candidate_count,
            CANONICAL_OBJECT_NEAREST_CANDIDATE_CAPACITY
        );
        assert!(edge_query.nearest.is_some());
    }

    #[test]
    fn nearest_malformed_snapshot_rejection() {
        let origin =
            TerrainPosition::new(RegionCoord::new(1_i64 << 40, -(1_i64 << 40)), 0, 0).unwrap();

        let mut incomplete = snapshot_with_radius(769, 73, 1);
        incomplete.active_cpu_pages.pop();
        assert!(query_nearest_snapshot(&incomplete, origin, 4096, None).is_err());

        let mut duplicate = snapshot(769, 73);
        let page = Arc::get_mut(&mut duplicate.active_cpu_pages[0]).unwrap();
        page.local_ids[1] = page.local_ids[0];
        assert!(query_nearest_snapshot(&duplicate, origin, 4096, None).is_err());

        let mut non_lattice = snapshot(769, 73);
        Arc::get_mut(&mut non_lattice.active_cpu_pages[0])
            .unwrap()
            .records[0]
            .position[0] += 1.0 / 1024.0;
        assert!(query_nearest_snapshot(&non_lattice, origin, 4096, None).is_err());
    }

    #[test]
    fn nearest_exact_exclusion() {
        let snapshot = snapshot_with_radius(769, 73, 2);
        let region = snapshot.global_config.global_center;
        let origin = TerrainPosition::new(region, -4096, -4096).unwrap();
        let first = query_nearest_snapshot(&snapshot, origin, u32::MAX, None)
            .unwrap()
            .nearest
            .unwrap();
        let excluded =
            query_nearest_snapshot(&snapshot, origin, u32::MAX, Some(first.object.identity))
                .unwrap();
        assert_eq!(
            excluded.candidate_count,
            CANONICAL_OBJECT_NEAREST_CANDIDATE_CAPACITY
        );
        assert_ne!(
            excluded.nearest.unwrap().object.identity,
            first.object.identity
        );

        let stale = CanonicalObjectIdentity {
            source_namespace: ObjectSourceNamespace::from_bytes([4; 32]),
            ..first.object.identity
        };
        assert!(query_nearest_snapshot(&snapshot, origin, u32::MAX, Some(stale)).is_err());
        let invalid = CanonicalObjectIdentity {
            authored_local_id: CANONICAL_OBJECTS_PER_REGION,
            ..first.object.identity
        };
        assert!(query_nearest_snapshot(&snapshot, origin, u32::MAX, Some(invalid)).is_err());
    }
}
