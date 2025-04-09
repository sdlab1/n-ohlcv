// volbars.rs
use eframe::egui;

pub fn draw(ui: &mut egui::Ui, rect: egui::Rect, data_window: &crate::DataWindow) {
    let painter = ui.painter();
    let color = ui.visuals().text_color();
    let bar_width = 5.0;
    let spacing = 1.0;
    
    // Calculate volume area
    let vol_rect = rect.split_bottom_fixed(rect.height() * data_window.volume_height_ratio).1;
    
    // Calculate visible bars and max volume
    let visible_count = (vol_rect.width() / (bar_width + spacing)) as usize;
    let start_idx = (data_window.bars.len() as f64 * data_window.visible_range.0) as usize;
    let end_idx = std::cmp::min(start_idx + visible_count, data_window.bars.len());
    
    let max_vol = data_window.bars[start_idx..end_idx].iter()
        .map(|b| b.volume)
        .fold(f64::NEG_INFINITY, f64::max);
    
    for (i, bar) in data_window.bars[start_idx..end_idx].iter().enumerate() {
        let x = vol_rect.left() + i as f32 * (bar_width + spacing);
        let height = (bar.volume / max_vol) as f32 * vol_rect.height();
        
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(x, vol_rect.bottom() - height),
                egui::vec2(bar_width, height)
            ),
            0.0,
            color,
        );
    }
}