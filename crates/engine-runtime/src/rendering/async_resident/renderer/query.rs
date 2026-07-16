use anyhow::{Context, Result, ensure};

use crate::async_resident::canonical_stable_seed;
use crate::region::RegionCoord;
use crate::runtime::{CANONICAL_OBJECTS_PER_REGION, CanonicalObject};

use super::{AsyncResidentRenderer, PublishedSnapshot};

impl AsyncResidentRenderer {
    pub(in crate::rendering) fn query_canonical_object(
        &self,
        region: RegionCoord,
        authored_local_id: u32,
    ) -> Result<CanonicalObject> {
        let snapshot = self
            .published
            .as_ref()
            .context("canonical object query requires a published snapshot")?;
        query_snapshot(snapshot, region, authored_local_id)
    }
}

fn query_snapshot(
    snapshot: &PublishedSnapshot,
    region: RegionCoord,
    authored_local_id: u32,
) -> Result<CanonicalObject> {
    ensure!(
        authored_local_id < CANONICAL_OBJECTS_PER_REGION,
        "canonical object authored local ID is outside the fixed region capacity"
    );
    let expected_count = snapshot.global_config.local_config()?.active_region_count() as usize;
    ensure!(
        snapshot.active_cpu_pages.len() == expected_count,
        "published canonical object CPU snapshot has an inconsistent active count"
    );
    let active_index = snapshot
        .global_config
        .active_index(region)
        .context("canonical object query region is outside the published active window")?;
    let page = &snapshot.active_cpu_pages[active_index];
    ensure!(
        page.global_region == region,
        "published canonical object CPU page has the wrong signed region"
    );
    ensure!(
        page.records.len() == CANONICAL_OBJECTS_PER_REGION as usize
            && page.local_ids.len() == page.records.len()
            && page.presentations.len() == page.records.len(),
        "published canonical object CPU page has inconsistent triple planes"
    );
    let mut matched_index = None;
    for (index, local_id) in page.local_ids.iter().copied().enumerate() {
        if local_id == authored_local_id {
            ensure!(
                matched_index.replace(index).is_none(),
                "published canonical object CPU page contains duplicate authored local IDs"
            );
        }
    }
    let index = matched_index.context("published canonical object CPU page is missing local ID")?;
    let record = page.records[index];
    ensure!(
        record.region_id == canonical_stable_seed(snapshot.object_stable_seed_namespace, region),
        "published canonical object CPU record has the wrong stable seed"
    );
    Ok(CanonicalObject {
        region,
        authored_local_id,
        position: record.position,
        height: record.height,
        presentation: page.presentations[index],
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::address::GlobalRegionConfig;
    use crate::async_resident::ObjectSourceNamespace;
    use crate::load::INSTANCES_PER_REGION;
    use crate::rendering::async_resident::transfer::CpuObjectPage;
    use crate::resident::{InstanceRecord, PresentationRecord};

    fn snapshot(order_multiplier: u32, order_offset: u32) -> PublishedSnapshot {
        let far = 1_i64 << 40;
        let global_config = GlobalRegionConfig::new(far, -far, far, -far, 0).unwrap();
        let region = global_config.global_center;
        let stable_seed_namespace = ObjectSourceNamespace::from_bytes([7; 32]);
        let stable_seed = canonical_stable_seed(stable_seed_namespace, region);
        let mut records = Vec::with_capacity(INSTANCES_PER_REGION as usize);
        let mut local_ids = Vec::with_capacity(INSTANCES_PER_REGION as usize);
        let mut presentations = Vec::with_capacity(INSTANCES_PER_REGION as usize);
        for physical in 0..INSTANCES_PER_REGION {
            let local_id = physical
                .wrapping_mul(order_multiplier)
                .wrapping_add(order_offset)
                % INSTANCES_PER_REGION;
            records.push(InstanceRecord {
                position: [local_id as f32 / 8.0, -2.0, -(local_id as f32) / 16.0],
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
        PublishedSnapshot {
            config: global_config.local_config().unwrap(),
            global_config,
            object_source_namespace: ObjectSourceNamespace::from_bytes([3; 32]),
            object_stable_seed_namespace: stable_seed_namespace,
            object_page_checksums: vec![[0; 32]],
            active_slots: vec![17],
            active_cpu_pages: vec![Arc::new(CpuObjectPage {
                global_region: region,
                records,
                local_ids,
                presentations,
            })],
        }
    }

    #[test]
    fn lookup_ignores_physical_order() {
        let first = snapshot(769, 73);
        let second = snapshot(641, 419);
        for local_id in [0, 511, 1023] {
            let expected =
                query_snapshot(&first, first.global_config.global_center, local_id).unwrap();
            let reordered =
                query_snapshot(&second, second.global_config.global_center, local_id).unwrap();
            assert_eq!(reordered, expected);
            assert_eq!(expected.authored_local_id, local_id);
        }
    }

    #[test]
    fn invalid_inputs_fail() {
        let snapshot = snapshot(769, 73);
        let region = snapshot.global_config.global_center;
        assert!(query_snapshot(&snapshot, region, CANONICAL_OBJECTS_PER_REGION).is_err());
        assert!(query_snapshot(&snapshot, region.checked_offset(1, 0).unwrap(), 0,).is_err());
    }

    #[test]
    fn malformed_pages_fail() {
        let mut mismatched = snapshot(769, 73);
        let region = mismatched.global_config.global_center;
        Arc::get_mut(&mut mismatched.active_cpu_pages[0])
            .unwrap()
            .global_region = region.checked_offset(1, 0).unwrap();
        assert!(query_snapshot(&mismatched, region, 0).is_err());

        let mut malformed = snapshot(769, 73);
        Arc::get_mut(&mut malformed.active_cpu_pages[0])
            .unwrap()
            .presentations
            .pop();
        assert!(query_snapshot(&malformed, region, 0).is_err());
    }
}
