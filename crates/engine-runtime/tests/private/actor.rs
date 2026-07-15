use super::{ActorHandle, ActorPresentation, ActorSlot};
use crate::RegionCoord;
use crate::terrain_query::{TerrainBody, TerrainBodyMotion, TerrainPosition};

fn motion(center: i32, velocity: i32) -> TerrainBodyMotion {
    TerrainBodyMotion::new(
        TerrainBody::new(
            TerrainPosition::new(RegionCoord::new(-7, 11), -4096, 2048).unwrap(),
            center,
            65_536,
        )
        .unwrap(),
        velocity,
    )
}

fn presentation() -> ActorPresentation {
    ActorPresentation::animated(7, 63, 0, 1, 0, 0)
}

#[test]
fn handle_rejects_generation_zero() {
    assert!(ActorHandle::new(0).is_err());
    assert_eq!(ActorHandle::new(9).unwrap().generation(), 9);
}

#[test]
fn slot_spawns_reads_and_despawns_exact_actor() {
    let mut slot = ActorSlot::new();
    let input = motion(200_000, -17);

    let first = slot.spawn(input, presentation()).unwrap();
    assert_eq!(first.handle.generation(), 1);
    assert_eq!(first.motion, input);
    assert_eq!(first.presentation, presentation());
    assert_eq!(slot.read(first.handle).unwrap(), first);
    assert_eq!(slot.despawn(first.handle).unwrap(), first);
    assert!(slot.read(first.handle).is_err());
}

#[test]
fn invalid_presentations_do_not_consume_generation_or_occupy_slot() {
    let invalid = [
        ActorPresentation::static_object(8, 0, 0),
        ActorPresentation::static_object(0, 64, 0),
        ActorPresentation::static_object(0, 0, 65_536),
        ActorPresentation::animated(7, 63, 0, 8, 0, 0),
        ActorPresentation::animated(7, 63, 0, 1, 64, 0),
    ];
    let mut slot = ActorSlot::new();
    for presentation in invalid {
        assert!(slot.spawn(motion(1, 2), presentation).is_err());
        assert_eq!(slot.generation, 0);
        assert!(slot.actor.is_none());
    }

    let actor = slot.spawn(motion(3, 4), presentation()).unwrap();
    assert_eq!(actor.handle.generation(), 1);
}

#[test]
fn occupied_spawn_and_wrong_handle_preserve_live_actor() {
    let mut slot = ActorSlot::new();
    let first = slot.spawn(motion(300_000, 23), presentation()).unwrap();
    let wrong = ActorHandle::new(first.handle.generation() + 1).unwrap();

    assert!(slot.spawn(motion(-100_000, -99), presentation()).is_err());
    assert!(slot.read(wrong).is_err());
    assert!(slot.despawn(wrong).is_err());
    assert_eq!(slot.read(first.handle).unwrap(), first);
}

#[test]
fn respawn_changes_generation_and_rejects_stale_handle() {
    let mut slot = ActorSlot::new();
    let first = slot.spawn(motion(1, 2), presentation()).unwrap();
    slot.despawn(first.handle).unwrap();

    let second = slot.spawn(motion(3, 4), presentation()).unwrap();
    assert_eq!(second.handle.generation(), 2);
    assert_ne!(second.handle, first.handle);
    assert!(slot.read(first.handle).is_err());
    assert!(slot.despawn(first.handle).is_err());
    assert_eq!(slot.read(second.handle).unwrap(), second);
}

#[test]
fn generation_exhaustion_leaves_empty_slot_unchanged() {
    let mut slot = ActorSlot {
        generation: u64::MAX,
        actor: None,
    };

    assert!(slot.spawn(motion(5, 6), presentation()).is_err());
    assert_eq!(slot.generation, u64::MAX);
    assert!(slot.actor.is_none());
}

#[test]
fn motion_replace_preserves_generation_and_presentation() {
    let mut slot = ActorSlot::new();
    let actor = slot.spawn(motion(100, -3), presentation()).unwrap();
    let replacement = motion(-200, 7);

    let output = slot.replace_motion(actor.handle, replacement).unwrap();
    assert_eq!(output.handle, actor.handle);
    assert_eq!(output.motion, replacement);
    assert_eq!(output.presentation, actor.presentation);
    assert_eq!(slot.read(actor.handle).unwrap(), output);
}

#[test]
fn empty_and_wrong_handle_replace_preserve_slot() {
    let mut slot = ActorSlot::new();
    let absent = ActorHandle::new(1).unwrap();
    assert!(slot.replace_motion(absent, motion(1, 2)).is_err());

    let actor = slot.spawn(motion(3, 4), presentation()).unwrap();
    let wrong = ActorHandle::new(2).unwrap();
    assert!(slot.replace_motion(wrong, motion(5, 6)).is_err());
    assert_eq!(slot.read(actor.handle).unwrap(), actor);
}
