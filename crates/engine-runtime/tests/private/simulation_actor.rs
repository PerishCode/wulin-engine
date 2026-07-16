use anyhow::{Result, bail};

use super::*;
use crate::RegionCoord;
use crate::terrain_query::{TERRAIN_QUERY_HEIGHT_DENOMINATOR, TerrainBody, TerrainTriangle};

fn motion() -> TerrainBodyMotion {
    let position = TerrainPosition::new(RegionCoord::ZERO, 0, 0).unwrap();
    let body = TerrainBody::new(position, 65_536, 65_536).unwrap();
    TerrainBodyMotion::new(body, 0)
}

fn flat(_: TerrainPosition) -> Result<TerrainHeight> {
    Ok(TerrainHeight {
        height_numerator: 0,
        height_denominator: TERRAIN_QUERY_HEIGHT_DENOMINATOR,
        triangle: TerrainTriangle::First,
    })
}

fn command(delta_x_q9: i32, delta_z_q9: i32) -> ActorSimulationCommand {
    ActorSimulationCommand {
        delta_x_q9,
        delta_z_q9,
        step_up_limit_q16: 0,
        initial_step_velocity_delta_q16: 0,
        step_acceleration_q16: 0,
        presentation: ActorPresentation::animated(7, 63, 0, 1, 0, 0),
    }
}

fn run(intervals: &[u64]) -> (SimulationSchedule, TerrainBodyMotion, u32) {
    let mut schedule = SimulationSchedule::new();
    let mut actor_motion = motion();
    let mut queries = 0;
    for elapsed in intervals {
        let prepared =
            prepare_simulation_actor(schedule, actor_motion, *elapsed, command(1, -1), flat)
                .unwrap();
        schedule = prepared.schedule;
        actor_motion = prepared.motion.output;
        queries += prepared.motion.terrain_query_count;
    }
    (schedule, actor_motion, queries)
}

#[test]
fn fractional_elapsed_commits_no_actor_step() {
    let input = motion();
    let mut impulse = command(17, -19);
    impulse.initial_step_velocity_delta_q16 = 7_777;
    let prepared =
        prepare_simulation_actor(SimulationSchedule::new(), input, 1, impulse, flat).unwrap();
    assert_eq!(prepared.simulation.step_count, 0);
    assert_eq!(prepared.simulation.remainder_numerator, 60);
    assert_eq!(prepared.motion.output, input);
    assert_eq!(prepared.motion.terrain_query_count, 0);
}

#[test]
fn emitted_step_applies_initial_velocity_delta_before_acceleration() {
    let input = motion();
    let mut impulse = command(0, 0);
    impulse.initial_step_velocity_delta_q16 = 1_000;
    impulse.step_acceleration_q16 = -100;
    let prepared =
        prepare_simulation_actor(SimulationSchedule::new(), input, 16_666_667, impulse, flat)
            .unwrap();
    assert_eq!(prepared.simulation.step_count, 1);
    assert_eq!(prepared.motion.output.step_velocity_q16(), 900);
    assert_eq!(
        prepared.motion.output.body().center_height_numerator(),
        66_436
    );
    assert_eq!(prepared.motion.terrain_query_count, 1);
}

#[test]
fn initial_velocity_delta_overflow_preserves_schedule_without_query() {
    let schedule = SimulationSchedule::new();
    let status = schedule.status_json();
    let input = TerrainBodyMotion::new(motion().body(), i32::MAX);
    let mut impulse = command(0, 0);
    impulse.initial_step_velocity_delta_q16 = 1;
    let mut queries = 0;
    let result = prepare_simulation_actor(schedule, input, 16_666_667, impulse, |position| {
        queries += 1;
        flat(position)
    });
    assert!(result.is_err());
    assert_eq!(queries, 0);
    assert_eq!(schedule.status_json(), status);
}

#[test]
fn invalid_presentation_fails_before_schedule_or_query_work() {
    let schedule = SimulationSchedule::new();
    let status = schedule.status_json();
    let mut invalid = command(1, -1);
    invalid.presentation = ActorPresentation::static_object(8, 0, 0);
    let mut query_count = 0;
    let result = prepare_simulation_actor(schedule, motion(), 125_000_000, invalid, |position| {
        query_count += 1;
        flat(position)
    });
    let error = match result {
        Ok(_) => panic!("invalid simulation presentation unexpectedly succeeded"),
        Err(error) => error,
    };

    assert_eq!(
        error.to_string(),
        "presentation archetype 8 exceeds catalog capacity"
    );
    assert_eq!(query_count, 0);
    assert_eq!(schedule.status_json(), status);
}

#[test]
fn coarse_and_nominal_partitions_match() {
    let coarse = [125_000_000; 8];
    let nominal = [vec![16_666_666; 20], vec![16_666_667; 40]].concat();
    let (coarse_schedule, coarse_motion, coarse_queries) = run(&coarse);
    let (nominal_schedule, nominal_motion, nominal_queries) = run(&nominal);
    let coarse_status = coarse_schedule.status_json();
    let nominal_status = nominal_schedule.status_json();
    assert_eq!(coarse_status["tick"], nominal_status["tick"]);
    assert_eq!(
        coarse_status["remainderNumerator"],
        nominal_status["remainderNumerator"]
    );
    assert_eq!(
        coarse_status["emittedStepCount"],
        nominal_status["emittedStepCount"]
    );
    assert_eq!(coarse_status["successfulAdvanceCount"], 8);
    assert_eq!(nominal_status["successfulAdvanceCount"], 60);
    assert_eq!(coarse_motion, nominal_motion);
    assert_eq!(coarse_queries, 60);
    assert_eq!(nominal_queries, 60);
}

#[test]
fn schedule_copy_survives_actor_motion_failure() {
    let schedule = SimulationSchedule::new();
    let status = schedule.status_json();
    let input = motion();
    let mut queries = 0;
    let result =
        prepare_simulation_actor(schedule, input, 125_000_000, command(1, 0), |position| {
            queries += 1;
            if queries == 3 {
                bail!("controlled actor motion failure");
            }
            flat(position)
        });
    let error = match result {
        Ok(_) => panic!("controlled simulation-actor failure unexpectedly succeeded"),
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "terrain-body motion batch step 3 of 7 failed: controlled actor motion failure"
    );
    assert_eq!(schedule.status_json(), status);
    assert_eq!(input, motion());
    assert_eq!(queries, 3);
}
