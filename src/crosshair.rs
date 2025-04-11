// crosshair.rs
use eframe::egui;
use chrono::{DateTime, Utc};

#[derive(Default)]
pub struct Crosshair {
    pub visible: bool,
}

impl Crosshair {
    pub fn get_bar_info(&self, mouse_pos: egui::Pos2, data_window: &crate::DataWindow) -> Option<String> {
        if data_window.bars.is_empty() {
            return None;
        }

        let visible_bars = data_window.bars.len();
        let start_idx = (visible_bars as f64 * data_window.visible_range.0) as usize;
        let end_idx = (visible_bars as f64 * data_window.visible_range.1) as usize;
        let visible_slice = &data_window.bars[start_idx..end_idx.min(visible_bars)];

        let bar_width = data_window.visible_range.1 - data_window.visible_range.0;
        let bar_index = ((mouse_pos.x / bar_width as f32) * visible_slice.len() as f32) as usize;
        let bar_index = bar_index.min(visible_slice.len().saturating_sub(1));

        visible_slice.get(bar_index).map(|bar| {
            let dt = DateTime::<Utc>::from_timestamp_millis(bar.time).unwrap();
            format!("Time: {} | O: {:.2} H: {:.2} L: {:.2} C: {:.2}",
                dt.format("%H:%M"),
                bar.open, bar.high, bar.low, bar.close)
        })
    }

    pub fn draw(&self, ui: &mut egui::Ui, rect: egui::Rect, data_window: &crate::DataWindow, mouse_pos: egui::Pos2) {
        if !self.visible || !rect.contains(mouse_pos) {
            return;
        }

        let painter = ui.painter();
        
        // Вертикальная линия
        painter.line_segment(
            [egui::pos2(mouse_pos.x, rect.top()), egui::pos2(mouse_pos.x, rect.bottom())],
            (1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)),
        );
        
        // Горизонтальная линия
        painter.line_segment(
            [egui::pos2(rect.left(), mouse_pos.y), egui::pos2(rect.right(), mouse_pos.y)],
            (1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100)),
        );
    }
}