use std::time::Duration;

pub struct ExponentialBackoff {
    attempts: u32,
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
}

impl ExponentialBackoff {
    pub fn new(max_attempts: u32, base_delay_secs: u64, max_delay_secs: u64) -> Self {
        Self {
            attempts: 0,
            max_attempts,
            base_delay: Duration::from_secs(base_delay_secs),
            max_delay: Duration::from_secs(max_delay_secs),
        }
    }

    pub fn reset(&mut self) {
        self.attempts = 0;
    }

    // For self healing errors we might try longer 
    pub fn set_max_attempts(&mut self, max_attempts: u32) {
        self.max_attempts = max_attempts;
    }

    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    pub fn next_delay(&mut self) -> Option<Duration> {
        if self.attempts >= self.max_attempts {
            return None;
        }

        let delay = self.base_delay * 2u32.pow(self.attempts);
        self.attempts += 1;

        Some(std::cmp::min(delay, self.max_delay))
    }
}
