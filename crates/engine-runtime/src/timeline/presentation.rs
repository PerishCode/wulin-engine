use anyhow::{Result, ensure};
use serde_json::{Value, json};

pub(crate) struct PresentationTimeline {
    tick: u32,
    running: bool,
    automatic_advance_count: u64,
    manual_step_count: u64,
    wrap_count: u64,
}

impl PresentationTimeline {
    pub(crate) fn new() -> Self {
        Self {
            tick: 0,
            running: true,
            automatic_advance_count: 0,
            manual_step_count: 0,
            wrap_count: 0,
        }
    }

    pub(crate) fn tick(&self) -> u32 {
        self.tick
    }

    pub(crate) fn status_json(&self) -> Value {
        json!({
            "revision": "source-duration-presentation-time-v1",
            "tick": self.tick,
            "phaseCount": animation_catalog::SAMPLE_COUNT,
            "framePeriod": animation_catalog::PRESENTATION_CLOCK_FRAME_PERIOD,
            "timeUnitsPerFrame": animation_catalog::PRESENTATION_TIME_UNITS_PER_FRAME,
            "timeUnitsPerSecond": animation_catalog::PRESENTATION_TIME_UNITS_PER_SECOND,
            "running": self.running,
            "automaticAdvanceCount": self.automatic_advance_count,
            "manualStepCount": self.manual_step_count,
            "wrapCount": self.wrap_count,
        })
    }

    pub(crate) fn pause(&mut self) {
        self.running = false;
    }

    pub(crate) fn resume(&mut self) {
        self.running = true;
    }

    pub(crate) fn set(&mut self, tick: u32) -> Result<()> {
        ensure!(
            !self.running,
            "presentation clock must be paused before setting time"
        );
        ensure!(
            tick < animation_catalog::PRESENTATION_CLOCK_FRAME_PERIOD,
            "presentation tick must be below {}",
            animation_catalog::PRESENTATION_CLOCK_FRAME_PERIOD
        );
        self.tick = tick;
        Ok(())
    }

    pub(crate) fn step(&mut self, ticks: u32) -> Result<()> {
        ensure!(
            !self.running,
            "presentation clock must be paused before stepping"
        );
        ensure!(
            (1..=4_096).contains(&ticks),
            "presentation step must contain 1..=4096 ticks"
        );
        self.advance(ticks);
        self.manual_step_count = self.manual_step_count.wrapping_add(u64::from(ticks));
        Ok(())
    }

    pub(crate) fn commit_canonical_frame(&mut self) {
        if self.running {
            self.advance(1);
            self.automatic_advance_count = self.automatic_advance_count.wrapping_add(1);
        }
    }

    fn advance(&mut self, ticks: u32) {
        let total = u64::from(self.tick) + u64::from(ticks);
        let period = u64::from(animation_catalog::PRESENTATION_CLOCK_FRAME_PERIOD);
        self.wrap_count = self.wrap_count.wrapping_add(total / period);
        self.tick = (total % period) as u32;
    }
}

#[cfg(test)]
#[path = "../../tests/private/timeline.rs"]
mod tests;
