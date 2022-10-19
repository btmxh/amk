use std::time::{Duration, SystemTime};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SystemInstant(SystemTime);

pub trait SubtractableInstant: Sized + Copy + std::ops::Sub<Self, Output = Duration> {}

pub trait ConstructibleInstant: SubtractableInstant + std::ops::Sub<Duration, Output = Self> {}

impl SubtractableInstant for std::time::Instant {}
impl ConstructibleInstant for std::time::Instant {}
impl std::ops::Sub<SystemInstant> for SystemInstant {
    type Output = Duration;

    fn sub(self, rhs: SystemInstant) -> Self::Output {
        self.0.duration_since(rhs.0).unwrap()
    }
}
impl SubtractableInstant for SystemInstant {}

pub trait Clock {
    type Instant;
    fn now(&self) -> Self::Instant;
}

#[derive(Default)]
pub struct SystemClock;
impl Clock for SystemClock {
    type Instant = SystemInstant;
    fn now(&self) -> SystemInstant {
        SystemInstant(SystemTime::now())
    }
}

#[derive(Default)]
pub struct InstantClock;
impl Clock for InstantClock {
    type Instant = std::time::Instant;
    fn now(&self) -> Self::Instant {
        std::time::Instant::now()
    }
}
