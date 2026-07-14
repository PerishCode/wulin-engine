use meshlet_catalog::{
    ARCHETYPE_COUNT, Catalog, IMPORTED_ARCHETYPE, LOD_COUNT, MAX_MESHLET_PRIMITIVES,
    MAX_MESHLET_VERTICES,
};

#[test]
fn deterministic_catalog() {
    let first = Catalog::build();
    let second = Catalog::build();
    assert_eq!(first, second);
    assert_eq!(first.sha256(), second.sha256());
    assert_eq!(
        first.sha256(),
        "e24aaf210a746aa281232e3bf1e2c26222cf5134b22224305ecf92189937c736"
    );
    first.validate().unwrap();
}

#[test]
fn imported_geometry_is_pinned_and_reducing() {
    let catalog = Catalog::build();
    let imported = &catalog.imported;
    assert_eq!(imported.revision, "cooked-gltf-geometry-v1");
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
        "a4dc1dbf84171f62cc61e522d43068f67d013467d3bdb591c733fab666f06124"
    );
    assert_eq!(imported.vertex_count, 434);
    assert_eq!(imported.lod_index_counts, [1728, 864, 432]);
    assert_eq!(imported.bounds_min, [-0.15934314, 0.0, -0.9788812]);
    assert_eq!(imported.bounds_max, [0.15934314, 1.0, 0.9788812]);
    for lod in 0..LOD_COUNT {
        assert_eq!(
            catalog.lod(IMPORTED_ARCHETYPE, lod).primitive_count * 3,
            imported.lod_index_counts[lod as usize]
        );
    }
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
