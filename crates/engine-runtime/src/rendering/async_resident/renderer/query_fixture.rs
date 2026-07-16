use std::sync::Arc;

use super::{PublishedSnapshot, canonical_stable_seed};
use crate::address::GlobalRegionConfig;
use crate::async_resident::ObjectSourceNamespace;
use crate::load::INSTANCES_PER_REGION;
use crate::region::RegionCoord;
use crate::rendering::async_resident::transfer::CpuObjectPage;
use crate::resident::{InstanceRecord, PresentationRecord};

pub(super) fn snapshot(order_multiplier: u32, order_offset: u32) -> PublishedSnapshot {
    snapshot_with_radius(order_multiplier, order_offset, 0)
}

pub(super) fn snapshot_with_radius(
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

pub(super) fn snapshot_at(
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
