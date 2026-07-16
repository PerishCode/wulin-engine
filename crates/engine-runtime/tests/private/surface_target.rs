use crate::rendering::meshlet_scene::skeletal::surface::target_probe::pixel_count;

fn pixel(candidate: Option<u32>) -> [u8; 8] {
    let word = candidate.map_or(0, |candidate| candidate + 1);
    let mut bytes = [0; 8];
    bytes[..4].copy_from_slice(&word.to_le_bytes());
    bytes
}

#[test]
fn uses_authored_id() {
    let mut ids = (0..1024).collect::<Vec<_>>();
    ids.swap(7, 91);
    let target = crate::rendering::ObjectTargetFeedback {
        active_index: 1,
        semantic_region: 42,
        authored_local_id: 7,
    };
    let exact_candidate = 1024 + 91;
    let mut visibility = Vec::new();
    for candidate in [
        Some(exact_candidate),
        Some(1024 + 7),
        None,
        Some(exact_candidate),
    ] {
        visibility.extend(pixel(candidate));
    }
    assert_eq!(
        pixel_count(&visibility, Some(target), &[vec![0; 1024], ids]).unwrap(),
        2
    );
    assert_eq!(pixel_count(&visibility, None, &[]).unwrap(), 0);
}
