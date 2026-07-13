use super::*;

fn camera(x: f32, z: f32) -> Camera {
    Camera {
        position: [x, 5.0, z],
        target: [x, 0.0, z - 1.0],
        vertical_fov_degrees: 60.0,
        near_plane_meters: 0.1,
    }
}

#[test]
fn far_target_is_exact() {
    let far = 1_i64 << 40;
    let global = GlobalRegionConfig::new(far, -far, far, -far, 2).unwrap();
    let published = TraversalTarget {
        config: global.local_config().unwrap(),
        global_config: Some(global),
    };
    let basis = TraversalBasis::new(published, false).unwrap();
    let local = map_camera(camera(16.0, -16.0), &basis).unwrap();
    let target = basis.target(local).unwrap();
    assert_eq!((local.active_center_x, local.active_center_z), (65, 63));
    let mapped = target.global_config.unwrap();
    assert_eq!(mapped.global_origin, RegionCoord::new(far, -far));
    assert_eq!(mapped.global_center, RegionCoord::new(far + 1, -far - 1));
}

#[test]
fn legacy_status_is_unchanged() {
    let config = LoadConfig::new(128, 64, 64, 2).unwrap();
    let target = TraversalTarget {
        config,
        global_config: None,
    };
    assert_eq!(target.status_json(), json!(config));
    let basis = TraversalBasis::new(target, false).unwrap();
    assert_eq!(json!(basis).get("globalOrigin"), None);
}

#[test]
fn basis_rejects_extent_overflow() {
    let global = GlobalRegionConfig::new(i64::MAX, 0, i64::MAX, 0, 2).unwrap();
    let published = TraversalTarget {
        config: global.local_config().unwrap(),
        global_config: Some(global),
    };
    assert!(TraversalBasis::new(published, false).is_err());
}

#[test]
fn rollover_preserves_center() {
    let far = 1_i64 << 40;
    let global = GlobalRegionConfig::new(far - 33, -far, far, -far, 2).unwrap();
    let published = TraversalTarget {
        config: global.local_config().unwrap(),
        global_config: Some(global),
    };
    let basis = TraversalBasis::new(published, true).unwrap();
    let local = map_camera(camera(528.0, 0.0), &basis).unwrap();
    let target = basis.camera_target(local).unwrap();
    assert_eq!(
        (target.config.active_center_x, target.config.active_center_z),
        (64, 64)
    );
    let target_global = target.global_config.unwrap();
    assert_eq!(target_global.global_origin, RegionCoord::new(far, -far));
    assert_eq!(target_global.global_center, RegionCoord::new(far, -far));
}

#[test]
fn rollover_shifts_at_commit() {
    let far = 1_i64 << 40;
    let global = GlobalRegionConfig::new(far - 33, -far, far, -far, 2).unwrap();
    let published = TraversalTarget {
        config: global.local_config().unwrap(),
        global_config: Some(global),
    };
    let mut traversal = CameraTraversal::default();
    traversal.enable(published, true).unwrap();
    let target = traversal
        .plan(camera(528.0, 0.0), published, None)
        .unwrap()
        .unwrap();
    traversal
        .mark_published(7, target.config, target.global_config)
        .unwrap();
    assert_eq!(traversal.take_camera_delta(), Some([-33, 0]));
    assert_eq!(
        traversal.basis.unwrap().global_origin,
        Some(RegionCoord::new(far, -far))
    );
    assert_eq!(traversal.rollover.status_json().unwrap()["count"], 1);
}
