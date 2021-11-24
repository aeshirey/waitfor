use crate::waitfor::Wait;
use std::time::{Duration, Instant};

/// Handles waiting for one or more [Wait]s.
pub struct WaitMultiple(pub(crate) Vec<Wait>);

impl WaitMultiple {
    /// Returns true if any of the [Wait]s have completed.
    pub fn condition_met(&self) -> bool {
        self.0.is_empty() || self.0.iter().any(|w| w.condition_met())
    }

    /// Wait for any one of the [Wait]s to complete.
    pub fn wait(&self, interval: Duration) {
        if !self.0.is_empty() {
            loop {
                let start = Instant::now();
                for waitfor in self.0.iter() {
                    if waitfor.condition_met() {
                        return;
                    }
                }

                let loop_time = start.elapsed();
                if interval > loop_time {
                    std::thread::sleep(interval - loop_time);
                }
            }
        }
    }
}

impl From<&[Wait]> for WaitMultiple {
    fn from(it: &[Wait]) -> Self {
        Self(it.to_vec())
    }
}

impl std::ops::BitOr<Wait> for WaitMultiple {
    type Output = Self;

    fn bitor(mut self, other: Wait) -> Self {
        self.0.push(other);
        self
    }
}
