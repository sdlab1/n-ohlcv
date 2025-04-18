// crosshair.rs
use eframe::egui::Rect;
use chrono::{DateTime, Utc};
use crate::datawindow::DataWindow;

#[derive(Default)]
pub struct Crosshair {
    rect: Option<egui::Rect>, // Private field for chart area
}

impl Crosshair {
    pub fn get_bar_info(&self, mouse_pos: egui::Pos2, data_window: &DataWindow) -> Option<String> {
        let rect = match self.rect {
            Some(rect) => rect,
            None => return None, // No chart area defined
        };
        let (start, end) = data_window.visible_range;
        let visible_slice = &data_window.bars.get(start as usize..end as usize)?;
        if visible_slice.is_empty() {
            return None;
        }

        let chart_left = rect.left();
        let chart_width = rect.width();

        let adjusted_x = mouse_pos.x - data_window.pixel_offset;
        let normalized_x = (adjusted_x - chart_left) / chart_width;
        if normalized_x < 0.0 || normalized_x >= 1.0 {
            return None;
        }
        let index_float = normalized_x * visible_slice.len() as f32;
        let index = index_float.floor() as usize;
        if index >= visible_slice.len() {
            return None;
        }
        let bar = &visible_slice[index];

        let dt = DateTime::<Utc>::from_timestamp_millis(bar.time).unwrap_or(Utc::now());
        let volume_str = {
            let volume = bar.volume;
            let (base, unit) = if volume < 1000.0 {
                (1.0, "")
            } else if volume < 1_000_000.0 {
                (1000.0, "k")
            } else {
                (1_000_000.0, "m")
            };
            let value = volume / base;
            let decimals = if value < 10.0 {
                2
            } else if value < 100.0 {
                1
            } else {
                0
            };
            format!("{:.*}{}", decimals, value, unit)
        };
        Some(format!(
            "{} | o {:.2} h {:.2} l {:.2} c {:.2} v {}",
            dt.format("%H:%M"),
            bar.open,
            bar.high,
            bar.low,
            bar.close,
            volume_str
        ))
    }

    pub fn highlight_bar(&self, 
        ui: &mut egui::Ui, 
        rect: Rect,
        data_window: &DataWindow, 
        mouse_pos: egui::Pos2,
        scale_price: &impl Fn(f64) -> f32,) {
        let painter = ui.painter();
        let highlight_color = egui::Color32::from_rgb(100, 100, 100);

        let volume_height = rect.height() * data_window.volume_height_ratio;
        let price_rect = egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.max.y - volume_height));

        let (start, end) = data_window.visible_range;
        if start >= end || end as usize > data_window.bars.len() {
            return;
        }
        let visible_slice = &data_window.bars[start as usize..end as usize];
        if visible_slice.is_empty() {
            return;
        }

        let chart_left = price_rect.left();
        let chart_width = price_rect.width();
        let adjusted_x = mouse_pos.x - data_window.pixel_offset;
        let normalized_x = (adjusted_x - chart_left) / chart_width;
        if normalized_x < 0.0 || normalized_x >= 1.0 {
            return;
        }
        let index_float = normalized_x * visible_slice.len() as f32;
        let index = index_float.floor() as usize;
        if index >= visible_slice.len() {
            return;
        }
        let bar = &visible_slice[index];
        let count = visible_slice.len() as f32;
        let bar_width = (price_rect.width() / count).min(5.0);
        let i = index as f32;
        let x_right = price_rect.left() + ((i + 1.0) / count) * price_rect.width() + data_window.pixel_offset;
        let x_left = x_right - bar_width;
        let high_y = scale_price(bar.high);
        let low_y = scale_price(bar.low);

        let expanded_rect = egui::Rect::from_min_max(
            egui::pos2(x_left - 0.5, high_y - 0.5), // shift left top by 0.5px
            egui::pos2(x_right + 0.5, low_y + 0.5),  // shift right bottom by 0.5px
        );
        // draw filled rect
        painter.rect_filled(
            expanded_rect,
            1.0, // round angles
            highlight_color
        );
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, rect: Rect, _data_window: &DataWindow, mouse_pos: egui::Pos2) {
        self.rect = Some(rect);
        let painter = ui.painter();
        let color = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 100);

        painter.line_segment(
            [egui::pos2(mouse_pos.x, rect.top()), egui::pos2(mouse_pos.x, rect.bottom())],
            (1.0, color)
        );
        painter.line_segment(
            [egui::pos2(rect.left(), mouse_pos.y), egui::pos2(rect.right(), mouse_pos.y)],
            (1.0, color)
        );
    }
}