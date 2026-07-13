use meshlet_catalog::Catalog as MeshletCatalog;
use surface_catalog::{Catalog, MATERIAL_COUNT, MIP_COUNT, decode_octahedral};

#[test]
fn deterministic_catalog() {
    let mesh = MeshletCatalog::build();
    let first = Catalog::build(&mesh);
    let second = Catalog::build(&mesh);
    assert_eq!(first, second);
    assert_eq!(first.sha256(), second.sha256());
    assert_eq!(first.encoded_bytes(), second.encoded_bytes());
    assert_eq!(
        first.sha256(),
        "e9715635b9e9f2a7dd0089c35db3cb3ccd6ae87fc2119cc548ed2f37a4996989"
    );
    assert_eq!(first.gpu_bytes(), 1_488_384);
}

#[test]
fn topology_expansion_is_complete() {
    let mesh = MeshletCatalog::build();
    let surface = Catalog::build(&mesh);
    assert_eq!(surface.vertices.len(), mesh.vertices.len());
    assert_eq!(surface.primitives.len(), mesh.primitives.len());
    surface.validate(&mesh).unwrap();
}

#[test]
fn octahedral_normals_decode_to_unit_vectors() {
    let mesh = MeshletCatalog::build();
    let surface = Catalog::build(&mesh);
    for vertex in surface.vertices {
        let normal = decode_octahedral([vertex.oct_normal_uv[0], vertex.oct_normal_uv[1]]);
        let length = normal
            .into_iter()
            .map(|value| value * value)
            .sum::<f32>()
            .sqrt();
        assert!((length - 1.0).abs() < 0.00001);
    }
}

#[test]
fn material_texture_shape_is_canonical() {
    let mesh = MeshletCatalog::build();
    let surface = Catalog::build(&mesh);
    assert_eq!(surface.materials.len(), MATERIAL_COUNT as usize);
    assert_eq!(surface.texture_mips.len(), MIP_COUNT as usize);
    assert!(
        surface
            .texture_mips
            .windows(2)
            .all(|mips| mips[0].len() == mips[1].len() * 4)
    );
}
