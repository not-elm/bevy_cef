use bevy::prelude::Message;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

//TODO: Rename this struct
#[derive(Default, Debug, Clone)]
pub struct MessagePumpChecker(Arc<Mutex<Vec<MessagePumpTimer>>>);

impl MessagePumpChecker {
    pub fn schedule(&self, delay_ms: i64) {
        let mut guard = self.0.lock().unwrap();
        guard.push(MessagePumpTimer::new(delay_ms));
    }

    pub fn should_iterate_message_loop_count(&self) -> usize {
        let mut guard = self.0.lock().unwrap();
        let total = guard.len();
        guard.retain(|timer| !timer.finished());
        total - guard.len()
    }
}

#[repr(transparent)]
#[derive(Debug)]
struct MessagePumpTimer(Instant);

impl MessagePumpTimer {
    pub fn new(delay_ms: i64) -> Self {
        let instant = if delay_ms < 0 {
            Instant::now()
        } else {
            Instant::now() + std::time::Duration::from_millis(delay_ms as u64)
        };
        Self(instant)
    }

    pub fn finished(&self) -> bool {
        self.0 <= Instant::now()
    }
}
