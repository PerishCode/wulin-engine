use anyhow::{Context, Result, ensure};

use crate::async_resident::canonical_stable_seed;
use crate::runtime::{
    CANONICAL_OBJECT_NEAREST_CANDIDATE_CAPACITY, CANONICAL_OBJECTS_PER_REGION, CanonicalObject,
    CanonicalObjectIdentity, CanonicalObjectNearest, CanonicalObjectNearestQuery,
};
use crate::terrain_query::{TERRAIN_POSITION_REGION_SIDE_Q9, TerrainPosition};

use super::{AsyncResidentRenderer, PublishedSnapshot};
use crate::rendering::async_resident::transfer::CpuObjectPage;

impl AsyncResidentRenderer {
    pub(in crate::rendering) fn query_canonical_object(
        &self,
        identity: CanonicalObjectIdentity,
    ) -> Result<CanonicalObject> {
        let snapshot = self
            .published
            .as_ref()
            .context("canonical object query requires a published snapshot")?;
        query_snapshot(snapshot, identity)
    }

    pub(in crate::rendering) fn query_nearest_canonical_object(
        &self,
        origin: TerrainPosition,
        max_distance_q9: u32,
    ) -> Result<CanonicalObjectNearestQuery> {
        let snapshot = self
            .published
            .as_ref()
            .context("canonical object nearest query requires a published snapshot")?;
        query_nearest_snapshot(snapshot, origin, max_distance_q9)
    }
}

fn query_snapshot(
    snapshot: &PublishedSnapshot,
    identity: CanonicalObjectIdentity,
) -> Result<CanonicalObject> {
    ensure!(
        identity.source_namespace == snapshot.object_source_namespace,
        "canonical object identity source does not match the published snapshot"
    );
    require_snapshot_shape(snapshot)?;
    ensure!(
        identity.authored_local_id < CANONICAL_OBJECTS_PER_REGION,
        "canonical object authored local ID is outside the fixed region capacity"
    );
    let active_index = snapshot
        .global_config
        .active_index(identity.region)
        .context("canonical object query region is outside the published active window")?;
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
    Ok(object)
}

