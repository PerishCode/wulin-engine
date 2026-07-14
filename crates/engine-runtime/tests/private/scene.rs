use super::*;

#[test]
fn camera_relative_clip_matches() {
    let scene = SceneState::new();
    let local = scene.view_projection(1280.0 / 720.0) * OBJECTS[0].model_matrix();
    let relative = scene.calibration_view_projection(1280.0 / 720.0).unwrap()
        * scene.calibration_model_matrix(&OBJECTS[0]).unwrap();
    for point in CLIP_PROBE_POINTS {
        let expected = local * point;
        let actual = relative * point;
        assert!((expected - actual).abs().max_element() <= 0.0001);
    }
}
