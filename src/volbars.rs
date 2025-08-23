// volbars.rs
use crate::datawindow::DataWindow;
use crate::drawing_util;
use eframe::egui;

pub fn draw(ui: &mut egui::Ui, rect: egui::Rect, data_window: &mut DataWindow) {
    let painter = ui.painter();
    let up_color = egui::Color32::from_rgb(100, 180, 100);
    let down_color = egui::Color32::from_rgb(180, 100, 100);

    let volume_height = rect.height() * data_window.volume_height_ratio;
    let vol_rect =
        egui::Rect::from_min_max(egui::pos2(rect.min.x, rect.max.y - volume_height), rect.max);

    let (start, end) = data_window.visible_range;
    if start >= end || end as usize > data_window.bars.len() {
        return;
    }

    let max_volume = data_window.get_max_volume();
    if max_volume <= 0.0 {
        return;
    }

    let visible_slice = &data_window.bars[start as usize..end as usize];
    if visible_slice.is_empty() {
        return;
    }

    let visible_count = visible_slice.len();

    for (i, bar) in visible_slice.iter().enumerate() {
        let (x_left, x_right) = drawing_util::calculate_bar_x_position(
            i,
            visible_count,
            vol_rect, // Используем vol_rect для правильного масштабирования
            data_window.pixel_offset,
        );

        let height = (bar.volume / max_volume) as f32 * vol_rect.height();
        let y_top = vol_rect.bottom() - height;
        let color = if bar.close >= bar.open {
            up_color
        } else {
            down_color
        };

        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(x_left, y_top),
                egui::pos2(x_right, vol_rect.bottom()),
            ),
            0.0,
            color,
        );
    }
}