fn query_nearest_snapshot(
    snapshot: &PublishedSnapshot,
    origin: TerrainPosition,
    max_distance_q9: u32,
) -> Result<CanonicalObjectNearestQuery> {
    require_snapshot_shape(snapshot)?;
    ensure!(
        snapshot
            .global_config
            .active_index(origin.region())
            .is_some(),
        "canonical object nearest origin is outside the published active window"
    );
    let radius = i128::from(max_distance_q9);
    let radius_squared = u128::from(max_distance_q9) * u128::from(max_distance_q9);
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
            let terrain_position = object.terrain_position()?;
            let (delta_x_q9, delta_z_q9) = planar_delta_q9(origin, terrain_position);
            if !(-radius..=radius).contains(&delta_x_q9)
                || !(-radius..=radius).contains(&delta_z_q9)
            {
                continue;
            }
            let distance_squared =
                u128::try_from(delta_x_q9 * delta_x_q9 + delta_z_q9 * delta_z_q9)
                    .expect("radius-bounded squared distance must be nonnegative");
            if distance_squared > radius_squared {
                continue;
            }
            let candidate = CanonicalObjectNearest {
                object,
                terrain_position,
                delta_x_q9: i64::try_from(delta_x_q9)
                    .expect("radius-bounded X delta must fit signed 64-bit"),
                delta_z_q9: i64::try_from(delta_z_q9)
                    .expect("radius-bounded Z delta must fit signed 64-bit"),
                distance_squared_q18: u64::try_from(distance_squared)
                    .expect("radius-bounded squared distance must fit unsigned 64-bit"),
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

fn planar_delta_q9(origin: TerrainPosition, candidate: TerrainPosition) -> (i128, i128) {
    let region_side = i128::from(TERRAIN_POSITION_REGION_SIDE_Q9);
    let delta_x = (i128::from(candidate.region().x) - i128::from(origin.region().x)) * region_side
        + i128::from(candidate.local_x_q9() - origin.local_x_q9());
    let delta_z = (i128::from(candidate.region().z) - i128::from(origin.region().z)) * region_side
        + i128::from(candidate.local_z_q9() - origin.local_z_q9());
    (delta_x, delta_z)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::address::GlobalRegionConfig;
    use crate::async_resident::ObjectSourceNamespace;
    use crate::load::INSTANCES_PER_REGION;
    use crate::region::RegionCoord;
    use crate::rendering::async_resident::transfer::CpuObjectPage;
    use crate::resident::{InstanceRecord, PresentationRecord};

    fn snapshot(order_multiplier: u32, order_offset: u32) -> PublishedSnapshot {
        snapshot_with_radius(order_multiplier, order_offset, 0)
    }

    fn snapshot_with_radius(
        order_multiplier: u32,
        order_offset: u32,
        active_radius: u32,
    ) -> PublishedSnapshot {
        let far = 1_i64 << 40;
        snapshot_at(
            order_multiplier,
            order_offset,
            active_radius,
            RegionCoord::new(far, -far),
        )
    }

    fn snapshot_at(
        order_multiplier: u32,
        order_offset: u32,
        active_radius: u32,
        center: RegionCoord,
    ) -> PublishedSnapshot {
        let global_config =
            GlobalRegionConfig::new(center.x, center.z, center.x, center.z, active_radius).unwrap();
        let stable_seed_namespace = ObjectSourceNamespace::from_bytes([7; 32]);
        let mut active_cpu_pages = Vec::new();
        for addressed in global_config.addressed_regions().unwrap() {
            let stable_seed = canonical_stable_seed(stable_seed_namespace, addressed.global_region);
            let mut records = Vec::with_capacity(INSTANCES_PER_REGION as usize);
            let mut local_ids = Vec::with_capacity(INSTANCES_PER_REGION as usize);
            let mut presentations = Vec::with_capacity(INSTANCES_PER_REGION as usize);
            for physical in 0..INSTANCES_PER_REGION {
                let local_id = physical
                    .wrapping_mul(order_multiplier)
                    .wrapping_add(order_offset)
                    % INSTANCES_PER_REGION;
                let local_x = local_id % 32;
                let local_z = local_id / 32;
                let local_q9 = |axis| match axis {
                    31 => 4096,
                    value => -4096 + i32::try_from(value * 256).unwrap(),
                };
                records.push(InstanceRecord {
                    position: [
                        local_q9(local_x) as f32 / 512.0,
                        -2.0,
                        local_q9(local_z) as f32 / 512.0,
                    ],
                    height: local_id as f32 / 32.0,
                    region_id: stable_seed,
                });
                local_ids.push(local_id);
                presentations.push(PresentationRecord::static_object(
                    local_id % 8,
                    local_id % 64,
                    local_id & 0xffff,
                ));
            }
            active_cpu_pages.push(Arc::new(CpuObjectPage {
                global_region: addressed.global_region,
                records,
                local_ids,
                presentations,
            }));
        }
        let active_count = global_config.local_config().unwrap().active_region_count() as usize;
        PublishedSnapshot {
            config: global_config.local_config().unwrap(),
            global_config,
            object_source_namespace: ObjectSourceNamespace::from_bytes([3; 32]),
            object_stable_seed_namespace: stable_seed_namespace,
            object_page_checksums: vec![[0; 32]; active_count],
            active_slots: (0..u32::try_from(active_count).unwrap()).collect(),
            active_cpu_pages,
        }
    }

    #[test]
    fn lookup_ignores_physical_order() {
        let first = snapshot(769, 73);
        let second = snapshot(641, 419);
        for local_id in [0, 511, 1023] {
            let identity = CanonicalObjectIdentity {
                source_namespace: first.object_source_namespace,
                region: first.global_config.global_center,
                authored_local_id: local_id,
            };
            let expected = query_snapshot(&first, identity).unwrap();
            let reordered = query_snapshot(&second, identity).unwrap();
            assert_eq!(reordered, expected);
            assert_eq!(expected.identity, identity);
        }
    }

    #[test]
    fn invalid_inputs_fail() {
        let snapshot = snapshot(769, 73);
        let region = snapshot.global_config.global_center;
        let identity = |source_namespace, region, authored_local_id| CanonicalObjectIdentity {
            source_namespace,
            region,
            authored_local_id,
        };
        assert!(
            query_snapshot(
                &snapshot,
                identity(
                    snapshot.object_source_namespace,
                    region,
                    CANONICAL_OBJECTS_PER_REGION,
                ),
            )
            .is_err()
        );
        assert!(
            query_snapshot(
                &snapshot,
                identity(
                    snapshot.object_source_namespace,
                    region.checked_offset(1, 0).unwrap(),
                    0,
                ),
            )
            .is_err()
        );
        assert!(
            query_snapshot(
                &snapshot,
                identity(ObjectSourceNamespace::from_bytes([4; 32]), region, 0),
            )
            .is_err()
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
        assert!(query_snapshot(&mismatched, identity).is_err());

        let mut malformed = snapshot(769, 73);
        Arc::get_mut(&mut malformed.active_cpu_pages[0])
            .unwrap()
            .presentations
            .pop();
        assert!(query_snapshot(&malformed, identity).is_err());
    }

    #[test]
    fn nearest_order_and_seam() {
        let first = snapshot_with_radius(769, 73, 1);
        let second = snapshot_with_radius(641, 419, 1);
        let center = first.global_config.global_center;
        let origin = TerrainPosition::new(center, -4096, -4096).unwrap();
        let expected = query_nearest_snapshot(&first, origin, 0).unwrap();
        let reordered = query_nearest_snapshot(&second, origin, 0).unwrap();
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
        let exact_query = query_nearest_snapshot(&snapshot, exact, 0).unwrap();
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
            query_nearest_snapshot(&snapshot, displaced, 0)
                .unwrap()
                .nearest,
            None
        );
        let inclusive = query_nearest_snapshot(&snapshot, displaced, 1)
            .unwrap()
            .nearest
            .unwrap();
        assert_eq!(inclusive.object.identity.authored_local_id, 0);
        assert_eq!((inclusive.delta_x_q9, inclusive.delta_z_q9), (-1, 0));
        assert_eq!(inclusive.distance_squared_q18, 1);
    }

    #[test]
    fn nearest_capacity_and_origin() {
        let snapshot = snapshot_with_radius(769, 73, 2);
        let center = snapshot.global_config.global_center;
        let local_origin = TerrainPosition::new(center, 0, 0).unwrap();
        assert_eq!(
            query_nearest_snapshot(&snapshot, local_origin, 4096)
                .unwrap()
                .candidate_count,
            CANONICAL_OBJECT_NEAREST_CANDIDATE_CAPACITY
        );

        let outside = TerrainPosition::new(RegionCoord::new(i64::MIN, i64::MAX), 0, 0).unwrap();
        assert!(query_nearest_snapshot(&snapshot, outside, u32::MAX).is_err());

        let edge_center = RegionCoord::new(i64::MIN + 2, i64::MAX - 3);
        let signed_edge = snapshot_at(769, 73, 2, edge_center);
        let edge_origin = TerrainPosition::new(edge_center, 0, 0).unwrap();
        let edge_query = query_nearest_snapshot(&signed_edge, edge_origin, u32::MAX).unwrap();
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
        assert!(query_nearest_snapshot(&incomplete, origin, 4096).is_err());

        let mut duplicate = snapshot(769, 73);
        let page = Arc::get_mut(&mut duplicate.active_cpu_pages[0]).unwrap();
        page.local_ids[1] = page.local_ids[0];
        assert!(query_nearest_snapshot(&duplicate, origin, 4096).is_err());

        let mut non_lattice = snapshot(769, 73);
        Arc::get_mut(&mut non_lattice.active_cpu_pages[0])
            .unwrap()
            .records[0]
            .position[0] += 1.0 / 1024.0;
        assert!(query_nearest_snapshot(&non_lattice, origin, 4096).is_err());
    }
}
