use crate::address::GlobalRegionConfig;
use crate::region::RegionCoord;
use crate::rendering::ActorRenderProjection;
use crate::runtime::{ActorHandle, ActorPresentation, RuntimeActor};
use crate::terrain_query::{TerrainBody, TerrainBodyMotion, TerrainPosition};

use super::{ACTOR_VISIBLE_RECORD_BYTES, ActorVisibleCandidate};
use crate::rendering::meshlet_scene::skeletal::resources::ACTOR_CANDIDATE_INDEX;

fn projection(generation: u64) -> ActorRenderProjection {
    let position =
        TerrainPosition::new(RegionCoord::new(2_i64.pow(40), -(2_i64.pow(40))), 0, 0).unwrap();
    let body = TerrainBody::new(position, 196_608, 65_536).unwrap();
    ActorRenderProjection {
        actor: RuntimeActor {
            handle: ActorHandle::new(generation).unwrap(),
            motion: TerrainBodyMotion::new(body, -17),
            presentation: ActorPresentation::animated(7, 63, 32_768, 1, 5, 9),
        },
        global_config: GlobalRegionConfig::new(
            2_i64.pow(40),
            -(2_i64.pow(40)),
            2_i64.pow(40),
            -(2_i64.pow(40)),
            2,
        )
        .unwrap(),
        active_region_index: 12,
        semantic_region: 64 * 128 + 64,
        window_position_q9: [-4_096, 8_192],
        center_height_q16: 196_608,
        half_height_q16: 65_536,
        position_denominator: 512,
        height_denominator: 65_536,
    }
}

#[test]
fn candidate_preserves_projection_and_full_generation() {
    let generation = 0xfedc_ba98_7654_3210;
    let candidate = ActorVisibleCandidate::from_projection(projection(generation)).unwrap();
    assert_eq!(ACTOR_VISIBLE_RECORD_BYTES, 56);
    assert_eq!(candidate.position, [-8.0, 2.0, 16.0]);
    assert_eq!(candidate.height, 2.0);
    assert_eq!(candidate.semantic_region, 64 * 128 + 64);
    assert_eq!(candidate.stable_identity_low, generation as u32);
    assert_eq!(candidate.stable_identity_high, (generation >> 32) as u32);
    assert_eq!(candidate.candidate_index, ACTOR_CANDIDATE_INDEX);
    assert_eq!(candidate.material, 63);
    assert_eq!(candidate.yaw_q16, 32_768);
}

#[test]
fn empty_slot_cannot_be_admitted() {
    assert_eq!(ActorVisibleCandidate::EMPTY.candidate_index, u32::MAX);
    assert_eq!(ActorVisibleCandidate::EMPTY.stable_identity_low, 0);
    assert_eq!(ActorVisibleCandidate::EMPTY.stable_identity_high, 0);
}
