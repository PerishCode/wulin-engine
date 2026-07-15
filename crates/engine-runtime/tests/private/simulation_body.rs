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

fn command(delta_x_q9: i32, delta_z_q9: i32) -> SimulationBodyCommand {
    SimulationBodyCommand {
        delta_x_q9,
        delta_z_q9,
        step_up_limit_q16: 0,
        step_acceleration_q16: 0,
    }
}

fn run(intervals: &[u64]) -> (SimulationSchedule, TerrainBodyMotion, u32) {
    let mut schedule = SimulationSchedule::new();
    let mut body = motion();
    let mut queries = 0;
    for elapsed in intervals {
        let prepared =
            prepare_simulation_body(schedule, body, *elapsed, command(1, -1), flat).unwrap();
        schedule = prepared.schedule;
        body = prepared.body.output;
        queries += prepared.body.terrain_query_count;
    }
    (schedule, body, queries)
}

#[test]
fn fractional_elapsed_commits_no_body_step() {
    let input = motion();
    let prepared =
        prepare_simulation_body(SimulationSchedule::new(), input, 1, command(17, -19), flat)
            .unwrap();
    assert_eq!(prepared.simulation.step_count, 0);
    assert_eq!(prepared.simulation.remainder_numerator, 60);
    assert_eq!(prepared.body.output, input);
    assert_eq!(prepared.body.terrain_query_count, 0);
}

#[test]
fn coarse_and_nominal_partitions_match() {
    let coarse = [125_000_000; 8];
    let nominal = [vec![16_666_666; 20], vec![16_666_667; 40]].concat();
    let (coarse_schedule, coarse_body, coarse_queries) = run(&coarse);
    let (nominal_schedule, nominal_body, nominal_queries) = run(&nominal);
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
    assert_eq!(coarse_body, nominal_body);
    assert_eq!(coarse_queries, 60);
    assert_eq!(nominal_queries, 60);
}

#[test]
fn schedule_copy_survives_body_failure() {
    let schedule = SimulationSchedule::new();
    let status = schedule.status_json();
    let input = motion();
    let mut queries = 0;
    let result = prepare_simulation_body(schedule, input, 125_000_000, command(1, 0), |position| {
        queries += 1;
        if queries == 3 {
            bail!("controlled body failure");
        }
        flat(position)
    });
    let error = match result {
        Ok(_) => panic!("controlled simulation-body failure unexpectedly succeeded"),
        Err(error) => error,
    };
    assert_eq!(
        error.to_string(),
        "retained terrain body batch step 3 of 7 failed: controlled body failure"
    );
    assert_eq!(schedule.status_json(), status);
    assert_eq!(input, motion());
    assert_eq!(queries, 3);
}
