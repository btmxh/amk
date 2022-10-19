use std::time::Duration;

use super::clock::{Clock, SubtractableInstant, InstantClock};

pub trait ClockSync {
    fn sync(&mut self, frequency: f64) {
        if frequency > 0.0 {
            self.sync_impl(frequency)
        }
    }

    fn sync_impl(&mut self, frequency: f64);
}

pub struct OFClockSync<I: SubtractableInstant> {
    clock: Box<dyn Clock<Instant = I>>,
    current_time: I,
    last_frame_time: I,
    sleep_error: f64,
}

impl<I: SubtractableInstant> ClockSync for OFClockSync<I> {
    fn sync_impl(&mut self, frequency: f64) {
        const MIN_LAG: f64 = -1.0 / 30.0;
        self.last_frame_time = self.current_time;
        self.current_time = self.clock.now();

        let excess_time = 1.0 / frequency - (self.current_time - self.last_frame_time).as_secs_f64();
        let before = self.current_time;
        let sleep_time = (excess_time + self.sleep_error).max(0.0);

        std::thread::sleep(Duration::from_secs_f64(sleep_time));
        self.current_time = self.clock.now();
        let time_slept = self.current_time - before;

        self.sleep_error += excess_time - time_slept.as_secs_f64();
        self.sleep_error = self.sleep_error.max(MIN_LAG);
    }
}

impl<I: SubtractableInstant> OFClockSync<I> {
    pub fn new(clock: Box<dyn Clock<Instant = I>>) -> Self {
        Self {
            current_time: clock.now(),
            last_frame_time: clock.now(),
            sleep_error: 0.0,
            clock,
        }
    }
}

pub fn new_clock_sync() -> Box<dyn ClockSync> {
    Box::new(OFClockSync::new(Box::new(InstantClock)))
}
