use anyhow::{Context, Result, ensure};
use serde::Serialize;
use serde_json::{Value, json};

pub const SIMULATION_STEPS_PER_SECOND: u64 = 60;
pub const SIMULATION_TIME_DENOMINATOR: u64 = 1_000_000_000;
pub const SIMULATION_MAX_ELAPSED_NANOSECONDS: u64 = 125_000_000;
pub const SIMULATION_MAX_STEPS_PER_ADVANCE: u32 = 8;

const REVISION: &str = "deterministic-fixed-simulation-schedule-v1";

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

#[cfg(test)]
#[path = "../../tests/private/simulation.rs"]
mod tests;
