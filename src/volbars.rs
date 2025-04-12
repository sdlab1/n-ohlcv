// volbars.rs
use eframe::egui;
use crate::settings::*; // Импортируем настройки для отступов

pub fn draw(ui: &mut egui::Ui, rect: egui::Rect, data_window: &crate::DataWindow) {
    let painter = ui.painter();
    let up_color = egui::Color32::from_rgb(100, 180, 100);
    let down_color = egui::Color32::from_rgb(180, 100, 100);

    // Применяем отступы только по бокам
    let chart_rect = rect.shrink(CHART_MARGIN);

    let volume_height = chart_rect.height() * data_window.volume_height_ratio;
    let vol_rect = egui::Rect::from_min_max(
        egui::pos2(chart_rect.min.x, chart_rect.max.y - volume_height),
        chart_rect.max,
    );

    let (start, end) = data_window.visible_range;
    if start >= end || end as usize > data_window.bars.len() {
        return;
    }

    let visible_slice = &data_window.bars[start as usize..end as usize];
    if visible_slice.is_empty() {
        return;
    }

    let max_volume = visible_slice.iter().map(|b| b.volume).fold(0.0, f64::max);
    if max_volume <= 0.0 {// volbars.rs
        use eframe::egui;
        use crate::settings::*; // Импортируем настройки для отступов
        
        pub fn draw(ui: &mut egui::Ui, rect: egui::Rect, data_window: &crate::DataWindow) {
            let painter = ui.painter();
            let up_color = egui::Color32::from_rgb(100, 180, 100);
            let down_color = egui::Color32::from_rgb(180, 100, 100);
        
            // Применяем отступы из settings.rs
            let mut chart_rect = rect.shrink(CHART_MARGIN);
            chart_rect.set_height(chart_rect.height() - CHART_BOTTOM_MARGIN);
        
            let volume_height = chart_rect.height() * data_window.volume_height_ratio;
            let vol_rect = egui::Rect::from_min_max(
                egui::pos2(chart_rect.min.x, chart_rect.max.y - volume_height),
                chart_rect.max,
            );
        
            let (start, end) = data_window.visible_range;
            if start >= end || end as usize > data_window.bars.len() {
                return;
            }
        
            let visible_slice = &data_window.bars[start as usize..end as usize];
            if visible_slice.is_empty() {
                return;
            }
        
            let max_volume = visible_slice.iter().map(|b| b.volume).fold(0.0, f64::max);
            if max_volume <= 0.0 {
                return;
            }
        
            let count = visible_slice.len() as f32;
            let bar_width = (vol_rect.width() / (count + 1.0)).min(5.0); // +1 для пространства справа
            let spacing = (bar_width * 0.2).min(1.0);
        
            for (i, bar) in visible_slice.iter().enumerate() {
                // Вычисляем x пропорционально, как в axes.rs
                let x_center = vol_rect.left() + ((i as f32 + 0.5) / count) * vol_rect.width();
                let x_left = x_center - bar_width / 2.0;
                let height = (bar.volume / max_volume) as f32 * vol_rect.height();
                let y_top = vol_rect.bottom() - height;
                let color = if bar.close >= bar.open { up_color } else { down_color };
        
                painter.rect_filled(
                    egui::Rect::from_min_max(
                        egui::pos2(x_left, y_top),
                        egui::pos2(x_left + bar_width, vol_rect.bottom()),
                    ),
                    0.0,
                    color,
                );
            }
        }
        return;
    }

    let count = visible_slice.len() as f32;
    let bar_width = (vol_rect.width() / count).min(5.0); // Убрали +1, чтобы растянуть до края

    for (i, bar) in visible_slice.iter().enumerate() {
        // Вычисляем x так, чтобы правый край последнего бара совпадал с rect.right()
        let x_right = vol_rect.left() + ((i as f32 + 1.0) / count) * vol_rect.width();
        let x_left = x_right - bar_width;
        let height = (bar.volume / max_volume) as f32 * vol_rect.height();
        let y_top = vol_rect.bottom() - height;
        let color = if bar.close >= bar.open { up_color } else { down_color };

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