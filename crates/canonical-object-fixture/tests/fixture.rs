use canonical_object_fixture::generate_region;
use region_format::{GlobalRegion, RECORDS_PER_REGION};

#[test]
fn deterministic_region_local() {
    let far = 1_i64 << 40;
    let region = GlobalRegion::new(far, -far);
    let first = generate_region(region);
    let second = generate_region(region);
    assert_eq!(first, second);
    assert_eq!(first.len(), RECORDS_PER_REGION as usize);
    assert!(first.iter().all(|record| {
        (-8.0..=8.0).contains(&record.position[0])
            && (-8.0..=8.0).contains(&record.position[2])
            && record.region_id == first[0].region_id
    }));
    assert_ne!(
        first[0].region_id,
        generate_region(GlobalRegion::new(far + 1, -far))[0].region_id
    );
}
