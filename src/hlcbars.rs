//hlcbars.rs
use eframe::egui;

pub fn draw(ui: &mut egui::Ui, rect: egui::Rect, data_window: &crate::DataWindow, show_candles: bool) {
    let painter = ui.painter();
    let up_color = egui::Color32::from_rgb(0, 180, 0);
    let down_color = egui::Color32::from_rgb(180, 0, 0);
    let gray = egui::Color32::from_rgb(180, 180, 180);

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

    let (min_price, max_price) = visible_slice.iter().fold((f64::MAX, f64::MIN), |(min, max), bar| {
        (min.min(bar.low), max.max(bar.high))
    });

    let adjusted_min = min_price - (max_price - min_price) * 0.05;
    let adjusted_max = max_price + (max_price - min_price) * 0.05;
    let scale_price = |price: f64| -> f32 {
        price_rect.top() + ((adjusted_max - price) / (adjusted_max - adjusted_min)) as f32 * price_rect.height()
    };

    let count = visible_slice.len() as f32;
    let bar_width = (price_rect.width() / count).min(5.0); // Убрали +1, чтобы растянуть до края

    for (i, bar) in visible_slice.iter().enumerate() {
        // Вычисляем x так, чтобы правый край последнего бара совпадал с rect.right()
        let x_right = price_rect.left() + ((i as f32 + 1.0) / count) * price_rect.width() + data_window.pixel_offset;
        let x_left = x_right - bar_width;

        let high_y = scale_price(bar.high);
        let low_y = scale_price(bar.low);
        let close_y = scale_price(bar.close);
        let open_y = scale_price(bar.open);

        let color = if bar.close >= bar.open { up_color } else { down_color };

        if show_candles {
            let x_center = (x_left + x_right) / 2.0; // Центр для линий high-low
            painter.line_segment([egui::pos2(x_center, high_y), egui::pos2(x_center, low_y)], (1.0, color));
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(x_left, open_y.min(close_y)),
                    egui::pos2(x_right, open_y.max(close_y)),
                ),
                0.0,
                color,
            );
        } else {
            let x_center = (x_left + x_right) / 2.0;
            painter.line_segment([egui::pos2(x_center, high_y), egui::pos2(x_center, low_y)], (1.0, gray));
            let tick_width = bar_width * 0.6;
            painter.line_segment(
                [egui::pos2(x_center, close_y), egui::pos2(x_center + tick_width, close_y)],
                (1.0, gray),
            );
        }
    }
}