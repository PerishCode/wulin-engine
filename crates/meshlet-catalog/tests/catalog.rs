use meshlet_catalog::{
    ARCHETYPE_COUNT, Catalog, IMPORTED_ARCHETYPE, LOD_COUNT, MAX_MESHLET_PRIMITIVES,
    MAX_MESHLET_VERTICES,
};
use sha2::{Digest, Sha256};

#[test]
fn deterministic_catalog() {
    let first = Catalog::build();
    let second = Catalog::build();
    assert_eq!(first, second);
    assert_eq!(first.sha256(), second.sha256());
    assert_eq!(
        first.sha256(),
        "af535c808e73edbfe84e8df316b21b9d07849fd35da6e68ef80fda1ae11bab60"
    );
    first.validate().unwrap();
}

#[test]
fn imported_geometry_is_pinned_and_reducing() {
    let catalog = Catalog::build();
    let imported = &catalog.imported;
    assert_eq!(imported.revision, "cooked-gltf-geometry-v2-skin");
    assert_eq!(
        imported.source_json_sha256,
        "6ddcabf511c0257b87dedf6ac51f1bdb6f21e570eee5fa7c4fa6162d055cb002"
    );
    assert_eq!(
        imported.source_bin_sha256,
        "c7d0d8de28a84d5b25623037f88e063e1502495a2ee6c55f182c61161ad12f80"
    );
    assert_eq!(
        imported.source_texture_sha256,
        "61c8b109ee7f8bf262791933380fafb1465f7b51cbe6472c2d21eff0b31f83a1"
    );
    assert_eq!(
        imported.cooked_sha256,
        "b0eb4940ee63a34e0b64569774ade165b767a458d5806b0239cf90dcf759c077"
    );
    assert_eq!(imported.vertex_count, 434);
    assert_eq!(imported.source_joint_count, 24);
    assert_eq!(imported.maximum_joint_depth, 7);
    assert_eq!(imported.lod_index_counts, [1728, 864, 432]);
    assert_eq!(imported.bounds_min, [-0.15934314, 0.0, -0.9788812]);
    assert_eq!(imported.bounds_max, [0.15934314, 1.0, 0.9788812]);
    assert_eq!(catalog.imported_vertex_bindings.len(), 434);
    assert_eq!(
        hex(Sha256::digest(catalog.imported_vertex_binding_bytes())),
        "de8831585bbb3a13504a049d106258c8819fb990e3908408239b03554baff319"
    );
    let mut nonzero_influence_counts = [0u32; 5];
    for binding in &catalog.imported_vertex_bindings {
        let indices = unpack(binding.indices);
        let weights = unpack(binding.weights);
        assert!(indices.into_iter().all(|joint| joint < 24));
        assert_eq!(weights.into_iter().map(u32::from).sum::<u32>(), 255);
        nonzero_influence_counts[weights.into_iter().filter(|weight| *weight != 0).count()] += 1;
    }
    assert_eq!(nonzero_influence_counts, [0, 202, 218, 12, 2]);
    for lod in 0..LOD_COUNT {
        assert_eq!(
            catalog.lod(IMPORTED_ARCHETYPE, lod).primitive_count * 3,
            imported.lod_index_counts[lod as usize]
        );
    }
}

fn unpack(value: u32) -> [u8; 4] {
    [
        value as u8,
        (value >> 8) as u8,
        (value >> 16) as u8,
        (value >> 24) as u8,
    ]
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn reducing_lods() {
    let catalog = Catalog::build();
    for archetype in 0..ARCHETYPE_COUNT {
        let fine = catalog.lod(archetype, 0);
        let medium = catalog.lod(archetype, 1);
        let coarse = catalog.lod(archetype, 2);
        assert!(fine.vertex_count > medium.vertex_count);
        assert!(medium.vertex_count > coarse.vertex_count);
        assert!(fine.primitive_count > medium.primitive_count);
        assert!(medium.primitive_count > coarse.primitive_count);
    }
    assert_eq!(catalog.lods.len(), (ARCHETYPE_COUNT * LOD_COUNT) as usize);
}

#[test]
fn shader_limits() {
    let catalog = Catalog::build();
    assert!(catalog.meshlets.iter().all(|meshlet| {
        meshlet.vertex_count <= MAX_MESHLET_VERTICES
            && meshlet.primitive_count <= MAX_MESHLET_PRIMITIVES
    }));
}
