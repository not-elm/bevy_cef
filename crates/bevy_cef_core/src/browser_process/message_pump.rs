use bevy::prelude::Deref;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

/// Maximum delay between message loop iterations (~30fps).
/// Following CEF's official cefclient pattern (`kMaxTimerDelay = 1000/30`).
const MAX_TIMER_DELAY_MS: u64 = 1000 / 30;

#[repr(transparent)]
#[derive(Debug, Deref)]
pub struct MessageLoopWorkingReceiver(pub Receiver<MessageLoopTimer>);

#[derive(Debug)]
pub struct MessageLoopTimer(Instant);

impl MessageLoopTimer {
    pub fn new(delay_ms: i64) -> Self {
        let fire_time = if delay_ms <= 0 {
            Instant::now()
        } else {
            Instant::now() + Duration::from_millis(delay_ms as u64)
        };
        Self(fire_time)
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        self.0 <= Instant::now()
    }
}

#[derive(Debug, Deref)]
pub struct MessageLoopWorkingMaxDelayTimer(MessageLoopTimer);

impl Default for MessageLoopWorkingMaxDelayTimer {
    fn default() -> Self {
        Self(MessageLoopTimer::new(MAX_TIMER_DELAY_MS as i64))
    }
}
