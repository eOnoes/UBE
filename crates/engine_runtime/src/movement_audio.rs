use std::time::Duration;

/// Emits one footstep immediately and then at a bounded fixed-step cadence while active.
pub struct FootstepCadence {
    interval: Duration,
    until_next: Duration,
}

impl Default for FootstepCadence {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(420),
            until_next: Duration::ZERO,
        }
    }
}

impl FootstepCadence {
    pub fn update(&mut self, active: bool, elapsed: Duration) -> bool {
        if !active {
            self.until_next = Duration::ZERO;
            return false;
        }
        if self.until_next.is_zero() {
            self.until_next = self.interval;
            return true;
        }

        self.until_next = self.until_next.saturating_sub(elapsed);
        if self.until_next.is_zero() {
            self.until_next = self.interval;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STEP: Duration = Duration::from_millis(100);

    #[test]
    fn grounded_movement_emits_immediately_and_at_intervals() {
        let mut cadence = FootstepCadence::default();
        assert!(cadence.update(true, STEP));
        assert!(!cadence.update(true, STEP));
        assert!(!cadence.update(true, STEP));
        assert!(!cadence.update(true, STEP));
        assert!(!cadence.update(true, STEP));
        assert!(cadence.update(true, STEP));
    }

    #[test]
    fn idle_or_airborne_movement_is_silent_and_resets_the_next_step() {
        let mut cadence = FootstepCadence::default();
        assert!(!cadence.update(false, STEP));
        assert!(cadence.update(true, STEP));
        assert!(!cadence.update(true, STEP));
        assert!(!cadence.update(false, STEP));
        assert!(!cadence.update(false, STEP));
        assert!(cadence.update(true, STEP));
        assert!(!cadence.update(false, STEP));
        assert!(cadence.update(true, STEP));
    }

    #[test]
    fn airborne_transition_suppresses_and_restarts_movement_audio() {
        let mut cadence = FootstepCadence::default();
        assert!(cadence.update(true, STEP));
        assert!(!cadence.update(false, STEP));
        assert!(!cadence.update(false, STEP));
        assert!(cadence.update(true, STEP));
    }
}
