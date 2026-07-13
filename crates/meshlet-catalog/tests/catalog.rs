use meshlet_catalog::{
    ARCHETYPE_COUNT, Catalog, LOD_COUNT, MAX_MESHLET_PRIMITIVES, MAX_MESHLET_VERTICES,
};

#[test]
fn deterministic_catalog() {
    let first = Catalog::build();
    let second = Catalog::build();
    assert_eq!(first, second);
    assert_eq!(first.sha256(), second.sha256());
    assert_eq!(
        first.sha256(),
        "9553748209f9de17e9b524b1c21080404f32df57be62959714b58db1121f0a4e"
    );
    first.validate().unwrap();
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
