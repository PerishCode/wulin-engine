use super::*;

fn advance_sequence(intervals: &[u64]) -> (SimulationSchedule, Vec<u32>) {
    let mut schedule = SimulationSchedule::new();
    let batches = intervals
        .iter()
        .map(|elapsed| schedule.advance(*elapsed).unwrap().step_count)
        .collect();
    (schedule, batches)
}

#[test]
fn one_second_is_partition_invariant() {
    let coarse = [125_000_000; 8];
    let mut nominal = vec![16_666_666; 20];
    nominal.extend([16_666_667; 40]);
    let mut reordered = nominal.clone();
    reordered.reverse();

    let (coarse_schedule, coarse_batches) = advance_sequence(&coarse);
    let (nominal_schedule, nominal_batches) = advance_sequence(&nominal);
    let (reordered_schedule, _) = advance_sequence(&reordered);
    assert_eq!(coarse_batches, [7, 8, 7, 8, 7, 8, 7, 8]);
    assert_eq!(nominal_batches.iter().sum::<u32>(), 60);
    assert_eq!(coarse_schedule.tick, 60);
    assert_eq!(coarse_schedule.remainder_numerator, 0);
    assert_eq!(nominal_schedule.tick, coarse_schedule.tick);
    assert_eq!(nominal_schedule.remainder_numerator, 0);
    assert_eq!(reordered_schedule.tick, coarse_schedule.tick);
    assert_eq!(reordered_schedule.remainder_numerator, 0);
    assert_eq!(advance_sequence(&nominal).1, nominal_batches);
}

#[test]
fn bounds_batches_and_substeps() {
    let mut schedule = SimulationSchedule::new();
    assert_eq!(schedule.advance(0).unwrap().step_count, 0);
    assert_eq!(schedule.advance(1).unwrap().step_count, 0);
    assert_eq!(schedule.remainder_numerator, 60);
    let before = schedule;
    assert!(
        schedule
            .advance(SIMULATION_MAX_ELAPSED_NANOSECONDS + 1)
            .is_err()
    );
    assert_eq!(schedule, before);

    schedule.remainder_numerator = SIMULATION_TIME_DENOMINATOR - 1;
    let batch = schedule
        .advance(SIMULATION_MAX_ELAPSED_NANOSECONDS)
        .unwrap();
    assert_eq!(batch.step_count, SIMULATION_MAX_STEPS_PER_ADVANCE);
    assert_eq!(batch.remainder_numerator, 499_999_999);
}

#[test]
fn failures_are_transactional() {
    for mut schedule in [
        SimulationSchedule {
            tick: u64::MAX,
            remainder_numerator: SIMULATION_TIME_DENOMINATOR - 1,
            successful_advance_count: 0,
            emitted_step_count: 0,
        },
        SimulationSchedule {
            tick: 0,
            remainder_numerator: 0,
            successful_advance_count: u64::MAX,
            emitted_step_count: 0,
        },
        SimulationSchedule {
            tick: 0,
            remainder_numerator: SIMULATION_TIME_DENOMINATOR - 1,
            successful_advance_count: 0,
            emitted_step_count: u64::MAX,
        },
    ] {
        let before = schedule;
        assert!(schedule.advance(1).is_err());
        assert_eq!(schedule, before);
    }
}

#[test]
fn one_hour_has_no_drift() {
    let mut schedule = SimulationSchedule::new();
    let mut seven_step_batches = 0;
    let mut eight_step_batches = 0;
    for _ in 0..28_800 {
        match schedule.advance(125_000_000).unwrap().step_count {
            7 => seven_step_batches += 1,
            8 => eight_step_batches += 1,
            value => panic!("unexpected simulation batch {value}"),
        }
    }
    assert_eq!(seven_step_batches, 14_400);
    assert_eq!(eight_step_batches, 14_400);
    assert_eq!(schedule.tick, 216_000);
    assert_eq!(schedule.remainder_numerator, 0);
    assert_eq!(schedule.successful_advance_count, 28_800);
    assert_eq!(schedule.emitted_step_count, 216_000);
}
