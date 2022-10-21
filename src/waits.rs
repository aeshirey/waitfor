use crate::wait::Wait;
use std::time::{Duration, Instant};

/// Handles waiting for one or more [Wait]s.
pub enum Waits {
    Single(Wait),
    Or(Box<(Waits, Waits)>),
    And(Box<(Waits, Waits)>),
}

impl Waits {
    /// Returns true if this `Waits` is satisfied
    pub fn condition_met(&self) -> bool {
        match self {
            Waits::Single(u) => u.condition_met(),
            Waits::Or(cc) => cc.0.condition_met() || cc.1.condition_met(),
            Waits::And(cc) => cc.0.condition_met() && cc.1.condition_met(),
        }
    }

    /// Wait for the completion.
    pub fn wait(&self, interval: Duration) {
        loop {
            let start = Instant::now();
            if self.condition_met() {
                return;
            }

            let loop_time = start.elapsed();
            if interval > loop_time {
                std::thread::sleep(interval - loop_time);
            }
        }
    }
}

impl From<Wait> for Waits {
    fn from(w: Wait) -> Self {
        Waits::Single(w)
    }
}

impl std::ops::BitOr for Wait {
    type Output = Waits;

    fn bitor(self, other: Wait) -> Self::Output {
        Waits::Or(Box::new((self.into(), other.into())))
    }
}

impl std::ops::BitAnd for Wait {
    type Output = Waits;

    fn bitand(self, other: Wait) -> Self::Output {
        Waits::And(Box::new((self.into(), other.into())))
    }
}

impl std::ops::BitOr<Waits> for Wait {
    type Output = Waits;

    fn bitor(self, other: Waits) -> Self::Output {
        Waits::Or(Box::new((self.into(), other)))
    }
}

impl std::ops::BitAnd<Waits> for Wait {
    type Output = Waits;

    fn bitand(self, other: Waits) -> Self::Output {
        Waits::And(Box::new((self.into(), other)))
    }
}

impl std::ops::BitOr<Wait> for Waits {
    type Output = Self;

    fn bitor(self, other: Wait) -> Self {
        Waits::Or(Box::new((self, other.into())))
    }
}

impl std::ops::BitAnd<Wait> for Waits {
    type Output = Self;

    fn bitand(self, other: Wait) -> Self {
        Waits::And(Box::new((self, other.into())))
    }
}

impl std::ops::BitAnd for Waits {
    type Output = Self;

    fn bitand(self, other: Waits) -> Self {
        Waits::And(Box::new((self, other)))
    }
}

impl std::ops::BitOr for Waits {
    type Output = Self;

    fn bitor(self, other: Waits) -> Self {
        Waits::Or(Box::new((self, other)))
    }
}
