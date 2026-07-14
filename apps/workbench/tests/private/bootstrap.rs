use super::*;

fn valid_document() -> Vec<u8> {
    br#"{
        "schemaVersion": 1,
        "terrain": "out/cooked/test/terrain.wlt",
        "objects": "out/cooked/test/objects.wlr",
        "globalOrigin": {"x": 1099511627776, "z": -1099511627776},
        "globalCenter": {"x": 1099511627776, "z": -1099511627776},
        "activeRadius": 2
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
fn schema_one_document_decodes_exact_paths_and_signed_config() {
    let bytes = valid_document();
    let plan = Plan::decode(PathBuf::from("out/cooked/test/bootstrap.json"), &bytes).unwrap();
    assert_eq!(
        plan.terrain_path(),
        Path::new("out/cooked/test/terrain.wlt")
    );
    assert_eq!(plan.object_path(), Path::new("out/cooked/test/objects.wlr"));
    assert_eq!(plan.global_config().global_origin.x, 1_i64 << 40);
    assert_eq!(plan.global_config().global_center.z, -(1_i64 << 40));
    assert_eq!(plan.pending_json()["configBytes"], bytes.len());
}

#[test]
fn document_rejects_unknown_schema_path_and_projection() {
    let path = PathBuf::from("out/cooked/test/bootstrap.json");
    let unknown = String::from_utf8(valid_document()).unwrap().replace(
        "\"activeRadius\": 2",
        "\"activeRadius\": 2, \"fallback\": true",
    );
    assert!(Plan::decode(path.clone(), unknown.as_bytes()).is_err());

    let schema = String::from_utf8(valid_document())
        .unwrap()
        .replace("\"schemaVersion\": 1", "\"schemaVersion\": 2");
    assert!(Plan::decode(path.clone(), schema.as_bytes()).is_err());

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
