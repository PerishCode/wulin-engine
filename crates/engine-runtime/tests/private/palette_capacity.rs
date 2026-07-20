use std::mem::size_of;

use animation_catalog::{Affine, BONE_COUNT, MAX_POSE_KEYS};

use super::{MAX_SHARED_POSES, PALETTE_BYTES, PALETTE_ELEMENT_COUNT, SKELETAL_CANDIDATE_CAPACITY};

#[test]
fn palette_capacity_is_the_complete_shared_pose_domain() {
    assert_eq!(MAX_SHARED_POSES, MAX_POSE_KEYS);
    assert_eq!(MAX_SHARED_POSES, 1_024);
    assert_eq!(PALETTE_ELEMENT_COUNT, MAX_SHARED_POSES * BONE_COUNT);
    assert_eq!(PALETTE_ELEMENT_COUNT, 131_072);
    assert_eq!(size_of::<Affine>(), 48);
    assert_eq!(PALETTE_BYTES, 6_291_456);
}

#[test]
fn palette_capacity_is_independent_from_candidate_capacity() {
    assert_eq!(SKELETAL_CANDIDATE_CAPACITY, 25_601);
    assert_eq!(
        SKELETAL_CANDIDATE_CAPACITY as u64 * BONE_COUNT as u64 * size_of::<Affine>() as u64
            - PALETTE_BYTES,
        151_001_088
    );
}
