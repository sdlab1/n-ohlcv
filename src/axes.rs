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
 * with 5-7 ticks and k/m suffixes for large values. It now handles duplicate labels
 * by increasing precision.
 *
 * Dependencies:
 * - eframe::egui: For UI rendering
 * - chrono: For timestamp parsing and formatting
 * - crate::DataWindow: Provides OHLCV bar data and visible range
 * - crate::settings: For PRICE_FRACTION_THRESHOLD
 *
 * Key Functions:
 * - draw: Renders price and time axes with adaptive grids and labels
 * - nice_range: Calculates "nice" price boundaries and tick spacing
 * - format_price: Formats price values with k/m suffixes
 * - format_price_high_precision: Formats price values with increased precision
 *
 * Usage:
 * Call `draw` with a `Ui` context, a `Rect` for the chart area, and a `DataWindow`
 * containing OHLCV data. The function computes grids, draws lines, and places labels.
 *
 * Debugging Tips:
 * - No labels on right:
 * Check: println!("Drawing text: label = {}, x_right = {}, text_x = {}", label, x_right, text_x);
 * Ensure `x_right` <= `rect.right() + 50.0` and `text_x` fits in `clip_rect`.
 * - Fewer than 6 labels:
 * Check: println!("Final labels: {}, time_interval_ms: {}", labels.len(), time_interval_ms);
 * Extend `time_span_ms` with `+ 5 * time_interval_ms`.
 * - Labels overlap:
 * Increase: let min_pixel_gap = 80.0;
 * - 12-hour labels missing:
 * Check: println!("Switch to 12h: labels = {}, interval = {}", labels.len(), time_interval_ms);
 * Ensure `labels.len() <= 7`.
 * - Price labels unreadable:
 * Adjust: egui::FontId::proportional(12.0);
 *
 * Notes:
 * - Time labels adapt to zoom: years/months for wide ranges, days/hours for narrow.
 * - Price axis adjusts to min/max prices with 5% padding.
 * - Ensure `DataWindow` has valid bars to avoid empty ranges.
 *
 * Author: Grok 3, improved by Gemini 2.5
 * Date: April 12, 2025 // Updated April 13, 2025
 */

 use eframe::egui::{self, Color32, Rect, Ui};
 use chrono::{DateTime, Utc, Datelike, Timelike};
 use crate::DataWindow;
 use crate::settings; // Убедитесь, что settings импортирован
 
 // --- Функция форматирования ---
 fn format_price(price: f64) -> String {
     let abs_price = price.abs();
     // Используем usize для decimals
     let (value, suffix, decimals): (f64, &str, usize) = if abs_price >= 1_000_000.0 {
         (price / 1_000_000.0, "m", 1)
     } else if abs_price >= 1_000.0 {
         (price / 1_000.0, "k", 1)
     } else {
         (price, "", 2)
     };
 
     let tolerance = if suffix.is_empty() { 1e-9 } else { 10.0_f64.powi(-( (decimals + 1) as i32 )) }; // Приведение к i32 для powi
     let is_round = value.fract().abs() < tolerance;
 
     if is_round {
         format!("{:.0}{}", value, suffix)
     } else if suffix.is_empty() && abs_price > 1.0 && value.fract().abs() < settings::PRICE_FRACTION_THRESHOLD {
          format!("{:.0}", value)
     } else {
         // decimals уже usize, ошибки быть не должно
         format!("{:.prec$}{}", value, suffix, prec = decimals)
     }
 }
 
 // --- Функция форматирования с повышенной точностью ---
 fn format_price_high_precision(price: f64) -> String {
     let abs_price = price.abs();
     // Используем usize для decimals
     let (value, suffix, decimals): (f64, &str, usize) = if abs_price >= 1_000_000.0 {
         (price / 1_000_000.0, "m", 2)
     } else if abs_price >= 1_000.0 {
         (price / 1_000.0, "k", 2)
     } else {
         (price, "", 3)
     };
     // decimals уже usize
     format!("{:.prec$}{}", value, suffix, prec = decimals)
 }
 
 
 pub fn draw(ui: &mut Ui, rect: Rect, data_window: &DataWindow) {
     let painter = ui.painter();
     let text_color = ui.style().visuals.text_color();
     let grid_color = Color32::from_gray(60);
 
     // Проверяем данные
     let (start, end) = data_window.visible_range;
     let end = end.min(data_window.bars.len() as i64);
     if start < 0 || start >= end || end > data_window.bars.len() as i64 {
         return;
     }
 
     let visible_slice = &data_window.bars[start as usize..end as usize];
     if visible_slice.is_empty() {
         return;
     }
 
     // --- Price Axis (Y-axis) ---
     let (min_price, max_price) = visible_slice.iter().fold((f64::MAX, f64::MIN), |(min, max), bar| {
         (min.min(bar.low), max.max(bar.high))
     });
 
     if min_price == f64::MAX || max_price == f64::MIN || min_price >= max_price {
         return;
     }
 
     let price_range = (max_price - min_price).max(1e-9);
     let adjusted_min = min_price - price_range * 0.05;
     let adjusted_max = max_price + price_range * 0.05;
 
     // --- Функция nice_range ---
     fn nice_range(min: f64, max: f64, ticks: usize) -> (f64, f64, f64) {
         let range = (max - min).max(1e-9);
         if range <= 1e-9 { return (min, max, 1.0); }
 
         let target_ticks = ticks.max(2);
         let tick_spacing = range / (target_ticks - 1) as f64;
         let magnitude = 10_f64.powf(tick_spacing.log10().floor());
         // Добавим проверку на нулевой magnitude
          if magnitude <= 1e-9 { return (min, max, range / (target_ticks - 1) as f64); }
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
 
         let nice_tick = nice_tick.max(1e-9);
 
         let nice_min = (min / nice_tick).floor() * nice_tick;
         let nice_max = (max / nice_tick).ceil() * nice_tick;
 
         // Проверка, чтобы избежать слишком большого количества тиков
         if (nice_max - nice_min) / nice_tick > 1000.0 {
              return (min, max, (max-min).max(1e-9)/4.0);
         }
 
         (nice_min, nice_max, nice_tick)
     }
 
     let (nice_min, nice_max, tick_spacing) = nice_range(adjusted_min, adjusted_max, 6);
 
     if nice_max <= nice_min || tick_spacing <= 1e-9 {
         return;
     }
 
     let price_rect = rect;
 
     // --- ЭТАП 1: Сбор информации о метках ---
     let mut price_labels_info: Vec<(f64, String, f32)> = Vec::new();
     let tick_count = (((nice_max - nice_min) / tick_spacing).round() as i32).min(100);
 
     for i in 0..=tick_count {
         let price = nice_min + i as f64 * tick_spacing;
         let y = if (nice_max - nice_min).abs() > 1e-9 {
              price_rect.top() + ((nice_max - price) / (nice_max - nice_min)) as f32 * price_rect.height()
         } else {
              price_rect.center().y
         };
 
         if y < price_rect.top() - 10.0 || y > price_rect.bottom() + 10.0 {
             continue;
         }
 
         painter.line_segment(
             [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
             (0.5, grid_color),
         );
 
         let initial_label = format_price(price);
         price_labels_info.push((price, initial_label, y));
     }
 
     // --- ЭТАП 2: Проверка на дубликаты и переформатирование ---
     if price_labels_info.len() >= 2 {
         let mut final_labels = price_labels_info.clone();
         let mut changed = false;
         for i in 1..final_labels.len() {
              if final_labels[i].1 == final_labels[i-1].1 {
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
 
     // --- ЭТАП 3: Отрисовка текстовых меток ---
     for (_price, label_text, y) in &price_labels_info {
         // Исправлено: Разыменовываем y для сравнения
         if *y >= price_rect.top() - 2.0 && *y <= price_rect.bottom() + 12.0 {
              painter.text(
                  egui::pos2(rect.left() + 5.0, y - 2.0), // Используем y напрямую для позиционирования
                  egui::Align2::LEFT_BOTTOM,
                  label_text.clone(),
                  egui::FontId::proportional(10.0),
                  text_color,
              );
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
      // Исправлено: unwrap_or_else требует замыкание без аргументов
      let first_dt = DateTime::<Utc>::from_timestamp_millis(first_time).unwrap_or_else(|| Utc::now());
 
      let first_time_rounded = first_time - first_time % time_interval_ms.max(1);
 
      // Исправлено: unwrap_or_else требует замыкание без аргументов
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
                       let last_bar_idx_abs = start + visible_slice.len() as i64 -1;
                       if last_bar_idx_abs >= start { // Проверка что индекс валидный
                          let normalized_pos = (last_bar_idx_abs as f64 - start as f64) / visible_bar_count;
                          let x_right = rect.left() + (normalized_pos as f32) * rect.width();
                          if labels.last().map_or(true, |l| (x_right - l.2).abs() >= min_pixel_gap * 0.8) {
                              labels.push((last_bar.time, last_bar_idx_abs as usize, x_right));
                          }
                       }
                  }
                  break;
               }
 
              let visible_bar_count = (end - start).max(1) as f64;
              // Используем bar_idx напрямую, т.к. он уже абсолютный индекс в data_window.bars
              let normalized_pos = (bar_idx as f64 - start as f64) / visible_bar_count;
              let x_right = rect.left() + (normalized_pos as f32) * rect.width();
 
 
               if x_right >= rect.left() + left_margin && x_right <= rect.right() - right_margin {
                  if labels.last().map_or(true, |last_label| (x_right - last_label.2).abs() >= min_pixel_gap * 0.8) {
                      labels.push((current_time_check, bar_idx, x_right));
                  }
              } else if bar_idx as i64 > start && x_right > rect.right() - right_margin {
                   break;
              }
          } else if current_time_check > first_time {
               break;
          }
          if time_interval_ms <= 0 { break; }
          current_time_check += time_interval_ms;
      }
 
 
      // Рисуем метки времени
      let mut last_drawn_x: Option<f32> = None;
      // Исправлено: используем _bar_idx, т.к. переменная не используется
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
 
          // Исправлено: unwrap_or_else требует замыкание без аргументов
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
          // Используем разыменованное значение *x_right
          let text_x = x_right - galley.size().x / 2.0;
 
          painter.text(
              // Используем разыменованное значение *x_right
              egui::pos2(text_x, rect.bottom() + 2.0),
              egui::Align2::CENTER_TOP,
              label,
              egui::FontId::proportional(10.0),
              text_color,
          );
 
          last_drawn_x = Some(*x_right);
      }
 }