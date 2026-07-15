use super::*;

use crate::load::{LoadConfig, MAX_REGION_SIDE};
use crate::region::RegionCoord;
use crate::runtime::{ActorHandle, ActorPresentation};
use crate::scene::{Camera, SceneState};
use crate::terrain_query::{TerrainBody, TerrainBodyMotion, TerrainPosition};

const FAR: i64 = 1_i64 << 40;

fn config(origin: RegionCoord, center: RegionCoord) -> GlobalRegionConfig {
    GlobalRegionConfig::new(origin.x, origin.z, center.x, center.z, 2).unwrap()
}

fn actor(region: RegionCoord, local: [i32; 2]) -> RuntimeActor {
    let position = TerrainPosition::new(region, local[0], local[1]).unwrap();
    let body = TerrainBody::new(position, 200_003, 65_537).unwrap();
    RuntimeActor {
        handle: ActorHandle::new(7).unwrap(),
        motion: TerrainBodyMotion::new(body, -19),
        presentation: ActorPresentation::animated(7, 63, 32_768, 1, 17, 2),
    }
}

fn project_config(global: GlobalRegionConfig, actor: RuntimeActor) -> ActorRenderProjection {
    project(global, global.local_config().unwrap(), actor).unwrap()
}

#[test]
fn center_and_corners_project_exactly() {
    let center = RegionCoord::new(FAR, -FAR);
    let global = config(center, center);
    let cases = [
        (
            [-2, -2],
            [-4096, -4096],
            0,
            62 * 128 + 62,
            [-20_480, -20_480],
        ),
        ([0, 0], [0, 0], 12, 64 * 128 + 64, [0, 0]),
        ([2, -2], [4095, -4096], 4, 62 * 128 + 66, [20_479, -20_480]),
        ([-2, 2], [-4096, 4095], 20, 66 * 128 + 62, [-20_480, 20_479]),
        ([2, 2], [4095, 4095], 24, 66 * 128 + 66, [20_479, 20_479]),
    ];
    for (offset, local, active_index, semantic_region, expected_q9) in cases {
        let input = actor(
            center
                .checked_offset(i64::from(offset[0]), i64::from(offset[1]))
                .unwrap(),
            local,
        );
        let output = project_config(global, input);
        assert_eq!(output.actor, input);
        assert_eq!(output.global_config, global);
        assert_eq!(output.active_region_index, active_index);
        assert_eq!(output.semantic_region, semantic_region);
        assert_eq!(output.window_position_q9, expected_q9);
        assert_eq!(output.center_height_q16, 200_003);
        assert_eq!(output.half_height_q16, 65_537);
        assert_eq!(output.position_denominator, 512);
        assert_eq!(output.height_denominator, 65_536);
    }
}

#[test]
fn signed_seams_are_continuous() {
    let center = RegionCoord::new(FAR, -FAR);
    let global = config(center, center);
    let start = TerrainPosition::new(center, 4095, -4096).unwrap();
    let crossed = start.translated_q9(1, -1).unwrap();
    let start = project_config(
        global,
        actor(start.region(), [start.local_x_q9(), start.local_z_q9()]),
    );
    let crossed = project_config(
        global,
        actor(
            crossed.region(),
            [crossed.local_x_q9(), crossed.local_z_q9()],
        ),
    );
    assert_eq!(
        crossed.window_position_q9,
        [
            start.window_position_q9[0] + 1,
            start.window_position_q9[1] - 1
        ]
    );
}

#[test]
fn origin_alias_and_rollover_do_not_change_spatial_projection() {
    let center = RegionCoord::new(FAR + 123, -FAR - 456);
    let actor = actor(center.checked_offset(1, -1).unwrap(), [73, -91]);
    let aliased = config(center.checked_offset(-32, 32).unwrap(), center);
    let recentered = config(center, center);
    let before = project_config(aliased, actor);
    let after = project_config(recentered, actor);
    assert_ne!(before.global_config, after.global_config);
    assert_eq!(before.active_region_index, after.active_region_index);
    assert_eq!(before.semantic_region, after.semantic_region);
    assert_eq!(before.window_position_q9, after.window_position_q9);
    assert_eq!(before.actor, after.actor);
    assert_eq!(
        scene_center(before).unwrap().map(f32::to_bits),
        [
            ((33 * 8192 + 73) as f32 / 512.0).to_bits(),
            (200_003_f32 / 65_536.0).to_bits(),
            ((-33 * 8192 - 91) as f32 / 512.0).to_bits(),
        ]
    );
    assert_eq!(
        scene_center(after).unwrap().map(f32::to_bits),
        [
            ((8192 + 73) as f32 / 512.0).to_bits(),
            (200_003_f32 / 65_536.0).to_bits(),
            ((-8192 - 91) as f32 / 512.0).to_bits(),
        ]
    );
}

