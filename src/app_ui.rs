// app_ui.rs
use eframe::{Frame, egui};
use crate::{TradingApp, axes, hlcbars, volbars, crosshair};

impl eframe::App for TradingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Сначала отображаем информацию о текущем баре (если есть)
            if let Some(pos) = ctx.pointer_hover_pos() {
                if let Some(bar_info) = self.crosshair.get_bar_info(pos, &self.data_window) {
                    ui.horizontal(|ui| {
                        ui.label(bar_info);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(format!("{} OHLCV Viewer", self.symbol));
                        });
                    });
                } else {
                    ui.heading(format!("{} OHLCV Viewer", self.symbol));
                }
            } else {
                ui.heading(format!("{} OHLCV Viewer", self.symbol));
            }
            ui.horizontal(|ui| {
                if ui.button(if self.show_candles { "bars" } else { "candles" }).clicked() {
                    self.show_candles = !self.show_candles;
                }
                
                for &tf in &[5, 15, 60, 240] {
                    if ui.button(format!("{}m", tf)).clicked() {
                        self.timeframe = tf;
                        self.update_data_window();
                    }
                }
            });

            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    self.zoom(0.1); // Zoom in
                }
                if ui.button("-").clicked() {
                    self.zoom(-0.1); // Zoom out
                }
            });

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                // Выделяем область с возможностью drag
                let response = ui.interact(
                    ui.available_rect_before_wrap(), 
                    ui.id().with("chart_area"),
                    egui::Sense::drag() // Только для перемещения
                );

                let rect = response.rect.shrink(2.0); // Небольшой отступ от краев

                // Рисуем компоненты графика
                axes::draw(ui, rect, &self.data_window);
                hlcbars::draw(ui, rect, &self.data_window, self.show_candles);
                volbars::draw(ui, rect, &self.data_window);

                // Перекрестие - работает при любом hover, даже без Sense::hover()
                if self.crosshair.visible {
                    if let Some(pos) = ctx.pointer_hover_pos() {
                        if rect.contains(pos) {
                            self.crosshair.draw(ui, rect, &self.data_window, pos);
                        }
                    }
                }


                if response.dragged() && response.drag_delta().x != 0.0 {

                }

                let scroll_delta = ctx.input(|i| i.raw_scroll_delta.y);
                if scroll_delta != 0.0 {
                    self.zoom(scroll_delta as f64 * 0.1);
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