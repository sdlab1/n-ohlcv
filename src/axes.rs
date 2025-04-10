use eframe::egui;
use chrono::{DateTime, Utc};
use crate::fetch::PRICE_MULTIPLIER;

pub fn draw(ui: &mut egui::Ui, rect: egui::Rect, data_window: &crate::DataWindow) {
    let painter = ui.painter();
    let color = ui.visuals().text_color();
    let text_color = ui.style().visuals.text_color();
    let grid_color = egui::Color32::from_gray(60);

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
    
    let (min_price, max_price) = visible_slice.iter()
        .fold((f64::MAX, f64::MIN), |(min, max), bar| {
            (min.min(bar.low), max.max(bar.high))
        });
    
    let price_range = max_price - min_price;
    let adjusted_min = min_price - price_range * 0.05;
    let adjusted_max = max_price + price_range * 0.05;
    
    let volume_height = rect.height() * data_window.volume_height_ratio;
    let price_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.max.x, rect.max.y - volume_height),
    );
    
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()], 
        (1.0, color)
    );
    
    painter.line_segment(
        [rect.left_bottom(), rect.left_top()], 
        (1.0, color)
    );
    
    let price_steps = 5;
    for i in 0..=price_steps {
        let price = adjusted_min + (adjusted_max - adjusted_min) * (i as f64 / price_steps as f64);
        let y = price_rect.top() + ((adjusted_max - price) / (adjusted_max - adjusted_min)) as f32 * price_rect.height();
        
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            (0.5, grid_color)
        );
        
        let price_text = if PRICE_MULTIPLIER == 0 {
            format!("{:.0}", price)
        } else {
            format!("{:.PRICE_MULTIPLIER$}", price, PRICE_MULTIPLIER = PRICE_MULTIPLIER as usize)
        };
        
        painter.text(
            egui::pos2(rect.left() + 5.0, y - 10.0),
            egui::Align2::LEFT_BOTTOM,
            price_text,
            egui::FontId::proportional(10.0),
            text_color
        );
    }
    
    let time_steps = 6;
    let visible_count = end_idx - start_idx;
    
    for i in 0..=time_steps {
        let idx = start_idx + (visible_count * i / time_steps).min(visible_count - 1);
        if idx >= visible_bars {
            continue;
        }
        
        let bar_position = (idx - start_idx) as f32 / visible_count as f32;
        let x = rect.left() + bar_position * rect.width();
        
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            (0.5, grid_color)
        );
        
        let timestamp = data_window.bars[idx].time;
        let datetime: DateTime<Utc> = DateTime::from_timestamp_millis(timestamp).unwrap_or(Utc::now());
        let time_text = datetime.format("%H:%M\n%d/%m").to_string();
        
        painter.text(
            egui::pos2(x, rect.bottom() + 2.0),
            egui::Align2::CENTER_TOP,
            time_text,
            egui::FontId::proportional(10.0),
            text_color
        );
    }
}