// gui.rs
use eframe::{Frame, egui};
use crate::{interactivegui::InteractiveGui, axes, hlcbars, volbars};
use crate::axes_util;
use crate::settings;
use std::time::{Duration, Instant};

impl eframe::App for InteractiveGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let frame_start_time = Instant::now();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                let measure_button_text = if self.measure_frame_time { "x" } else { "F" };
                    if ui.button(measure_button_text).clicked() {
                        self.measure_frame_time = !self.measure_frame_time;
                    }
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
                if self.measure_frame_time {
                    if let Some(avg_time) = self.frame_info.get_average_frame_time() {
                        let t_avg = avg_time.as_secs_f64() * 1000.0;
                            ui.label(format!("{:.2} ms ", t_avg));
                    }
                }
                if let Some(pos) = ctx.pointer_hover_pos() {
                    if let Some(bar_info) = self.crosshair.get_bar_info(pos, &self.data_window) {
                        ui.horizontal(|ui| {
                            ui.label(bar_info);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(format!("{} {}m", self.symbol, self.timeframe));
                            });
                        });
                    }
                }
            });

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let response = ui.interact(
                    ui.available_rect_before_wrap(), 
                    ui.id().with("chart_area"),
                    egui::Sense::drag()
                );

                let mut rect = response.rect;
                rect.set_height(rect.height() - settings::CHART_BOTTOM_MARGIN);
                // let me actually draw chart
                self.data_window.update_price_range_extrema();
                let volume_height = rect.height() * self.data_window.volume_height_ratio;
                let price_rect = egui::Rect::from_min_max(
                    rect.min,
                    egui::pos2(rect.max.x, rect.max.y - volume_height),
                );
                let scale_price = axes_util::create_scale_price_fn(&self.data_window, price_rect);
                // Crosshair handling
                if let Some(pos) = ctx.pointer_hover_pos() {
                    if rect.contains(pos) {
                        self.crosshair.draw(ui, rect, &self.data_window, pos);
                        self.crosshair.highlight_bar(ui,rect, &self.data_window, pos, &scale_price);
                    }
                }
                hlcbars::draw(ui, rect, &self.data_window, self.show_candles, &scale_price);
                volbars::draw(ui, rect, &self.data_window);
                axes::draw(ui, rect, &self.data_window, &scale_price);

                if response.dragged() && response.drag_delta().x != 0.0 {
                    let delta_x = response.drag_delta().x;
                    let bars_len = self.data_window.bars.len() as i64;
                    let (start_idx, end_idx) = self.data_window.visible_range;
                    let visible_count = end_idx - start_idx;
                    

                    // Проверяем, находимся ли мы у правого края и тянем влево
                    let at_right_edge = end_idx >= bars_len;
                    let dragging_left = delta_x < 0.0;
                    
                    if !(at_right_edge && dragging_left) {
                        // Обновляем смещение в пикселях
                        self.data_window.pixel_offset += delta_x;
                        
                        // Вычисляем сколько баров соответствует текущему смещению
                        let bar_width = (rect.width() / visible_count as f32) - settings::BAR_SPACING;
                        let bars_offset = (self.data_window.pixel_offset / (bar_width + settings::BAR_SPACING)).round() as i64;
                        
                        // Если смещение превысило ширину бара, обновляем visible_range
                        if bars_offset.abs() >= 1 {
                            let shift = bars_offset;
                            let new_start = (start_idx - shift).clamp(0, bars_len.saturating_sub(visible_count));
                            let new_end = (new_start + visible_count).min(bars_len);
                            
                            self.data_window.visible_range = (new_start, new_end);
                            self.data_window.pixel_offset -= shift as f32 * (bar_width + settings::BAR_SPACING);
                        }
                        ctx.request_repaint();
                    }
                }
                let scroll_delta = ctx.input(|i| i.raw_scroll_delta.y);
                if scroll_delta != 0.0 {
                    self.zoom(scroll_delta as f64 * 0.1);
                }
            });

            if self.status_messages_last_ts
            .map_or(false, |ts| 
                ts.elapsed() < Duration::from_secs(settings::STATUS_MESSAGE_HIDE_TIME)) {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for msg in &self.status_messages {
                        ui.label(msg);
                    }
                });
            }
        }); // Закрытие для egui::CentralPanel::default().show
        let frame_end_time = Instant::now();
        self.frame_info.record_frame_time(frame_end_time - frame_start_time);
        //ctx.request_repaint(); // Ensure continuous repainting
    } // Закрытие для impl eframe::App for TradingApp
} // Закрытие для impl TradingApp