// hlcbars.rs (efficient rendering)
use eframe::egui;

pub fn draw(ui: &mut egui::Ui, rect: egui::Rect, data: &DataWindow) {
    let painter = ui.painter();
    let color = ui.visuals().text_color();
    let width = 5.0;
    let spacing = 1.0;
    
    let visible_bars = (rect.width() / (width + spacing)) as usize;
    let start_idx = (data.bars.len() as f64 * data.visible_range.0) as usize;
    let end_idx = (start_idx + visible_bars).min(data.bars.len());
    
    let price_range = data.bars[start_idx..end_idx].iter()
        .fold((f64::MAX, f64::MIN), |(min, max), b| {
            (min.min(b.low), max.max(b.high))
        });
    
    for (i, bar) in data.bars[start_idx..end_idx].iter().enumerate() {
        let x = rect.left() + i as f32 * (width + spacing);
        let scale = |price| rect.top() + (1.0 - (price - price_range.0) / (price_range.1 - price_range.0)) as f32 * rect.height();
        
        painter.line_segment(
            [egui::pos2(x + width/2.0, scale(bar.high)), 
             egui::pos2(x + width/2.0, scale(bar.low))],
            (1.0, color)
        );
    }
}