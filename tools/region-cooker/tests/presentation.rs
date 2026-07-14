use region_cooker::{
    PhysicalOrder, PresentationProfile, author_presentations, reorder_object_triples,
};
use region_format::{InstanceRecord, RECORDS_PER_REGION};

#[test]
fn orders_are_distinct_triple_preserving_permutations() {
    let source = (0..RECORDS_PER_REGION)
        .map(|local_id| InstanceRecord {
            position: [local_id as f32, 0.0, 0.0],
            height: 1.0,
            region_id: 7,
        })
        .collect::<Vec<_>>();
    let presentations = author_presentations(&source, PresentationProfile::Base);
    let (records_a, ids_a, presentation_a) =
        reorder_object_triples(source.clone(), presentations.clone(), PhysicalOrder::A);
    let (records_b, ids_b, presentation_b) =
        reorder_object_triples(source.clone(), presentations.clone(), PhysicalOrder::B);
    assert_ne!(ids_a, ids_b);
    for ((record, presentation), local_id) in records_a.iter().zip(&presentation_a).zip(&ids_a) {
        assert_eq!(*record, source[*local_id as usize]);
        assert_eq!(*presentation, presentations[*local_id as usize]);
    }
    for ((record, presentation), local_id) in records_b.iter().zip(&presentation_b).zip(&ids_b) {
        assert_eq!(*record, source[*local_id as usize]);
        assert_eq!(*presentation, presentations[*local_id as usize]);
    }
    let mut sorted_a = ids_a;
    sorted_a.sort_unstable();
    assert_eq!(sorted_a, (0..RECORDS_PER_REGION).collect::<Vec<_>>());
}

#[test]
fn mutation_profiles_change_only_the_selected_property() {
    let source = (0..RECORDS_PER_REGION)
        .map(|local_id| InstanceRecord {
            position: [local_id as f32, 0.0, 0.0],
            height: 1.0,
            region_id: 91,
        })
        .collect::<Vec<_>>();
    let base = author_presentations(&source, PresentationProfile::Base);
    for (profile, selected) in [
        (PresentationProfile::Archetype, 0),
        (PresentationProfile::Material, 1),
        (PresentationProfile::Yaw, 2),
        (PresentationProfile::Animation, 3),
    ] {
        let mutated = author_presentations(&source, profile);
        for (before, after) in base.iter().zip(mutated) {
            let fields = [
                before.archetype != after.archetype,
                before.material != after.material,
                before.yaw_q16 != after.yaw_q16,
                before.animation != after.animation,
            ];
            assert!(fields[selected]);
            assert_eq!(fields.iter().filter(|changed| **changed).count(), 1);
        }
    }
}