#[test]
fn scene_center_preserves_q9_seams_and_q16_height() {
    let center = RegionCoord::new(FAR, -FAR);
    let global = config(center, center);
    let start = actor(center, [4095, -4096]);
    let crossed_position = start.motion.body().position().translated_q9(1, -1).unwrap();
    let crossed = actor(
        crossed_position.region(),
        [crossed_position.local_x_q9(), crossed_position.local_z_q9()],
    );
    assert_eq!(
        scene_center(project_config(global, start))
            .unwrap()
            .map(f32::to_bits),
        [
            (4095_f32 / 512.0).to_bits(),
            (200_003_f32 / 65_536.0).to_bits(),
            (-4096_f32 / 512.0).to_bits(),
        ]
    );
    assert_eq!(
        scene_center(project_config(global, crossed))
            .unwrap()
            .map(f32::to_bits),
        [
            (4096_f32 / 512.0).to_bits(),
            (200_003_f32 / 65_536.0).to_bits(),
            (-4097_f32 / 512.0).to_bits(),
        ]
    );
}

#[test]
fn outside_window_and_config_divergence_fail_without_projection() {
    let center = RegionCoord::new(FAR, -FAR);
    let global = config(center, center);
    let outside = actor(center.checked_offset(3, 0).unwrap(), [0, 0]);
    assert!(
        project(global, global.local_config().unwrap(), outside)
            .unwrap_err()
            .to_string()
            .contains("outside the active render window")
    );
    let divergent = LoadConfig::new(MAX_REGION_SIDE, 65, 64, 2).unwrap();
    assert!(
        project(global, divergent, actor(center, [0, 0]))
            .unwrap_err()
            .to_string()
            .contains("local/global configs diverged")
    );
}

fn camera_bits(camera: Camera) -> ([u32; 3], [u32; 3], u32, u32) {
    (
        camera.position.map(f32::to_bits),
        camera.target.map(f32::to_bits),
        camera.vertical_fov_degrees.to_bits(),
        camera.near_plane_meters.to_bits(),
    )
}

#[test]
fn anchor_commits_candidate() {
    let mut scene = SceneState::new();
    scene
        .set_camera_from_anchor(
            [16.25, 2.164_062_5, -8.5],
            [9.0, 4.0, 12.0],
            [0.0, -1.0, -3.0],
            60.0,
        )
        .unwrap();
    assert_eq!(
        camera_bits(scene.camera()),
        (
            [25.25, 6.164_062_5, 3.5].map(f32::to_bits),
            [16.25, 1.164_062_5, -11.5].map(f32::to_bits),
            60.0_f32.to_bits(),
            0.1_f32.to_bits(),
        )
    );
}

#[test]
fn rejection_preserves_camera() {
    let rejected = [
        ([0.0, 0.0, 0.0], [f32::NAN, 0.0, 0.0], [0.0, 0.0, 0.0], 60.0),
        ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0], 60.0),
        ([0.0, 0.0, 0.0], [9.0, 6.0, 12.0], [0.0, 1.0, -3.0], 19.0),
        (
            [f32::MAX, 0.0, 0.0],
            [f32::MAX, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            60.0,
        ),
    ];
    for (anchor, position_offset, target_offset, fov) in rejected {
        let mut scene = SceneState::new();
        let before = camera_bits(scene.camera());
        assert!(
            scene
                .set_camera_from_anchor(anchor, position_offset, target_offset, fov)
                .is_err()
        );
        assert_eq!(camera_bits(scene.camera()), before);
    }
}
