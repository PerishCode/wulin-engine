use engine_runtime::{
    CanonicalObject, CanonicalObjectIdentity, CanonicalObjectPresentation, ObjectSourceNamespace,
    RegionCoord,
};

fn object(region: RegionCoord, x: f32, z: f32) -> CanonicalObject {
    CanonicalObject {
        identity: CanonicalObjectIdentity {
            source_namespace: ObjectSourceNamespace::from_bytes([1; 32]),
            region,
            authored_local_id: 73,
        },
        position: [x, 0.0, z],
        height: 1.0,
        presentation: CanonicalObjectPresentation::static_object(0, 0, 0),
    }
}

#[test]
fn exact_lattice_positions_preserve_owner_region() {
    let far = RegionCoord::new(1_i64 << 40, -(1_i64 << 40));
    for (x_q9, z_q9) in [(-4096, -4096), (-2049, 3073), (0, 0), (4095, 4095)] {
        let position = object(far, x_q9 as f32 / 512.0, z_q9 as f32 / 512.0)
            .terrain_position()
            .unwrap();
        assert_eq!(position.region(), far);
        assert_eq!(position.local_x_q9(), x_q9);
        assert_eq!(position.local_z_q9(), z_q9);
    }
}

#[test]
fn closed_positive_edges_normalize_to_half_open_regions() {
    let owner = RegionCoord::new(-17, 23);
    for (x, z, region, local) in [
        (-8.0, -8.0, owner, [-4096, -4096]),
        (
            8.0,
            -8.0,
            owner.checked_offset(1, 0).unwrap(),
            [-4096, -4096],
        ),
        (
            -8.0,
            8.0,
            owner.checked_offset(0, 1).unwrap(),
            [-4096, -4096],
        ),
        (
            8.0,
            8.0,
            owner.checked_offset(1, 1).unwrap(),
            [-4096, -4096],
        ),
    ] {
        let position = object(owner, x, z).terrain_position().unwrap();
        assert_eq!(position.region(), region);
        assert_eq!([position.local_x_q9(), position.local_z_q9()], local);
    }
}

#[test]
fn invalid_authored_coordinates_fail_without_coercion() {
    for (x, z) in [
        (f32::NAN, 0.0),
        (0.0, f32::INFINITY),
        (-8.001953, 0.0),
        (0.0, 8.001953),
        (1.0 / 1024.0, 0.0),
        (0.0, -3.0 / 1024.0),
    ] {
        assert!(object(RegionCoord::ZERO, x, z).terrain_position().is_err());
    }
}

#[test]
fn positive_edge_signed_region_overflow_fails() {
    assert!(
        object(RegionCoord::new(i64::MAX, 0), 8.0, 0.0)
            .terrain_position()
            .is_err()
    );
    assert!(
        object(RegionCoord::new(0, i64::MAX), 0.0, 8.0)
            .terrain_position()
            .is_err()
    );
}
