use meshlet_catalog::Catalog as MeshletCatalog;
use sha2::{Digest, Sha256};
use surface_catalog::{
    Catalog, IMPORTED_MATERIAL, MATERIAL_COUNT, MIP_COUNT, TEXTURE_SIDE, decode_octahedral,
};

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
        "4267365c1d71e96beaff2ece04d6a94c450fd86131ebe65c2447c7b95cb8c15d"
    );
    assert_eq!(first.gpu_bytes(), 1_500_416);
}

#[test]
fn imported_material_is_pinned_and_fixture_layers_are_stable() {
    let mesh = MeshletCatalog::build();
    let surface = Catalog::build(&mesh);
    let imported = &surface.imported_material;
    assert_eq!(imported.revision, "cooked-gltf-material-v1");
    assert_eq!(
        imported.source_json_sha256,
        "6ddcabf511c0257b87dedf6ac51f1bdb6f21e570eee5fa7c4fa6162d055cb002"
    );
    assert_eq!(
        imported.source_texture_sha256,
        "61c8b109ee7f8bf262791933380fafb1465f7b51cbe6472c2d21eff0b31f83a1"
    );
    assert_eq!(
        imported.cooked_sha256,
        "5c18b4a6c9f13f79d5b6714ece3d0ef3e4ee20c181b1a169c7eb6a8392e41f0c"
    );
    assert_eq!(imported.material_index, IMPORTED_MATERIAL);
    assert_eq!(imported.texture_layer, IMPORTED_MATERIAL);
    assert_eq!(imported.source_size, [1024, 1024]);
    assert_eq!(imported.texture_side, TEXTURE_SIDE);
    assert_eq!(imported.mip_sizes, [16384, 4096, 1024, 256, 64, 16, 4]);
    assert_eq!(
        imported.mip_sha256,
        [
            "81ebf8c59467fbcd556a006a64561ead857e4cefe46cb04535e6dc71440a6b44",
            "c131406c3d871b9fa461ec2b4d89f9eec479c9e34a3dd9da85fb34c8d034ddb1",
            "07e08e1c4d1c1a34c95f28f68deb5358d68afd0bf053b67608e7f9a848b8d5d9",
            "5a33b2db4c74015e2948c1fc55c3dc77fa2b8aad2a5cf54ed8d3919eb0ab258b",
            "4827229d464587a2b9ab13e307f854cc006eca4cba700b22ba3cd4bf9f1a653d",
            "c1a3ac1fd3143f239f757d248e231dad942502d41f2c1cdb3e628d06741810f2",
            "b551cde7bc5b2be81cbf0aa54c74b1382484d119a6e5c39ab7255b8b3d908a4e",
        ]
        .map(str::to_owned)
    );
    let material = surface.materials[IMPORTED_MATERIAL as usize];
    assert_eq!(material.base_color, [1.0; 4]);
    assert_eq!(material.roughness, 0.58);
    assert_eq!(material.metallic, 0.0);

    let mut fixture_bytes = Vec::new();
    for (mip, bytes) in surface.texture_mips.iter().enumerate() {
        let side = (TEXTURE_SIDE >> mip).max(1) as usize;
        fixture_bytes.extend_from_slice(&bytes[..IMPORTED_MATERIAL as usize * side * side * 4]);
    }
    assert_eq!(
        format!("{:x}", Sha256::digest(fixture_bytes)),
        "3f6256268867bf270268e7478145e12ab7e3216612b550cd1a412bf440357c8b"
    );
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
