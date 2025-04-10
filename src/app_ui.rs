// app_ui.rs
use eframe::{Frame, egui};
use crate::{TradingApp, axes, hlcbars, volbars};

impl eframe::App for TradingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("{} OHLCV Viewer", self.symbol));

            ui.horizontal(|ui| {
                for &tf in &[1, 2, 3, 5, 10, 15, 30, 60, 240, 1440] {
                    if ui.button(format!("{}m", tf)).clicked() {
                        self.timeframe = tf;
                        self.update_data_window();
                    }
                }
            });

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (rect, _) = ui.allocate_exact_size(
                    ui.available_size(),
                    egui::Sense::drag(),
                );

                axes::draw(ui, rect, &self.data_window);
                hlcbars::draw(ui, rect, &self.data_window);
                volbars::draw(ui, rect, &self.data_window);

                let response = ui.interact(rect, ui.id().with("chart_area"), egui::Sense::click_and_drag());

                if response.dragged() {
                    let drag_delta = response.drag_delta().x / rect.width();
                    if drag_delta != 0.0 {
                        let range_width = self.data_window.visible_range.1 - self.data_window.visible_range.0;
                        let drag_amount = drag_delta as f64 * range_width;
                        
                        let mut new_start = (self.data_window.visible_range.0 - drag_amount)
                            .max(0.0)
                            .min(1.0 - range_width);
                        let mut new_end = new_start + range_width;
                        
                        if new_end > 1.0 {
                            new_end = 1.0;
                            new_start = new_end - range_width;
                        }
                        
                        self.data_window.visible_range = (new_start, new_end);
                        self.log_status(format!("Visible range: {:.2} - {:.2}", new_start, new_end));
                    }
                }

                if let Some(scroll_delta) = ctx.input(|i| {
                    if i.smooth_scroll_delta.y != 0.0 {
                        Some(i.smooth_scroll_delta.y * self.scroll_sensitivity)
                    } else {
                        None
                    }
                }) {
                    if scroll_delta != 0.0 {
                        let zoom_factor = 0.02;
                        let range_width = self.data_window.visible_range.1 - self.data_window.visible_range.0;
                        let new_width = if scroll_delta > 0.0 {
                            range_width * (1.0 - zoom_factor as f64).max(0.01)
                        } else {
                            range_width * (1.0 + zoom_factor as f64).min(1.0)
                        };
                        
                        let mouse_pos = ctx.input(|i| i.pointer.hover_pos()).unwrap_or(response.rect.center());
                        let rel_x = (mouse_pos.x - rect.left()) / rect.width();
                        let rel_pos = self.data_window.visible_range.0 + rel_x as f64 * range_width;
                        
                        let new_start = (rel_pos - rel_x as f64 * new_width).max(0.0);
                        let mut new_end = new_start + new_width;
                        
                        if new_end > 1.0 {
                            new_end = 1.0;
                            let new_start = new_end - new_width;
                            self.data_window.visible_range = (new_start, new_end);
                        } else {
                            self.data_window.visible_range = (new_start, new_end);
                        }
                        
                        self.log_status(format!("Zoom: {:.2} - {:.2}", new_start, new_end));
                    }
                }
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                for msg in &self.status_messages {
                    ui.label(msg);
                }
            });
        });
    }
}