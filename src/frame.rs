use std::collections::VecDeque;
use std::time::{Duration, Instant};
use crate::settings; // Импортируем настройки

pub struct FrameInfo {
    frame_times: VecDeque<Duration>,
    last_status_update: Instant,
    status_message_lifetime: Duration,
}

impl Default for FrameInfo {
    fn default() -> Self {
        Self {
            frame_times: VecDeque::new(),
            last_status_update: Instant::now(),
            status_message_lifetime: Duration::from_secs(settings::STATUS_MESSAGE_LIFETIME_SECONDS),
        }
    }
}

impl FrameInfo {
    pub fn record_frame_time(&mut self, frame_time: Duration) {
        self.frame_times.push_back(frame_time);
        while self.frame_times.len() > settings::AVERAGE_FRAME_HISTORY_SIZE {
            self.frame_times.pop_front();
        }
    }

    pub fn get_average_frame_time(&self) -> Option<Duration> {
        if self.frame_times.is_empty() {
            return None;
        }
        let sum: Duration = self.frame_times.iter().sum();
        Some(sum / self.frame_times.len() as u32)
    }

    pub fn should_display_status(&self) -> bool {
        Instant::now() - self.last_status_update < self.status_message_lifetime
    }

    pub fn mark_status_displayed(&mut self) {
        self.last_status_update = Instant::now();
    }
}