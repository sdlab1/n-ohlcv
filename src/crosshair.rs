// crosshair.rs
use eframe::egui;
use chrono::{DateTime, Utc};

#[derive(Default)]
pub struct Crosshair {
    pub visible: bool,
}

impl Crosshair {
    pub fn get_bar_info(&self, mouse_pos: egui::Pos2, data_window: &crate::DataWindow) -> Option<String> {
        let (start, end) = data_window.visible_range;
        let visible_slice = &data_window.bars.get(start as usize..end as usize)?;

        if visible_slice.is_empty() {
            return None;
        }

        let index = ((mouse_pos.x / 10.0) as usize).min(visible_slice.len().saturating_sub(1)); // approximation
        visible_slice.get(index).map(|bar| {
            let dt = DateTime::<Utc>::from_timestamp_millis(bar.time).unwrap_or(Utc::now());
            format!("{} | O:{:.2} H:{:.2} L:{:.2} C:{:.2}", dt.format("%H:%M"), bar.open, bar.high, bar.low, bar.close)
        })
    }

    pub fn draw(&self, ui: &mut egui::Ui, rect: egui::Rect, _data_window: &crate::DataWindow, mouse_pos: egui::Pos2) {
        if !self.visible || !rect.contains(mouse_pos) {
            return;
        }

        let painter = ui.painter();
        let color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100);

        painter.line_segment([egui::pos2(mouse_pos.x, rect.top()), egui::pos2(mouse_pos.x, rect.bottom())], (1.0, color));
        painter.line_segment([egui::pos2(rect.left(), mouse_pos.y), egui::pos2(rect.right(), mouse_pos.y)], (1.0, color));
    }
}
