// app_ui.rs
use eframe::{Frame, egui};
use crate::{TradingApp, axes, hlcbars, volbars};
use crate::settings::*;


impl eframe::App for TradingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // --- ЕДИНЫЙ ГОРИЗОНТАЛЬНЫЙ РЯД: Инфо + Кнопки ---
            ui.horizontal(|ui| {
            //LIMITS RENDER
            //ctx.request_repaint_after(std::time::Duration::from_millis(1000));
            ui.horizontal(|ui| {
                if ui.button(if self.show_candles { "bars" } else { "candles" }).clicked() {
                    self.show_candles = !self.show_candles;
                }
                
                for &tf in &[5, 15, 60, 240] {
                    if ui.button(format!("{}", tf)).clicked() {
                        self.timeframe = tf;
                        self.update_data_window();
                    }
                }
                if ui.button("+").clicked() {
                    self.zoom(0.1); // Zoom in
                }
                if ui.button("-").clicked() {
                    self.zoom(-0.1); // Zoom out
                }
                });
                ui.add_space(15.0);    
                // bar info
                if let Some(pos) = ctx.pointer_hover_pos() {
                    if let Some(bar_info) = self.crosshair.get_bar_info(pos, &self.data_window) {
                        ui.horizontal(|ui| {
                            ui.label(bar_info);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(format!("{} OHLCV Viewer", self.symbol));
                            });
                        });
                    }
                }
            });

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                // Выделяем область с возможностью drag
                let response = ui.interact(
                    ui.available_rect_before_wrap(), 
                    ui.id().with("chart_area"),
                    egui::Sense::drag() // Только для перемещения
                );

                let mut rect = response.rect;
                //rect = rect.shrink(CHART_MARGIN); // Отступы по всем сторонам
                rect.set_height(rect.height() - CHART_BOTTOM_MARGIN); // Уменьшаем высоту для отступа снизу

                /*static mut UPDATE_COUNT: u32 = 0;
                unsafe {
                    UPDATE_COUNT += 1;
                    println!("Update call: {}", UPDATE_COUNT);
                }*/
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
                    let delta_x = response.drag_delta().x;
                    let bars_len = self.data_window.bars.len() as i64;
                    let (start_idx, end_idx) = self.data_window.visible_range;
                    let visible_count = end_idx - start_idx;
                
                    // Учитываем масштаб графика
                    let bars_per_pixel = visible_count as f32 / rect.width();
                    let sensitivity = DRAG_SENSITIVITY * 2.0; // Увеличиваем чувствительность
                    let shift = (delta_x * bars_per_pixel * sensitivity as f32).round() as i64;
                
                    let new_start = (start_idx - shift).clamp(0, bars_len.saturating_sub(visible_count));
                    let new_end = (new_start + visible_count).min(bars_len);
                
                    /*println!(
                        "Drag: delta_x = {}, bars_per_pixel = {}, sensitivity = {}, shift = {}, old_range = ({}, {}), new_range = ({}, {}), bars_len = {}",
                        delta_x, bars_per_pixel, sensitivity, shift, start_idx, end_idx, new_start, new_end, bars_len
                    );*/
                
                    self.data_window.visible_range = (new_start, new_end);
                    ctx.request_repaint();
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