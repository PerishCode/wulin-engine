use animation_catalog::{BONE_COUNT, CLIP_COUNT, Catalog, SAMPLE_COUNT, unpack_bytes};

#[test]
fn deterministic_catalog() {
    let first = Catalog::build();
    let second = Catalog::build();
    assert_eq!(first, second);
    assert_eq!(first.sha256(), second.sha256());
    assert_eq!(
        first.sha256(),
        "cc075037175990f29083ad1fc63823c1a77002d7aeccfbc429eee4f54de22a6e"
    );
    first.validate().unwrap();
}

#[test]
fn hierarchy_is_parent_first() {
    let catalog = Catalog::build();
    assert_eq!(catalog.bones.len(), BONE_COUNT as usize);
    assert_eq!(
        catalog.samples.len(),
        (CLIP_COUNT * SAMPLE_COUNT * BONE_COUNT) as usize
    );
    for (index, bone) in catalog.bones.iter().enumerate().skip(1) {
        assert!((bone.parent as usize) < index);
        assert_eq!(bone.depth, catalog.bones[bone.parent as usize].depth + 1);
    }
}

#[test]
fn skin_stream_has_four_normalized_influences() {
    let catalog = Catalog::build();
    for binding in catalog.skin_bindings {
        let indices = unpack_bytes(binding.indices);
        let weights = unpack_bytes(binding.weights);
        assert!(indices.iter().all(|index| u32::from(*index) < BONE_COUNT));
        assert!(weights.iter().all(|weight| *weight > 0));
        assert_eq!(
            weights.iter().map(|weight| u32::from(*weight)).sum::<u32>(),
            255
        );
    }
}

#[test]
fn pose_evaluation_is_deterministic_and_variant_sensitive() {
    let catalog = Catalog::build();
    let shared = catalog.evaluate_pose(3, 17, 64, 0);
    let repeated = catalog.evaluate_pose(3, 17, 64, 0);
    let unique = catalog.evaluate_pose(3, 17, 64, 42);
    assert_eq!(shared, repeated);
    assert_ne!(shared, unique);
}
