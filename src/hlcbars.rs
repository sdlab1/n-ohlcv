pub fn draw(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    data_window: &crate::DataWindow,
    show_candles: bool,
    scale_price: &impl Fn(f64) -> f32,
) {
    let painter = ui.painter();
    let pixels_per_point = ui.ctx().pixels_per_point();
    let pixel_offset = data_window.pixel_offset.floor();
    
    // Функция для выравнивания 1px линий
    let align_px = |x: f32| (x * pixels_per_point).floor() / pixels_per_point + 0.5 / pixels_per_point;

    let up_color = egui::Color32::from_rgb(0, 180, 0);
    let down_color = egui::Color32::from_rgb(180, 0, 0);
    let gray = egui::Color32::from_rgb(180, 180, 180);

    let (start, end) = data_window.visible_range;
    if start >= end || end as usize > data_window.bars.len() {
        return;
    }

    let count = (end - start) as f32;
    let bar_width = align_px((rect.width() / count).min(5.0));

    // Прямой доступ к барам через индексацию
    for i in start..end {
        let bar = &data_window.bars[i as usize];
        let visible_index = (i - start) as f32;

        let x_right = align_px(rect.left() + ((visible_index + 1.0) / count * rect.width()) + pixel_offset);
        let x_left = align_px(x_right - bar_width);

        let high_y = align_px(scale_price(bar.high));
        let low_y = align_px(scale_price(bar.low));
        let close_y = align_px(scale_price(bar.close));
        let open_y = align_px(scale_price(bar.open));

        let color = if bar.close >= bar.open { up_color } else { down_color };

        if show_candles {
            let x_center = align_px((x_left + x_right) / 2.0);
            painter.line_segment(
                [egui::pos2(x_center, high_y), egui::pos2(x_center, low_y)], 
                (1.0, color)
            );
            
            // Для прямоугольников используем то же выравнивание
            let rect_min_x = align_px(x_left);
            let rect_max_x = align_px(x_right);
            let rect_min_y = align_px(open_y.min(close_y));
            let rect_max_y = align_px(open_y.max(close_y));
            
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(rect_min_x, rect_min_y),
                    egui::pos2(rect_max_x, rect_max_y),
                ),
                0.0,
                color,
            );
        } else {
            let x_center = align_px((x_left + x_right) / 2.0);
            painter.line_segment(
                [egui::pos2(x_center, high_y), egui::pos2(x_center, low_y)], 
                (1.0, gray)
            );
            
            let tick_width = align_px(bar_width * 0.6);
            let tick_end = align_px(x_center + tick_width);
            painter.line_segment(
                [egui::pos2(x_center, close_y), egui::pos2(tick_end, close_y)],
                (1.0, gray),
            );
        }
    }
}