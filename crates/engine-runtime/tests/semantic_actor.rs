use engine_runtime::semantic_object;

const ACTOR_OBJECT_ID: u32 = 98_305;

#[test]
fn actor_id_is_disjoint() {
    let actor = semantic_object(ACTOR_OBJECT_ID).unwrap();
    assert_eq!(actor.name, "runtime.actor");
    assert_eq!(actor.kind, "runtime-actor");
    assert!(semantic_object(ACTOR_OBJECT_ID - 1).is_none());
    assert!(semantic_object(ACTOR_OBJECT_ID + 1).is_none());
}
