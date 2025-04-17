use std::collections::VecDeque;
use std::time::{Duration, Instant};
use crate::settings; // Импортируем настройки

pub struct FrameInfo {
    frame_times: VecDeque<Duration>,
    last_status_update: Instant,
}

impl Default for FrameInfo {
    fn default() -> Self {
        Self {
            frame_times: VecDeque::new(),
            last_status_update: Instant::now(),
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

    pub fn mark_status_displayed(&mut self) {
        self.last_status_update = Instant::now();
    }
}