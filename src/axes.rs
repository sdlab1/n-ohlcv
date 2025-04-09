// axes.rs
use eframe::egui;

pub fn draw(ui: &mut egui::Ui, rect: egui::Rect, data_window: &crate::DataWindow) {
    let painter = ui.painter();
    let color = ui.visuals().text_color();
    
    // X axis (time)
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()], 
        (1.0, color)
    );
    
    // Y axis (price)
    painter.line_segment(
        [rect.left_bottom(), rect.left_top()], 
        (1.0, color)
    );
    
    // TODO: Add labels and grid lines based on visible range
}