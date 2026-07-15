use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use engine_runtime::SIMULATION_MAX_ELAPSED_NANOSECONDS;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(
    tag = "outcome",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum HostElapsedSample {
    Reset,
    Ready {
        elapsed_nanoseconds: u64,
    },
    Stalled {
        elapsed_nanoseconds: u64,
        maximum_elapsed_nanoseconds: u64,
    },
    Suspended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostClockStatus {
    pub suspended: bool,
    pub has_baseline: bool,
    pub sample_count: u64,
    pub reset_count: u64,
    pub ready_count: u64,
    pub stall_count: u64,
    pub suspended_sample_count: u64,
    pub suspend_count: u64,
    pub resume_count: u64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ClockCounters {
    sample_count: u64,
    reset_count: u64,
    ready_count: u64,
    stall_count: u64,
    suspended_sample_count: u64,
    suspend_count: u64,
    resume_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostClock {
    baseline: Option<Instant>,
    suspended: bool,
    counters: ClockCounters,
}

impl HostClock {
    pub const fn new() -> Self {
        Self {
            baseline: None,
            suspended: false,
            counters: ClockCounters {
                sample_count: 0,
                reset_count: 0,
                ready_count: 0,
                stall_count: 0,
                suspended_sample_count: 0,
                suspend_count: 0,
                resume_count: 0,
            },
        }
    }

    pub fn sample(&mut self) -> Result<HostElapsedSample> {
        self.sample_at(Instant::now())
    }

    pub fn suspend(&mut self) -> Result<()> {
        if self.suspended {
            return Ok(());
        }
        let suspend_count = increment(self.counters.suspend_count, "host-clock suspend count")?;
        self.suspended = true;
        self.baseline = None;
        self.counters.suspend_count = suspend_count;
        Ok(())
    }

    pub fn resume(&mut self) -> Result<()> {
        if !self.suspended {
            return Ok(());
        }
        let resume_count = increment(self.counters.resume_count, "host-clock resume count")?;
        self.suspended = false;
        self.baseline = None;
        self.counters.resume_count = resume_count;
        Ok(())
    }

    pub const fn status(&self) -> HostClockStatus {
        HostClockStatus {
            suspended: self.suspended,
            has_baseline: self.baseline.is_some(),
            sample_count: self.counters.sample_count,
            reset_count: self.counters.reset_count,
            ready_count: self.counters.ready_count,
            stall_count: self.counters.stall_count,
            suspended_sample_count: self.counters.suspended_sample_count,
            suspend_count: self.counters.suspend_count,
            resume_count: self.counters.resume_count,
        }
    }

    fn sample_at(&mut self, now: Instant) -> Result<HostElapsedSample> {
        let sample_count = increment(self.counters.sample_count, "host-clock sample count")?;

        if self.suspended {
            let suspended_sample_count = increment(
                self.counters.suspended_sample_count,
                "host-clock suspended sample count",
            )?;
            self.counters.sample_count = sample_count;
            self.counters.suspended_sample_count = suspended_sample_count;
            return Ok(HostElapsedSample::Suspended);
        }

        let Some(baseline) = self.baseline else {
            let reset_count = increment(self.counters.reset_count, "host-clock reset count")?;
            self.baseline = Some(now);
            self.counters.sample_count = sample_count;
            self.counters.reset_count = reset_count;
            return Ok(HostElapsedSample::Reset);
        };

        let elapsed = now
            .checked_duration_since(baseline)
            .context("host-clock sample regressed behind its monotonic baseline")?;
        let elapsed_nanoseconds = duration_nanoseconds(elapsed)?;
        let (outcome, ready_count, stall_count) =
            if elapsed_nanoseconds <= SIMULATION_MAX_ELAPSED_NANOSECONDS {
                (
                    HostElapsedSample::Ready {
                        elapsed_nanoseconds,
                    },
                    increment(self.counters.ready_count, "host-clock ready count")?,
                    self.counters.stall_count,
                )
            } else {
                (
                    HostElapsedSample::Stalled {
                        elapsed_nanoseconds,
                        maximum_elapsed_nanoseconds: SIMULATION_MAX_ELAPSED_NANOSECONDS,
                    },
                    self.counters.ready_count,
                    increment(self.counters.stall_count, "host-clock stall count")?,
                )
            };

        self.baseline = Some(now);
        self.counters.sample_count = sample_count;
        self.counters.ready_count = ready_count;
        self.counters.stall_count = stall_count;
        Ok(outcome)
    }
}

impl Default for HostClock {
    fn default() -> Self {
        Self::new()
    }
}

fn increment(value: u64, name: &str) -> Result<u64> {
    value
        .checked_add(1)
        .with_context(|| format!("{name} overflowed"))
}

fn duration_nanoseconds(duration: Duration) -> Result<u64> {
    u64::try_from(duration.as_nanos()).context("host-clock elapsed nanoseconds exceeded u64")
}

#[cfg(test)]
#[path = "../tests/private/clock.rs"]
mod tests;
