use region_cooker::{IdentityOrder, reorder_identity_records};
use region_format::{InstanceRecord, RECORDS_PER_REGION};

#[test]
fn orders_are_distinct_pair_preserving_permutations() {
    let source = (0..RECORDS_PER_REGION)
        .map(|local_id| InstanceRecord {
            position: [local_id as f32, 0.0, 0.0],
            height: 1.0,
            region_id: 7,
        })
        .collect::<Vec<_>>();
    let (records_a, ids_a) = reorder_identity_records(source.clone(), IdentityOrder::A);
    let (records_b, ids_b) = reorder_identity_records(source.clone(), IdentityOrder::B);
    assert_ne!(ids_a, ids_b);
    for (record, local_id) in records_a.iter().zip(&ids_a) {
        assert_eq!(*record, source[*local_id as usize]);
    }
    for (record, local_id) in records_b.iter().zip(&ids_b) {
        assert_eq!(*record, source[*local_id as usize]);
    }
    let mut sorted_a = ids_a;
    sorted_a.sort_unstable();
    assert_eq!(sorted_a, (0..RECORDS_PER_REGION).collect::<Vec<_>>());
}
