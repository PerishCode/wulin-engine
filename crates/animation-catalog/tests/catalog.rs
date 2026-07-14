use animation_catalog::{
    BONE_COUNT, CLIP_COUNT, Catalog, FIXTURE_RIG, IMPORTED_RIG, RIG_COUNT, SAMPLE_COUNT,
    unpack_bytes,
};
use meshlet_catalog::Catalog as MeshletCatalog;

#[test]
fn deterministic_catalog() {
    let first = Catalog::build();
    let second = Catalog::build();
    assert_eq!(first, second);
    assert_eq!(first.sha256(), second.sha256());
    assert_eq!(
        first.sha256(),
        "4201d5d51820957df83700e7fbc22631e41b3fa8fca6ec076bf727bc61558f82"
    );
    assert_eq!(
        first.rig_sha256(FIXTURE_RIG),
        "bf4eb3fddf98f18eb191f2d5ed3a4a5b4dcb9efe399f6375d843faf62fee80e8"
    );
    assert_eq!(
        first.rig_sha256(IMPORTED_RIG),
        "1ca9897100f0f1b5909dcc0cb892f827483b87f924dfcd325d516cd5cc645b71"
    );
    first.validate().unwrap();
}

#[test]
fn imported_geometry_uses_cooked_source_bindings() {
    let mesh = MeshletCatalog::build();
    let animation = Catalog::build();
    let start = mesh.imported.vertex_start as usize;
    let end = start + mesh.imported.vertex_count as usize;
    for (binding, source) in animation.skin_bindings[start..end]
        .iter()
        .zip(&mesh.imported_vertex_bindings)
    {
        assert_eq!(binding.indices, source.indices);
        assert_eq!(binding.weights, source.weights);
    }
    assert_eq!(animation.imported.source_joint_count, 24);
    assert_eq!(animation.imported.maximum_joint_depth, 7);
    assert_eq!(
        animation.imported.source_clip_names,
        ["Survey", "Walk", "Run"]
    );
    assert_eq!(animation.imported.source_clip_key_counts, [83, 18, 25]);
    assert_eq!(animation.imported.clip_aliases, [0, 1, 2, 0, 1, 2, 0, 1]);
    assert_eq!(
        animation.imported.cooked_sha256,
        "fea223a83fc8d799c6ef794358f98aa5b524a8a0b7d92a80d9ca4c8fa0429ec1"
    );
}

#[test]
fn hierarchy_is_parent_first() {
    let catalog = Catalog::build();
    assert_eq!(catalog.bones.len(), (RIG_COUNT * BONE_COUNT) as usize);
    assert_eq!(
        catalog.samples.len(),
        (RIG_COUNT * CLIP_COUNT * SAMPLE_COUNT * BONE_COUNT) as usize
    );
    for rig in 0..RIG_COUNT as usize {
        let bones = &catalog.bones[rig * BONE_COUNT as usize..(rig + 1) * BONE_COUNT as usize];
        for (index, bone) in bones.iter().enumerate() {
            if bone.parent == u32::MAX {
                assert_eq!(bone.depth, 0);
            } else {
                assert!((bone.parent as usize) < index);
                assert_eq!(bone.depth, bones[bone.parent as usize].depth + 1);
            }
        }
    }
}

#[test]
fn skin_stream_has_four_normalized_influences() {
    let catalog = Catalog::build();
    for binding in catalog.skin_bindings {
        let indices = unpack_bytes(binding.indices);
        let weights = unpack_bytes(binding.weights);
        assert!(indices.iter().all(|index| u32::from(*index) < BONE_COUNT));
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
    let imported = catalog.evaluate_pose_for_rig(IMPORTED_RIG, 1, 0, 64, 0);
    let imported_next = catalog.evaluate_pose_for_rig(IMPORTED_RIG, 1, 16, 64, 0);
    let imported_variant = catalog.evaluate_pose_for_rig(IMPORTED_RIG, 1, 0, 64, 42);
    let imported_alias = catalog.evaluate_pose_for_rig(IMPORTED_RIG, 4, 0, 64, 0);
    assert_ne!(imported, imported_next);
    assert_eq!(imported, imported_variant);
    assert_eq!(imported, imported_alias);
}
