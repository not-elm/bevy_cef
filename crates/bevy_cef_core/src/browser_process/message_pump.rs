use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Maximum delay between message loop iterations (~30fps).
/// Following CEF's official cefclient pattern (`kMaxTimerDelay = 1000/30`).
const MAX_TIMER_DELAY_MS: u64 = 1000 / 30;

#[derive(Debug, Clone)]
pub struct MessagePumpChecker {
    timer: Arc<Mutex<Option<Instant>>>,
    last_work_time: Arc<Mutex<Instant>>,
}

impl Default for MessagePumpChecker {
    fn default() -> Self {
        Self {
            timer: Arc::default(),
            last_work_time: Arc::new(Mutex::new(Instant::now())),
        }
    }
}

impl MessagePumpChecker {
    /// Replaces the current timer with a new one.
    pub fn schedule(&self, delay_ms: i64) {
        let fire_time = if delay_ms <= 0 {
            Instant::now()
        } else {
            Instant::now() + Duration::from_millis(delay_ms as u64)
        };
        *self.timer.lock().unwrap() = Some(fire_time);
    }

    /// Returns true if cef_do_message_loop_work should be called.
    pub fn should_do_work(&self) -> bool {
        let mut timer = self.timer.lock().unwrap();
        let timer_finished = timer.is_some_and(|t| t <= Instant::now());
        if timer_finished {
            *timer = None;
        }

        let mut last_time = self.last_work_time.lock().unwrap();
        let elapsed = last_time.elapsed();

        // Call work if timer finished OR enough time elapsed (minimum frequency)
        if timer_finished || elapsed >= Duration::from_millis(MAX_TIMER_DELAY_MS) {
            *last_time = Instant::now();
            true
        } else {
            false
        }
    }
}
