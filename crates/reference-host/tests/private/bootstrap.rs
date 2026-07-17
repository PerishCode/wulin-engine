use super::*;

fn valid_document() -> Vec<u8> {
    br#"{
        "schemaVersion": 2,
        "terrain": "out/cooked/test/terrain.wlt",
        "objects": "out/cooked/test/objects.wlr",
        "globalOrigin": {"x": 1099511627776, "z": -1099511627776},
        "globalCenter": {"x": 1099511627776, "z": -1099511627776},
        "activeRadius": 2,
        "playableRegionBounds": {
            "minimum": {"x": 1099511627775, "z": -1099511627777},
            "maximum": {"x": 1099511627777, "z": -1099511627775}
        }
    }"#
    .to_vec()
}

#[test]
fn arguments_are_strict_and_single_use() {
    let parsed = parse_arguments([
        OsString::from("--bootstrap=out/cooked/test/bootstrap.json"),
        OsString::from("--sidecar-stamp=opaque"),
    ])
    .unwrap();
    assert!(parsed.launched_by_sidecar);
    assert_eq!(
        parsed.bootstrap.unwrap(),
        PathBuf::from("out/cooked/test/bootstrap.json")
    );

    assert!(
        parse_arguments([
            OsString::from("--bootstrap=a"),
            OsString::from("--bootstrap=b")
        ])
        .unwrap_err()
        .to_string()
        .contains("duplicate")
    );
    assert!(parse_arguments([OsString::from("--unknown")]).is_err());
    assert!(parse_arguments([OsString::from("--bootstrap=")]).is_err());
}

#[test]
fn schema_two_document_decodes_exact_paths_signed_config_and_bounds() {
    let bytes = valid_document();
    let plan = Plan::decode(PathBuf::from("out/cooked/test/bootstrap.json"), &bytes).unwrap();
    assert_eq!(
        plan.terrain_path(),
        Path::new("out/cooked/test/terrain.wlt")
    );
    assert_eq!(plan.object_path(), Path::new("out/cooked/test/objects.wlr"));
    assert_eq!(plan.global_config().global_origin.x, 1_i64 << 40);
    assert_eq!(plan.global_config().global_center.z, -(1_i64 << 40));
    assert_eq!(
        plan.playable_region_bounds().minimum(),
        RegionCoord::new((1_i64 << 40) - 1, -(1_i64 << 40) - 1)
    );
    assert_eq!(
        plan.playable_region_bounds().maximum(),
        RegionCoord::new((1_i64 << 40) + 1, -(1_i64 << 40) + 1)
    );
    assert_eq!(plan.pending_json()["configBytes"], bytes.len());
    assert_eq!(
        plan.pending_json()["playableRegionBounds"]["minimum"]["x"],
        (1_i64 << 40) - 1
    );
}

#[test]
fn document_rejects_escaping_path_and_invalid_projection() {
    let path = PathBuf::from("out/cooked/test/bootstrap.json");
    let escaping = String::from_utf8(valid_document())
        .unwrap()
        .replace("out/cooked/test/terrain.wlt", "out/cooked/../terrain.wlt");
    assert!(Plan::decode(path.clone(), escaping.as_bytes()).is_err());

    let projection = String::from_utf8(valid_document()).unwrap().replace(
        "\"globalCenter\": {\"x\": 1099511627776",
        "\"globalCenter\": {\"x\": 9223372036854775807",
    );
    assert!(Plan::decode(path, projection.as_bytes()).is_err());
}

#[test]
fn playable_region_bounds_are_ordered_and_contain_the_initial_center() {
    let path = PathBuf::from("out/cooked/test/bootstrap.json");
    let reversed_x = String::from_utf8(valid_document()).unwrap().replace(
        "\"minimum\": {\"x\": 1099511627775",
        "\"minimum\": {\"x\": 1099511627778",
    );
    assert!(Plan::decode(path.clone(), reversed_x.as_bytes()).is_err());

    let reversed_z = String::from_utf8(valid_document()).unwrap().replace(
        "\"minimum\": {\"x\": 1099511627775, \"z\": -1099511627777}",
        "\"minimum\": {\"x\": 1099511627775, \"z\": -1099511627774}",
    );
    assert!(Plan::decode(path.clone(), reversed_z.as_bytes()).is_err());

    let outside = String::from_utf8(valid_document()).unwrap().replace(
        "\"maximum\": {\"x\": 1099511627777",
        "\"maximum\": {\"x\": 1099511627775",
    );
    assert!(Plan::decode(path, outside.as_bytes()).is_err());
}

#[test]
fn playable_region_bounds_accept_signed_extremes() {
    let bounds = PlayableRegionBounds::new(
        RegionCoord::new(i64::MIN, -7),
        RegionCoord::new(i64::MAX, 11),
    )
    .unwrap();
    assert!(bounds.contains(RegionCoord::new(i64::MIN, -7)));
    assert!(bounds.contains(RegionCoord::new(i64::MAX, 11)));
    assert!(!bounds.contains(RegionCoord::new(0, 12)));
}

#[test]
fn document_size_and_config_path_are_bounded() {
    assert!(
        Plan::decode(
            PathBuf::from("out/cooked/test/bootstrap.json"),
            &vec![b' '; MAX_CONFIG_BYTES + 1]
        )
        .is_err()
    );
    assert!(validate_config_path(Path::new("bootstrap.json")).is_err());
    assert!(validate_config_path(Path::new("out/cooked/test/bootstrap.toml")).is_err());
    assert!(validate_config_path(Path::new("out/cooked/test/bootstrap.json")).is_ok());
}
