use std::time::Duration;

/// Aggregate measurements for one completed reporting interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameStatistics {
    pub frames: u32,
    pub frames_per_second: f64,
    pub average_frame_ms: f64,
    pub maximum_frame_ms: f64,
}

/// Collects frame timings without depending on a renderer or native clock.
#[derive(Clone, Debug)]
pub struct FrameStats {
    interval: Duration,
    elapsed: Duration,
    frames: u32,
    maximum_frame: Duration,
}

impl FrameStats {
    /// Creates a collector with a nonzero reporting interval.
    ///
    /// # Panics
    ///
    /// Panics if `interval` is zero.
    pub fn new(interval: Duration) -> Self {
        assert!(
            !interval.is_zero(),
            "frame statistics interval must be positive"
        );
        Self {
            interval,
            elapsed: Duration::ZERO,
            frames: 0,
            maximum_frame: Duration::ZERO,
        }
    }

    /// Records a frame and returns statistics when the interval completes.
    pub fn record(&mut self, frame_time: Duration) -> Option<FrameStatistics> {
        self.elapsed = self.elapsed.saturating_add(frame_time);
        self.frames = self.frames.saturating_add(1);
        self.maximum_frame = self.maximum_frame.max(frame_time);
        if self.elapsed < self.interval {
            return None;
        }

        let seconds = self.elapsed.as_secs_f64();
        let statistics = FrameStatistics {
            frames: self.frames,
            frames_per_second: f64::from(self.frames) / seconds,
            average_frame_ms: self.elapsed.as_secs_f64() * 1_000.0 / f64::from(self.frames),
            maximum_frame_ms: self.maximum_frame.as_secs_f64() * 1_000.0,
        };
        self.elapsed = Duration::ZERO;
        self.frames = 0;
        self.maximum_frame = Duration::ZERO;
        Some(statistics)
    }
}

impl Default for FrameStats {
    fn default() -> Self {
        Self::new(Duration::from_secs(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_only_after_a_complete_interval() {
        let mut stats = FrameStats::new(Duration::from_secs(1));
        assert_eq!(stats.record(Duration::from_millis(250)), None);
        assert_eq!(stats.record(Duration::from_millis(250)), None);
        assert_eq!(stats.record(Duration::from_millis(250)), None);

        let report = stats.record(Duration::from_millis(250)).unwrap();

        assert_eq!(report.frames, 4);
        assert_eq!(report.frames_per_second, 4.0);
        assert_eq!(report.average_frame_ms, 250.0);
        assert_eq!(report.maximum_frame_ms, 250.0);
        assert_eq!(stats.record(Duration::from_millis(100)), None);
    }

    #[test]
    fn tracks_maximum_and_average_independently() {
        let mut stats = FrameStats::new(Duration::from_millis(100));

        assert_eq!(stats.record(Duration::from_millis(40)), None);
        let report = stats
            .record(Duration::from_millis(60))
            .expect("the second frame should complete the interval");

        assert_eq!(report.frames, 2);
        assert_eq!(report.average_frame_ms, 50.0);
        assert_eq!(report.maximum_frame_ms, 60.0);
    }

    #[test]
    #[should_panic(expected = "frame statistics interval must be positive")]
    fn zero_interval_is_rejected() {
        let _ = FrameStats::new(Duration::ZERO);
    }
}
