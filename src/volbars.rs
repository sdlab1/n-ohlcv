// volbars.rs
use eframe::egui;

pub fn draw(ui: &mut egui::Ui, rect: egui::Rect, data_window: &crate::DataWindow) {
    let painter = ui.painter();
    let up_color = egui::Color32::from_rgb(100, 180, 100);   // Light green for rising volumes
    let down_color = egui::Color32::from_rgb(180, 100, 100); // Light red for falling volumes
    
    // Split the rectangle into price and volume areas
    let volume_height = rect.height() * data_window.volume_height_ratio;
    let volume_top = rect.max.y - volume_height;
    let vol_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, volume_top),
        rect.max,
    );
    
    // Define the visible data range
    let visible_bars = data_window.bars.len();
    if visible_bars == 0 {
        return; // Don't draw anything if there's no data
    }
    
    let start_idx = (visible_bars as f64 * data_window.visible_range.0) as usize;
    let end_idx = (visible_bars as f64 * data_window.visible_range.1) as usize;
    if start_idx >= visible_bars || end_idx <= start_idx {
        return; // Check for valid range
    }
    
    let end_idx = end_idx.min(visible_bars);
    let visible_slice = &data_window.bars[start_idx..end_idx];
    let visible_count = visible_slice.len();
    
    // Calculate maximum volume for scaling
    let max_volume = visible_slice.iter()
        .map(|bar| bar.volume)
        .fold(0.0, f64::max);
    
    if max_volume <= 0.0 {
        return; // Volume must be positive
    }
    
    // Calculate available width for all bars
    let total_width = vol_rect.width();
    let bar_width = (total_width / visible_count as f32).min(5.0);
    let spacing = (bar_width * 0.2).min(1.0);
    
    // Draw volume bars
    for (i, bar) in visible_slice.iter().enumerate() {
        let x_center = vol_rect.left() + (i as f32 + 0.5) * (bar_width + spacing);
        let x_left = x_center - bar_width / 2.0;
        
        // Scale volume (0-100% of volume area height)
        let vol_height = (bar.volume / max_volume) as f32 * vol_rect.height();
        let vol_y = vol_rect.bottom() - vol_height;
        
        // Determine color based on candle direction
        let color = if bar.close >= bar.open { up_color } else { down_color };
        
        // Draw volume bar
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(x_left, vol_y),
                egui::pos2(x_left + bar_width, vol_rect.bottom())
            ),
            0.0, // corner rounding
            color
        );
    }
    
    // Add maximum volume label
    let text_color = ui.style().visuals.text_color();
    let vol_text = if max_volume >= 1_000_000.0 {
        format!("{:.1}M", max_volume / 1_000_000.0)
    } else if max_volume >= 1_000.0 {
        format!("{:.1}K", max_volume / 1_000.0)
    } else {
        format!("{:.1}", max_volume)
    };
    
    painter.text(
        egui::pos2(vol_rect.left() + 5.0, vol_rect.top() + 2.0),
        egui::Align2::LEFT_TOP,
        format!("Vol: {}", vol_text),
        egui::FontId::proportional(10.0),
        text_color
    );
}