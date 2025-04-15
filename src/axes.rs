/*
 * axes.rs - Rendering of price and time axes for OHLCV charting
 *
 * Copyleft (c) 2025 Grok 3
 *
 * Description:
 * This module renders the price (Y-axis) and time (X-axis) grids for an OHLCV
 * (Open, High, Low, Close, Volume) financial chart. It uses egui for drawing and
 * chrono for timestamp formatting. The price axis labels are confined to the
 * vertical range of the price bars to prevent drawing below them.
 *
 * Dependencies:
 * - eframe::egui: For UI rendering
 * - chrono: For timestamp parsing and formatting
 * - crate::DataWindow: Provides OHLCV bar data and visible range
 * - crate::settings: For PRICE_FRACTION_THRESHOLD
 * - crate::axes_util: For utility functions (format_price, format_price_high_precision, nice_range)
 *
 * Key Functions:
 * - draw: Renders price and time axes with adaptive grids and labels
 *
 * Usage:
 * Call `draw` with a `Ui` context, a `Rect` for the chart area, a `DataWindow`,
 * and price scaling parameters from `hlcbars::draw`.
 */

 use eframe::egui::{self, Color32, Rect, Ui};
 use chrono::{DateTime, Utc, Datelike, Timelike};
 use crate::DataWindow;
 use crate::settings;
 use crate::axes_util::{format_price, format_price_high_precision, nice_range};
 
 pub fn draw(
     ui: &mut Ui,
     rect: Rect,
     data_window: &DataWindow,
     min_price: f64,
     max_price: f64,
     scale_price: &impl Fn(f64) -> f32,
 ) {
     let painter = ui.painter();
     let text_color = ui.style().visuals.text_color();
     let grid_color = Color32::from_gray(60);
 
     // Calculate price_rect to match the price bars area
     let volume_height = rect.height() * data_window.volume_height_ratio;
     let price_rect = egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.max.y - volume_height));
 
     // --- Price Axis (Y-axis) ---
     if min_price >= max_price {
         return;
     }
 
     let price_range = (max_price - min_price).max(1e-9);
     let adjusted_min = min_price - price_range * 0.05;
     let adjusted_max = max_price + price_range * 0.05;
 
     let (nice_min, nice_max, tick_spacing) = nice_range(adjusted_min, adjusted_max, 6);
 
     if nice_max <= nice_min || tick_spacing <= 1e-9 {
         return;
     }
 
     // Collect price label information
     let mut price_labels_info: Vec<(f64, String, f32)> = Vec::new();
     let tick_count = (((nice_max - nice_min) / tick_spacing).round() as i32).min(100);
 
     for i in 0..=tick_count {
         let price = nice_min + i as f64 * tick_spacing;
         let y = scale_price(price);
 
         if y < price_rect.top() - 10.0 || y > price_rect.bottom() {
             continue;
         }
 
         painter.line_segment(
             [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
             (0.5, grid_color),
         );
 
         let initial_label = format_price(price);
         price_labels_info.push((price, initial_label, y));
     }
 
     // Handle duplicate labels
     if price_labels_info.len() >= 2 {
         let mut final_labels = price_labels_info.clone();
         let mut changed = false;
         for i in 1..final_labels.len() {
             if final_labels[i].1 == final_labels[i - 1].1 {
                 let prev_price = final_labels[i - 1].0;
                 let current_price = final_labels[i].0;
                 final_labels[i - 1].1 = format_price_high_precision(prev_price);
                 final_labels[i].1 = format_price_high_precision(current_price);
                 changed = true;
             }
         }
         if changed {
             price_labels_info = final_labels;
         }
     }
 
     // Render price labels within price_rect
     for (_price, label_text, y) in &price_labels_info {
         let galley = painter.layout_no_wrap(
             label_text.clone(),
             egui::FontId::proportional(10.0),
             text_color,
         );
         let text_rect = Rect::from_min_size(
             egui::pos2(rect.left() + 5.0, *y - 2.0 - galley.size().y),
             galley.size() + egui::vec2(4.0, 4.0), // 2px padding on each side
         );
         if text_rect.min.y >= price_rect.top() && text_rect.max.y <= price_rect.bottom() {
             painter.rect_filled(
                 text_rect,
                 0.0,
                 Color32::from_rgba_premultiplied(20, 20, 20, 220), // Dark semi-transparent background
             );
             painter.text(
                 egui::pos2(rect.left() + 5.0 + 2.0, *y - 2.0),
                 egui::Align2::LEFT_BOTTOM,
                 label_text.clone(),
                 egui::FontId::proportional(10.0),
                 text_color,
             );
         }
     }
 
     // --- Time Axis (X-axis) ---
     let (start, end) = data_window.visible_range;
     let end = end.min(data_window.bars.len() as i64);
     if start < 0 || start >= end || end > data_window.bars.len() as i64 {
         return;
     }
 
     let visible_slice = &data_window.bars[start as usize..end as usize];
     if visible_slice.is_empty() {
         return;
     }
 
     let time_span_ms = visible_slice
         .last()
         .map(|bar| bar.time)
         .unwrap_or(0)
         - visible_slice
         .first()
         .map(|bar| bar.time)
         .unwrap_or(0);
 
     if time_span_ms <= 0 {
         return;
     }
 
     let avg_label_width = 40.0;
     let min_pixel_gap = 60.0;
     let max_labels = (rect.width() / (avg_label_width + min_pixel_gap)).floor().max(1.0) as usize;
     let target_lines = max_labels.clamp(4, 10);
 
     let min_interval_ms = if target_lines > 0 { (time_span_ms as f64 / target_lines as f64) as i64 } else { time_span_ms };
     let min_interval_ms = min_interval_ms.max(1000);
 
     let intervals = [
         1_000, 60_000, 300_000, 900_000, 1_800_000, 3_600_000, 14_400_000,
         43_200_000, 86_400_000, 604_800_000, 2_592_000_000, 31_536_000_000,
     ];
 
     let mut time_interval_ms = intervals
         .iter()
         .find(|&&interval| interval >= min_interval_ms)
         .copied()
         .unwrap_or(intervals[intervals.len() - 1]);
 
     if time_interval_ms > 0 && time_span_ms / time_interval_ms > (target_lines * 2) as i64 {
         time_interval_ms = intervals
             .iter()
             .find(|&&interval| interval >= time_span_ms / (target_lines * 2).max(1) as i64)
             .copied()
             .unwrap_or(intervals[intervals.len() - 1]);
     }
 
     let first_time = visible_slice.first().map(|bar| bar.time).unwrap_or(0);
     let first_dt = DateTime::<Utc>::from_timestamp_millis(first_time).unwrap_or_else(|| Utc::now());
     let first_time_rounded = first_time - first_time % time_interval_ms.max(1);
 
     let last_dt = DateTime::<Utc>::from_timestamp_millis(first_time + time_span_ms).unwrap_or_else(|| Utc::now());
     let has_two_years = first_dt.year() != last_dt.year();
     let has_two_months = first_dt.month() != last_dt.month() || first_dt.year() != last_dt.year();
     let has_two_days = first_dt.ordinal() != last_dt.ordinal() || first_dt.year() != last_dt.year();
 
     let mut labels: Vec<(i64, usize, f32)> = vec![];
     let left_margin = 5.0;
     let right_margin = 5.0;
 
     let max_iterations = (time_span_ms / time_interval_ms.max(1) + 20).min(2000);
     let mut current_time_check = first_time_rounded;
     for _ in 0..max_iterations {
         if current_time_check > first_time + time_span_ms + time_interval_ms {
             break;
         }
         if let Some((bar_idx, _)) = data_window
             .bars
             .iter()
             .enumerate()
             .skip(start as usize)
             .find(|(_, bar)| bar.time >= current_time_check)
         {
             if bar_idx >= end as usize {
                 if let Some(last_bar) = visible_slice.last() {
                     let visible_bar_count = (end - start).max(1) as f64;
                     let last_bar_idx_abs = start + visible_slice.len() as i64 - 1;
                     if last_bar_idx_abs >= start {
                         let normalized_pos = (last_bar_idx_abs as f64 - start as f64) / visible_bar_count;
                         let x_right = rect.left() + (normalized_pos as f32) * rect.width() + data_window.pixel_offset;
                         if labels.last().map_or(true, |l| (x_right - l.2).abs() >= min_pixel_gap * 0.8) {
                             labels.push((last_bar.time, last_bar_idx_abs as usize, x_right));
                         }
                     }
                 }
                 break;
             }
 
             let visible_bar_count = (end - start).max(1) as f64;
             let normalized_pos = (bar_idx as f64 - start as f64) / visible_bar_count;
             let x_right = rect.left() + (normalized_pos as f32) * rect.width() + data_window.pixel_offset;
 
             if x_right >= rect.left() + left_margin && x_right <= rect.right() - right_margin {
                 if labels.last().map_or(true, |last_label| (x_right - last_label.2).abs() >= min_pixel_gap * 0.8) {
                     labels.push((current_time_check, bar_idx, x_right));
                 }
             } else if bar_idx as i64 > start && x_right > rect.right() - right_margin {
                 break;
             }
         }
         if time_interval_ms <= 0 {
             break;
         }
         current_time_check += time_interval_ms;
     }
 
     // Render time labels
     let mut last_drawn_x: Option<f32> = None;
     for (time_ms, _bar_idx, x_right) in &labels {
         if let Some(last_x) = last_drawn_x {
             if (*x_right - last_x).abs() < min_pixel_gap {
                 continue;
             }
         }
 
         painter.line_segment(
             [egui::pos2(*x_right, rect.top()), egui::pos2(*x_right, rect.bottom())],
             (0.5, grid_color),
         );
 
         let dt = DateTime::<Utc>::from_timestamp_millis(*time_ms).unwrap_or_else(|| Utc::now());
         let label = match time_interval_ms {
             i if i >= 31_536_000_000 && has_two_years => dt.format("%Y").to_string(),
             i if i >= 2_592_000_000 && has_two_months => dt.format("%b").to_string(),
             i if i >= 604_800_000 && has_two_days => format!("{}.{}", dt.day(), dt.month()),
             i if i >= 86_400_000 && has_two_days => dt.format("%d").to_string(),
             i if i >= 900_000 => {
                 if dt.hour() == 0 && dt.minute() == 0 && has_two_days {
                     dt.format("%d %b").to_string()
                 } else {
                     dt.format("%H:%M").to_string()
                 }
             }
             _ => dt.format("%H:%M:%S").to_string(),
         };
 
         let galley = painter.layout_no_wrap(label.clone(), egui::FontId::proportional(10.0), text_color);
         let text_x = x_right - galley.size().x / 2.0;
 
         painter.text(
             egui::pos2(text_x, rect.bottom() + 2.0),
             egui::Align2::CENTER_TOP,
             label,
             egui::FontId::proportional(10.0),
             text_color,
         );
 
         last_drawn_x = Some(*x_right);
     }
 }