use std::time::{Duration, Instant};

pub struct TimeIter {
    start_time: Instant,
    duration: Duration,
    stop: bool,
}

pub fn repeat_for(duration: Duration) -> TimeIter {
    TimeIter {
        start_time: Instant::now(),
        duration: duration,
        stop: false,
    }
}

impl Iterator for TimeIter {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        let elapsed = self.start_time.elapsed();

        if self.stop {
            return None;
        }

        if elapsed > self.duration {
            self.stop = true;
            Some(elapsed)
        } else {
            Some(elapsed)
        }
    }
}
