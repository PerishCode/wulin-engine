use std::time::Instant;

use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const SIMULATION_STEPS_PER_SECOND: u64 = 60;
pub const SIMULATION_TIME_DENOMINATOR: u64 = 1_000_000_000;
pub const SIMULATION_MAX_ELAPSED_NANOSECONDS: u64 = 125_000_000;
pub const SIMULATION_MAX_STEPS_PER_ADVANCE: u32 = 8;

const REVISION: &str = "deterministic-fixed-simulation-schedule-v1";
const PROBE_ADVANCE_COUNT: u64 = 28_800;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationAdvance {
    pub elapsed_nanoseconds: u64,
    pub start_tick: u64,
    pub step_count: u32,
    pub end_tick: u64,
    pub remainder_numerator: u64,
    pub remainder_denominator: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SimulationSchedule {
    tick: u64,
    remainder_numerator: u64,
    successful_advance_count: u64,
    emitted_step_count: u64,
}

impl SimulationSchedule {
    pub(crate) const fn new() -> Self {
        Self {
            tick: 0,
            remainder_numerator: 0,
            successful_advance_count: 0,
            emitted_step_count: 0,
        }
    }

    pub(crate) fn status_json(self) -> Value {
        json!({
            "revision": REVISION,
            "tick": self.tick,
            "remainderNumerator": self.remainder_numerator,
            "remainderDenominator": SIMULATION_TIME_DENOMINATOR,
            "stepsPerSecond": SIMULATION_STEPS_PER_SECOND,
            "maximumElapsedNanoseconds": SIMULATION_MAX_ELAPSED_NANOSECONDS,
            "maximumStepsPerAdvance": SIMULATION_MAX_STEPS_PER_ADVANCE,
            "successfulAdvanceCount": self.successful_advance_count,
            "emittedStepCount": self.emitted_step_count,
        })
    }

    pub(crate) fn advance(&mut self, elapsed_nanoseconds: u64) -> Result<SimulationAdvance> {
        ensure!(
            elapsed_nanoseconds <= SIMULATION_MAX_ELAPSED_NANOSECONDS,
            "simulation elapsed nanoseconds must be in [0, {SIMULATION_MAX_ELAPSED_NANOSECONDS}]"
        );
        ensure!(
            self.remainder_numerator < SIMULATION_TIME_DENOMINATOR,
            "simulation schedule remainder is outside its rational denominator"
        );
        let scaled_elapsed = elapsed_nanoseconds
            .checked_mul(SIMULATION_STEPS_PER_SECOND)
            .context("simulation elapsed scaling overflowed")?;
        let total = self
            .remainder_numerator
            .checked_add(scaled_elapsed)
            .context("simulation accumulator overflowed")?;
        let step_count = total / SIMULATION_TIME_DENOMINATOR;
        ensure!(
            step_count <= u64::from(SIMULATION_MAX_STEPS_PER_ADVANCE),
            "simulation advance exceeded its fixed step-batch bound"
        );
        let next_remainder = total % SIMULATION_TIME_DENOMINATOR;
        let next_tick = self
            .tick
            .checked_add(step_count)
            .context("simulation tick overflowed")?;
        let next_advance_count = self
            .successful_advance_count
            .checked_add(1)
            .context("simulation advance counter overflowed")?;
        let next_emitted_step_count = self
            .emitted_step_count
            .checked_add(step_count)
            .context("simulation emitted-step counter overflowed")?;
        let step_count = u32::try_from(step_count).context("simulation step batch exceeded u32")?;
        let result = SimulationAdvance {
            elapsed_nanoseconds,
            start_tick: self.tick,
            step_count,
            end_tick: next_tick,
            remainder_numerator: next_remainder,
            remainder_denominator: SIMULATION_TIME_DENOMINATOR,
        };

        self.tick = next_tick;
        self.remainder_numerator = next_remainder;
        self.successful_advance_count = next_advance_count;
        self.emitted_step_count = next_emitted_step_count;
        Ok(result)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ProbeRun {
    schedule: SimulationSchedule,
    batch_histogram: [u64; 9],
    result_sha256: String,
}

fn run_one_hour_probe() -> Result<ProbeRun> {
    let mut schedule = SimulationSchedule::new();
    let mut batch_histogram = [0; 9];
    let mut digest = Sha256::new();
    for _ in 0..PROBE_ADVANCE_COUNT {
        let advance = schedule.advance(SIMULATION_MAX_ELAPSED_NANOSECONDS)?;
        batch_histogram[advance.step_count as usize] += 1;
        digest.update(advance.elapsed_nanoseconds.to_le_bytes());
        digest.update(advance.start_tick.to_le_bytes());
        digest.update(advance.step_count.to_le_bytes());
        digest.update(advance.end_tick.to_le_bytes());
        digest.update(advance.remainder_numerator.to_le_bytes());
        digest.update(advance.remainder_denominator.to_le_bytes());
    }
    Ok(ProbeRun {
        schedule,
        batch_histogram,
        result_sha256: format!("{:x}", digest.finalize()),
    })
}

pub(crate) fn simulation_probe() -> Result<Value> {
    let started = Instant::now();
    let measured = run_one_hour_probe()?;
    let elapsed_cpu_nanoseconds = u64::try_from(started.elapsed().as_nanos())
        .context("simulation probe CPU duration exceeded u64 nanoseconds")?;
    let replay = run_one_hour_probe()?;
    ensure!(measured == replay, "simulation probe replay diverged");
    let elapsed_input_nanoseconds = PROBE_ADVANCE_COUNT
        .checked_mul(SIMULATION_MAX_ELAPSED_NANOSECONDS)
        .context("simulation probe elapsed input overflowed")?;
    ensure!(
        measured.schedule.tick == 216_000
            && measured.schedule.remainder_numerator == 0
            && measured.schedule.successful_advance_count == PROBE_ADVANCE_COUNT
            && measured.schedule.emitted_step_count == 216_000
            && measured.batch_histogram[7] == 14_400
            && measured.batch_histogram[8] == 14_400
            && measured.batch_histogram[..7]
                .iter()
                .all(|count| *count == 0),
        "simulation probe exact one-hour invariant diverged"
    );
    Ok(json!({
        "revision": REVISION,
        "elapsedInputNanoseconds": elapsed_input_nanoseconds,
        "advanceCount": PROBE_ADVANCE_COUNT,
        "emittedStepCount": measured.schedule.emitted_step_count,
        "finalTick": measured.schedule.tick,
        "remainderNumerator": measured.schedule.remainder_numerator,
        "remainderDenominator": SIMULATION_TIME_DENOMINATOR,
        "batchHistogram": measured.batch_histogram,
        "resultSha256": measured.result_sha256,
        "replaySha256": replay.result_sha256,
        "elapsedCpuNanoseconds": elapsed_cpu_nanoseconds,
        "perAdvanceAllocationBytes": 0,
        "sourceReadCount": 0,
        "gpuCopyCount": 0,
        "gpuReadbackCount": 0,
        "fenceWaitCount": 0,
        "synchronizationCount": 0,
    }))
}

#[cfg(test)]
#[path = "../../tests/private/simulation.rs"]
mod tests;
