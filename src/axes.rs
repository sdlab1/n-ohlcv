/*
 * axes.rs - Rendering of price and time axes for OHLCV charting
 *
 * Copyleft (c) 2025 Grok 3
 *
 * Description:
 * This module renders the price (Y-axis) and time (X-axis) grids for an OHLCV
 * (Open, High, Low, Close, Volume) financial chart. It uses egui for drawing and
 * chrono for timestamp formatting. The time axis dynamically adjusts label intervals
 * (years, months, days, 12 hours, hours, minutes, seconds) based on the visible range,
 * ensuring 6-14 labels with readable spacing. The price axis uses "nice" tick spacing
 * with 5-7 ticks and k/m suffixes for large values.
 *
 * Dependencies:
 * - eframe::egui: For UI rendering
 * - chrono: For timestamp parsing and formatting
 * - crate::data_window: Provides OHLCV bar data and visible range
 * - crate::settings: For PRICE_FRACTION_THRESHOLD
 *
 * Key Functions:
 * - draw: Renders price and time axes with adaptive grids and labels
 * - nice_range: Calculates "nice" price boundaries and tick spacing
 * - format_price: Formats price values with k/m suffixes
 *
 * Usage:
 * Call `draw` with a `Ui` context, a `Rect` for the chart area, and a `DataWindow`
 * containing OHLCV data. The function computes grids, draws lines, and places labels.
 *
 * Debugging Tips:
 * - No labels on right:
 *   Check: println!("Drawing text: label = {}, x_right = {}, text_x = {}", label, x_right, text_x);
 *   Ensure `x_right` <= `rect.right() + 50.0` and `text_x` fits in `clip_rect`.
 * - Fewer than 6 labels:
 *   Check: println!("Final labels: {}, time_interval_ms: {}", labels.len(), time_interval_ms);
 *   Extend `time_span_ms` with `+ 5 * time_interval_ms`.
 * - Labels overlap:
 *   Increase: let min_pixel_gap = 80.0;
 * - 12-hour labels missing:
 *   Check: println!("Switch to 12h: labels = {}, interval = {}", labels.len(), time_interval_ms);
 *   Ensure `labels.len() <= 7`.
 * - Price labels unreadable:
 *   Adjust: egui::FontId::proportional(12.0);
 *
 * Notes:
 * - Time labels adapt to zoom: years/months for wide ranges, days/hours for narrow.
 * - Price axis adjusts to min/max prices with 5% padding.
 * - Ensure `DataWindow` has valid bars to avoid empty ranges.
 *
 * Author: Grok 3
 * Date: April 12 til 13th 3a.m., 2025
 */

 use eframe::egui::{self, Color32, Rect, Ui};
 use chrono::{DateTime, Utc, Datelike, Timelike};
 use crate::DataWindow;
 use crate::settings;
 
 pub fn draw(ui: &mut Ui, rect: Rect, data_window: &DataWindow) {
     let painter = ui.painter();
     //println!("Clip rect: {:?}", painter.clip_rect());
 
     let text_color = ui.style().visuals.text_color();
     let grid_color = Color32::from_gray(60);
 
     // Проверяем данные
     let (start, end) = data_window.visible_range;
     let end = end.min(data_window.bars.len() as i64);
     if start >= end || end as usize > data_window.bars.len() {
         //println!("Invalid range: start = {}, end = {}, bars_len = {}", start, end, data_window.bars.len());
         return;
     }
 
     let visible_slice = &data_window.bars[start as usize..end as usize];
     if visible_slice.is_empty() {
         //println!("Empty visible slice");
         return;
     }
 
     /*println!(
         "Axes: visible_range = ({}, {}), bars_len = {}, rect_width = {}",
         start, end, visible_slice.len(), rect.width()
     );*/
 
     // --- Price Axis (Y-axis) ---
     let (min_price, max_price) = visible_slice.iter().fold((f64::MAX, f64::MIN), |(min, max), bar| {
         (min.min(bar.low), max.max(bar.high))
     });
 
     if min_price == f64::MAX || max_price == f64::MIN {
         //println!("Invalid price range: min = {}, max = {}", min_price, max_price);
         return;
     }
 
     let price_range = max_price - min_price;
     let adjusted_min = min_price - price_range * 0.05;
     let adjusted_max = max_price + price_range * 0.05;
 
     fn nice_range(min: f64, max: f64, ticks: usize) -> (f64, f64, f64) {
         let range = max - min;
         let tick_spacing = range / (ticks - 1) as f64;
         let magnitude = 10_f64.powf(tick_spacing.log10().floor());
         let normalized = tick_spacing / magnitude;
 
         let nice_tick = if normalized <= 1.5 {
             1.0
         } else if normalized <= 3.0 {
             2.0
         } else if normalized <= 7.0 {
             5.0
         } else {
             10.0
         } * magnitude;
 
         let nice_min = (min / nice_tick).floor() * nice_tick;
         let nice_max = (max / nice_tick).ceil() * nice_tick;
 
         (nice_min, nice_max, nice_tick)
     }
 
     let (nice_min, nice_max, tick_spacing) = nice_range(adjusted_min, adjusted_max, 6);
     let price_rect = rect;
 
     // Рисуем ценовые линии и метки
     let tick_count = ((nice_max - nice_min) / tick_spacing).round() as i32;
     for i in 0..=tick_count {
         let price = nice_min + i as f64 * tick_spacing;
         let y = price_rect.top() + ((nice_max - price) / (nice_max - nice_min)) as f32 * price_rect.height();
 
         painter.line_segment(
             [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
             (0.5, grid_color),
         );
 
         let price_text = format_price(price);
         painter.text(
             egui::pos2(rect.left() + 5.0, y - 2.0),
             egui::Align2::LEFT_BOTTOM,
             price_text,
             egui::FontId::proportional(10.0),
             text_color,
         );
     }
 
     fn format_price(price: f64) -> String {
         let abs_price = price.abs();
         let (value, suffix) = if abs_price >= 1_000_000.0 {
             (price / 1_000_000.0, "m")
         } else if abs_price >= 1_000.0 {
             (price / 1_000.0, "k")
         } else {
             (price, "")
         };
 
         let fractional = value - value.floor();
         if fractional.abs() < settings::PRICE_FRACTION_THRESHOLD * value.abs() {
             format!("{}{}", value.round() as i64, suffix)
         } else {
             format!("{:.2}{}", value, suffix)
         }
     }
 
     // --- Time Axis (X-axis) ---
     let time_span_ms = visible_slice
         .last()
         .map(|bar| bar.time)
         .unwrap_or(0)
         - visible_slice
         .first()
         .map(|bar| bar.time)
         .unwrap_or(0);
 
     if time_span_ms <= 0 {
         //println!("Invalid time span: {}", time_span_ms);
         return;
     }
 
     // Расчёт интервалов
     let avg_label_width = 40.0;
     let min_pixel_gap = 80.0;
     let max_labels = (rect.width() / (avg_label_width + min_pixel_gap)).floor() as usize;
     let target_lines = max_labels.clamp(6, 14);
     let min_interval_ms = time_span_ms / target_lines as i64;
 
     let intervals = [
         1_000,          // 1 секунда
         60_000,         // 1 минута
         300_000,        // 5 минут
         900_000,        // 15 минут
         3_600_000,      // 1 час
         43_200_000,     // 12 часов
         86_400_000,     // 1 день
         604_800_000,    // 1 неделя
         2_592_000_000,  // 1 месяц
         31_536_000_000, // 1 год
     ];
 
     let mut time_interval_ms = intervals
         .iter()
         .find(|&&interval| interval >= min_interval_ms)
         .copied()
         .unwrap_or(intervals[intervals.len() - 1]);
 
     let first_time = visible_slice.first().map(|bar| bar.time).unwrap_or(0);
     let first_dt = DateTime::<Utc>::from_timestamp_millis(first_time).unwrap_or(Utc::now());
     let first_time_rounded = first_dt
         .with_hour(0)
         .and_then(|dt| dt.with_minute(0))
         .and_then(|dt| dt.with_second(0))
         .and_then(|dt| dt.with_nanosecond(0))
         .map(|dt| dt.timestamp_millis())
         .unwrap_or(first_time);
 
     //println!("First time: {}, rounded: {}", first_time, first_time_rounded);
 
     let last_dt = DateTime::<Utc>::from_timestamp_millis(first_time + time_span_ms).unwrap_or(Utc::now());
     let has_two_years = first_dt.year() != last_dt.year();
     let has_two_months = first_dt.month() != last_dt.month() || first_dt.year() != last_dt.year();
 
     let mut labels = vec![];
     let left_margin = 30.0;
 
     // Основной цикл меток
     for time_ms in (first_time_rounded..=first_time + time_span_ms + 10 * time_interval_ms)
         .step_by(time_interval_ms as usize)
     {
         if let Some((bar_idx, _)) = data_window
             .bars
             .iter()
             .enumerate()
             .skip(start as usize)
             .find(|(_, bar)| bar.time >= time_ms)
         {
             if bar_idx as i64 > end {
                 //println!("Stopped at bar_idx = {}, end = {}", bar_idx, end);
                 break;
             }
             let normalized_pos = (bar_idx as f64 - start as f64) / (end - start) as f64;
             let x_right = rect.left() + (normalized_pos as f32) * rect.width();
 
             if x_right < rect.left() + left_margin || x_right > rect.right() + 200.0 {
                 //println!("Filtered: x_right = {}, rect.right() = {}", x_right, rect.right());
                 continue;
             }
 
             labels.push((time_ms, bar_idx, x_right));
         }
     }
 
     // 12-часовые метки
     if labels.len() <= 7 && time_interval_ms >= 86_400_000 {
         //println!("Switch to 12h: labels = {}, interval = {}", labels.len(), time_interval_ms);
         time_interval_ms = 43_200_000;
         labels.clear();
         for time_ms in (first_time_rounded..=first_time + time_span_ms + 10 * time_interval_ms)
             .step_by(time_interval_ms as usize)
         {
             if let Some((bar_idx, _)) = data_window
                 .bars
                 .iter()
                 .enumerate()
                 .skip(start as usize)
                 .find(|(_, bar)| bar.time >= time_ms)
             {
                 if bar_idx as i64 > end {
                     //println!("Stopped at bar_idx = {}, end = {}", bar_idx, end);
                     break;
                 }
                 let normalized_pos = (bar_idx as f64 - start as f64) / (end - start) as f64;
                 let x_right = rect.left() + (normalized_pos as f32) * rect.width();
 
                 if x_right < rect.left() + left_margin || x_right > rect.right() + 200.0 {
                     //println!("Filtered: x_right = {}, rect.right() = {}", x_right, rect.right());
                     continue;
                 }
 
                labels.push((time_ms, bar_idx, x_right));
             }
         }
     }
 
     //println!("Final labels: {}, time_interval_ms: {}", labels.len(), time_interval_ms);
 
     // Рисуем метки
     let mut drawn_lines = 0;
     let mut last_x_right: Option<f32> = None;
 
     for (time_ms, bar_idx, x_right) in labels {
         if let Some(prev_x) = last_x_right {
             let pixel_gap = if drawn_lines < 6 { 50.0 } else { min_pixel_gap };
             if (x_right - prev_x).abs() < pixel_gap {
                 continue;
             }
         }
 
         /*println!(
             "Time grid: time_ms = {}, bar_idx = {}, x_right = {}",
             time_ms, bar_idx, x_right
         );*/
 
         painter.line_segment(
             [egui::pos2(x_right, rect.top()), egui::pos2(x_right, rect.bottom())],
             (0.5, grid_color),
         );
 
         let dt = DateTime::<Utc>::from_timestamp_millis(time_ms).unwrap_or(Utc::now());
         let label = match time_interval_ms {
             i if i >= 31_536_000_000 && has_two_years => dt.year().to_string(),
             i if i >= 2_592_000_000 && has_two_months => dt.format("%b").to_string(),
             i if i >= 86_400_000 => dt.day().to_string(),
             i if i >= 43_200_000 => {
                 if dt.hour() == 0 {
                     dt.day().to_string()
                 } else {
                     dt.format("%H:%M").to_string()
                 }
             }
             i if i >= 3_600_000 => dt.format("%H:%M").to_string(),
             _ => dt.format("%H:%M:%S").to_string(),
         };
 
         let galley = painter.layout_no_wrap(label.clone(), egui::FontId::proportional(10.0), text_color);
         let text_x = x_right - galley.size().x / 2.0;
 
         //println!("Drawing text: label = {}, x_right = {}, text_x = {}", label, x_right, text_x);
         painter.text(
             egui::pos2(text_x, rect.bottom() + 2.0),
             egui::Align2::CENTER_TOP,
             label,
             egui::FontId::proportional(10.0),
             text_color,
         );
 
         last_x_right = Some(x_right);
         drawn_lines += 1;
         if drawn_lines >= target_lines {
             break;
         }
     }
 }