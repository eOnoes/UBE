use std::time::Duration;

const NANOS_PER_SECOND: u128 = 1_000_000_000;

/// The result of advancing a fixed-step accumulator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StepOutcome {
    /// Number of fixed simulation steps that should run now.
    pub steps: u32,
    /// Unconsumed time remaining after the fixed steps.
    pub remainder: Duration,
    /// Whole simulation steps discarded to enforce the per-frame bound.
    pub dropped: Duration,
}

/// Converts elapsed wall time into a bounded number of fixed simulation steps.
#[derive(Clone, Debug)]
pub struct FixedStep {
    step_ns: u128,
    accumulator_ns: u128,
    max_steps_per_frame: u32,
    total_dropped_ns: u128,
}

impl FixedStep {
    /// Creates an accumulator with an explicit step and catch-up bound.
    ///
    /// # Panics
    ///
    /// Panics if `step` is zero or `max_steps_per_frame` is zero. These are
    /// programming errors in the caller's timing configuration.
    pub fn new(step: Duration, max_steps_per_frame: u32) -> Self {
        assert!(!step.is_zero(), "fixed step duration must be positive");
        assert!(
            max_steps_per_frame > 0,
            "maximum steps per frame must be positive"
        );

        Self {
            step_ns: step.as_nanos(),
            accumulator_ns: 0,
            max_steps_per_frame,
            total_dropped_ns: 0,
        }
    }

    /// Adds elapsed time and returns the work that should be performed.
    ///
    /// At most `max_steps_per_frame` steps are returned. If more whole steps
    /// are pending, they are discarded while the fractional remainder is kept.
    pub fn advance(&mut self, elapsed: Duration) -> StepOutcome {
        self.accumulator_ns = self.accumulator_ns.saturating_add(elapsed.as_nanos());

        let available_steps = self.accumulator_ns / self.step_ns;
        let steps = available_steps.min(u128::from(self.max_steps_per_frame));
        let dropped_steps = available_steps - steps;
        let dropped_ns = dropped_steps * self.step_ns;

        self.accumulator_ns %= self.step_ns;
        self.total_dropped_ns = self.total_dropped_ns.saturating_add(dropped_ns);

        StepOutcome {
            steps: steps as u32,
            remainder: duration_from_nanos(self.accumulator_ns),
            dropped: duration_from_nanos(dropped_ns),
        }
    }

    /// Returns all simulation time discarded by this accumulator.
    pub fn total_dropped(&self) -> Duration {
        duration_from_nanos(self.total_dropped_ns)
    }
}

fn duration_from_nanos(nanos: u128) -> Duration {
    let seconds = nanos / NANOS_PER_SECOND;
    if seconds > u128::from(u64::MAX) {
        return Duration::MAX;
    }

    Duration::new(seconds as u64, (nanos % NANOS_PER_SECOND) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    #[test]
    fn zero_elapsed_time_produces_no_step() {
        let mut timing = FixedStep::new(ms(10), 5);

        let outcome = timing.advance(Duration::ZERO);

        assert_eq!(outcome.steps, 0);
        assert_eq!(outcome.remainder, Duration::ZERO);
        assert_eq!(outcome.dropped, Duration::ZERO);
    }

    #[test]
    fn fractional_steps_accumulate() {
        let mut timing = FixedStep::new(ms(10), 5);

        let first = timing.advance(ms(8));
        let second = timing.advance(ms(8));

        assert_eq!(first.steps, 0);
        assert_eq!(first.remainder, ms(8));
        assert_eq!(second.steps, 1);
        assert_eq!(second.remainder, ms(6));
        assert_eq!(second.dropped, Duration::ZERO);
    }

    #[test]
    fn multiple_whole_steps_are_returned() {
        let mut timing = FixedStep::new(ms(10), 5);

        let outcome = timing.advance(ms(35));

        assert_eq!(outcome.steps, 3);
        assert_eq!(outcome.remainder, ms(5));
        assert_eq!(outcome.dropped, Duration::ZERO);
    }

    #[test]
    fn long_frames_are_bounded_and_preserve_the_remainder() {
        let mut timing = FixedStep::new(ms(10), 3);

        let outcome = timing.advance(ms(105));

        assert_eq!(outcome.steps, 3);
        assert_eq!(outcome.remainder, ms(5));
        assert_eq!(outcome.dropped, ms(70));
        assert_eq!(timing.total_dropped(), ms(70));
    }

    #[test]
    fn dropped_time_accumulates_across_frames() {
        let mut timing = FixedStep::new(ms(10), 2);

        timing.advance(ms(45));
        let outcome = timing.advance(ms(35));

        assert_eq!(outcome.steps, 2);
        assert_eq!(outcome.remainder, Duration::ZERO);
        assert_eq!(outcome.dropped, ms(20));
        assert_eq!(timing.total_dropped(), ms(40));
    }

    #[test]
    #[should_panic(expected = "fixed step duration must be positive")]
    fn zero_step_is_rejected() {
        let _ = FixedStep::new(Duration::ZERO, 5);
    }

    #[test]
    #[should_panic(expected = "maximum steps per frame must be positive")]
    fn zero_step_limit_is_rejected() {
        let _ = FixedStep::new(ms(10), 0);
    }
}
