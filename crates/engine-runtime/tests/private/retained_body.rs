use super::{TerrainBodyHandle, TerrainBodySlot};
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

#[test]
fn handle_rejects_generation_zero() {
    assert!(TerrainBodyHandle::new(0).is_err());
    assert_eq!(TerrainBodyHandle::new(9).unwrap().generation(), 9);
}

#[test]
fn slot_spawns_reads_and_despawns_exact_motion() {
    let mut slot = TerrainBodySlot::new();
    let input = motion(200_000, -17);

    let first = slot.spawn(input).unwrap();
    assert_eq!(first.handle.generation(), 1);
    assert_eq!(first.motion, input);
    assert_eq!(slot.read(first.handle).unwrap(), first);
    assert_eq!(slot.despawn(first.handle).unwrap(), first);
    assert!(slot.read(first.handle).is_err());
}

#[test]
fn occupied_spawn_and_wrong_handle_preserve_live_value() {
    let mut slot = TerrainBodySlot::new();
    let first = slot.spawn(motion(300_000, 23)).unwrap();
    let wrong = TerrainBodyHandle::new(first.handle.generation() + 1).unwrap();

    assert!(slot.spawn(motion(-100_000, -99)).is_err());
    assert!(slot.read(wrong).is_err());
    assert!(slot.despawn(wrong).is_err());
    assert_eq!(slot.read(first.handle).unwrap(), first);
}

#[test]
fn respawn_changes_generation_and_rejects_stale_handle() {
    let mut slot = TerrainBodySlot::new();
    let first = slot.spawn(motion(1, 2)).unwrap();
    slot.despawn(first.handle).unwrap();

    let second = slot.spawn(motion(3, 4)).unwrap();
    assert_eq!(second.handle.generation(), 2);
    assert_ne!(second.handle, first.handle);
    assert!(slot.read(first.handle).is_err());
    assert!(slot.despawn(first.handle).is_err());
    assert_eq!(slot.read(second.handle).unwrap(), second);
}

#[test]
fn generation_exhaustion_leaves_empty_slot_unchanged() {
    let mut slot = TerrainBodySlot {
        generation: u64::MAX,
        retained: None,
    };

    assert!(slot.spawn(motion(5, 6)).is_err());
    assert_eq!(slot.generation, u64::MAX);
    assert!(slot.retained.is_none());
}

#[test]
fn replace_preserves_generation_and_changes_only_motion() {
    let mut slot = TerrainBodySlot::new();
    let retained = slot.spawn(motion(100, -3)).unwrap();
    let replacement = motion(-200, 7);

    let output = slot.replace(retained.handle, replacement).unwrap();
    assert_eq!(output.handle, retained.handle);
    assert_eq!(output.motion, replacement);
    assert_eq!(slot.read(retained.handle).unwrap(), output);
}

#[test]
fn empty_and_wrong_handle_replace_preserve_slot() {
    let mut slot = TerrainBodySlot::new();
    let absent = TerrainBodyHandle::new(1).unwrap();
    assert!(slot.replace(absent, motion(1, 2)).is_err());

    let retained = slot.spawn(motion(3, 4)).unwrap();
    let wrong = TerrainBodyHandle::new(2).unwrap();
    assert!(slot.replace(wrong, motion(5, 6)).is_err());
    assert_eq!(slot.read(retained.handle).unwrap(), retained);
}
