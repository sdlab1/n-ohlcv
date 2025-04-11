use eframe::egui;

pub fn draw(ui: &mut egui::Ui, rect: egui::Rect, data_window: &crate::DataWindow, show_candles: bool) {
    let painter = ui.painter();
    let up_color = egui::Color32::from_rgb(0, 180, 0);   // Green for rising
    let down_color = egui::Color32::from_rgb(180, 0, 0); // Red for falling
    let gray = egui::Color32::from_rgb(180, 180, 180); // uni for bars
    let bar_width = 5.0;
    let spacing = 1.0;
    
    let volume_height = rect.height() * data_window.volume_height_ratio;
    let price_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.max.x, rect.max.y - volume_height),
    );
    
    let visible_bars = data_window.bars.len();
    if visible_bars == 0 {
        return;
    }
    
    let start_idx = (visible_bars as f64 * data_window.visible_range.0) as usize;
    let end_idx = (visible_bars as f64 * data_window.visible_range.1) as usize;
    if start_idx >= visible_bars || end_idx <= start_idx {
        return;
    }
    
    let end_idx = end_idx.min(visible_bars);
    let visible_slice = &data_window.bars[start_idx..end_idx];
    let visible_count = visible_slice.len();
    
    let (min_price, max_price) = visible_slice.iter()
        .fold((f64::MAX, f64::MIN), |(min, max), bar| {
            (min.min(bar.low), max.max(bar.high))
        });
    
    let price_range = max_price - min_price;
    let adjusted_min = min_price - price_range * 0.05;
    let adjusted_max = max_price + price_range * 0.05;
    
    let scale_price = |price: f64| -> f32 {
        price_rect.top() + ((adjusted_max - price) / (adjusted_max - adjusted_min)) as f32 * price_rect.height()
    };
    
    let total_width = price_rect.width();
    let bar_width = (total_width / visible_count as f32).min(bar_width);
    let spacing = (bar_width * 0.2).min(spacing);
    
    for (i, bar) in visible_slice.iter().enumerate() {
        let x_center = price_rect.left() + (i as f32 + 0.5) * (bar_width + spacing);
        let x_left = x_center - bar_width / 2.0;
        
        let high_y = scale_price(bar.high);
        let low_y = scale_price(bar.low);
        let close_y = scale_price(bar.close);
        
        let color = if bar.close >= bar.open { up_color } else { down_color };
        
        if show_candles {
            // Режим свечей
            let open_y = scale_price(bar.open);
            
            // Draw wick (high-low)
            painter.line_segment(
                [egui::pos2(x_center, high_y), egui::pos2(x_center, low_y)],
                (1.0, color)
            );
            
            // Draw candle body (open-close)
            let body_top = open_y.min(close_y);
            let body_bottom = open_y.max(close_y);
            
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(x_left, body_top),
                    egui::pos2(x_left + bar_width, body_bottom)
                ),
                0.0,
                color
            );
        } else {
            // Режим HLC баров
            // Draw main line (high-low)
            painter.line_segment(
                [egui::pos2(x_center, high_y), egui::pos2(x_center, low_y)],
                (1.0, gray)
            );
            
            // Draw closing tick (горизонтальная перекладина)
            let tick_width = bar_width * 0.6;
            painter.line_segment(
                [egui::pos2(x_center, close_y), 
                 egui::pos2(x_center + tick_width, close_y)],
                (1.0, gray)
            );
        }
    }
}