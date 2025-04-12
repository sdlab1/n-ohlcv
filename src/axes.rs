// axes.rs
use eframe::egui;
use chrono::{DateTime, Utc, Datelike, Timelike};
use crate::settings;

pub fn draw(ui: &mut egui::Ui, rect: egui::Rect, data_window: &crate::DataWindow) {
    let painter = ui.painter();
    let text_color = ui.style().visuals.text_color();
    let grid_color = egui::Color32::from_gray(60);

    let (start, end) = data_window.visible_range;
    let end = end.min(data_window.bars.len() as i64); // Ограничиваем end
    if start >= end || end as usize > data_window.bars.len() {
        return;
    }

    let visible_slice = &data_window.bars[start as usize..end as usize];
    if visible_slice.is_empty() {
        return;
    }

    println!("Axes: visible_range = ({}, {}), bars_len = {}, rect_width = {}", 
             start, end, visible_slice.len(), rect.width());

    let (min_price, max_price) = visible_slice.iter().fold((f64::MAX, f64::MIN), |(min, max), bar| {
        (min.min(bar.low), max.max(bar.high))
    });

    let price_range = max_price - min_price;
    let adjusted_min = min_price - price_range * 0.05;
    let adjusted_max = max_price + price_range * 0.05;

    // Функция для создания "красивых" границ и шага
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

    let volume_height = rect.height() * data_window.volume_height_ratio;
    let price_rect = egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.max.y - volume_height));

    painter.line_segment([rect.left_bottom(), rect.right_bottom()], (1.0, text_color));
    painter.line_segment([rect.left_bottom(), rect.left_top()], (1.0, text_color));

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
        if fractional.abs() < crate::settings::PRICE_FRACTION_THRESHOLD * value.abs() {
            format!("{}{}", value.round() as i64, suffix)
        } else {
            format!("{:.2}{}", value, suffix)
        }
    }

    let tick_count = ((nice_max - nice_min) / tick_spacing).round() as i32;
    for i in 0..=tick_count {
        let price = nice_min + i as f64 * tick_spacing;
        let y = price_rect.top() + ((nice_max - price) / (nice_max - nice_min)) as f32 * price_rect.height();
        painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], (0.5, grid_color));

        let price_text = format_price(price);
        painter.text(
            egui::pos2(rect.left() + 5.0, y - 2.0),
            egui::Align2::LEFT_BOTTOM,
            price_text,
            egui::FontId::proportional(10.0),
            text_color,
        );
    }

    // Time grid
    let visible_slice = &data_window.bars[start as usize..end as usize];
    if visible_slice.is_empty() {
        return;
    }

    let time_span_ms = visible_slice.last().map(|bar| bar.time).unwrap_or(0)
        - visible_slice.first().map(|bar| bar.time).unwrap_or(0);

    // Автоматический расчёт количества меток
    let avg_label_width = 40.0; // Примерная ширина метки в пикселях
    let min_pixel_gap = 50.0; // Минимальный зазор между метками
    let max_labels = (rect.width() / (avg_label_width + min_pixel_gap)).floor() as usize;
    let mut target_lines = max_labels.clamp(6, 12); // Минимум 6, максимум 12
    let mut min_interval_ms = time_span_ms / target_lines as i64;

    let intervals = [
        60_000,        // 1 минута
        300_000,       // 5 минут
        900_000,       // 15 минут
        1_800_000,     // 30 минут
        3_600_000,     // 1 час
        7_200_000,     // 2 часа
        14_400_000,    // 4 часа
        21_600_000,    // 6 часов
        43_200_000,    // 12 часов
        86_400_000,    // 1 день
        604_800_000,   // 1 неделя
        2_592_000_000, // 1 месяц
        31_536_000_000 // 1 год
    ];
    let mut time_interval_ms = intervals
        .iter()
        .find(|&&interval| interval >= min_interval_ms)
        .copied()
        .unwrap_or(intervals[intervals.len() - 1]);

    let first_time = visible_slice.first().map(|bar| bar.time).unwrap_or(0);
    let first_time_rounded = (first_time / time_interval_ms) * time_interval_ms;

    // Проверяем наличие двух годов и месяцев
    let first_dt = DateTime::<Utc>::from_timestamp_millis(first_time).unwrap_or(Utc::now());
    let last_dt = DateTime::<Utc>::from_timestamp_millis(first_time + time_span_ms).unwrap_or(Utc::now());
    let has_two_years = first_dt.year() != last_dt.year();
    let has_two_months = first_dt.month() != last_dt.month();

    let mut drawn_lines = 0;
    let mut last_x_right: Option<f32> = None;
    let left_margin: f32 = 30.0;
    let mut labels = vec![];

    // Основной цикл для меток
    for time_ms in (first_time_rounded..=first_time + time_span_ms + time_interval_ms).step_by(time_interval_ms as usize) {
        if let Some((bar_idx, bar)) = data_window
            .bars
            .iter()
            .enumerate()
            .skip(start as usize)
            .find(|(_, bar)| bar.time >= time_ms)
        {
            if bar_idx as i64 > end {
                break;
            }
            let normalized_pos = (bar_idx as f64 - start as f64) / (end - start) as f64;
            let x_right = rect.left() + (normalized_pos as f32) * rect.width();

            if x_right < rect.left() + left_margin || x_right > rect.right() - 10.0 {
                continue;
            }

            labels.push((time_ms, bar_idx, normalized_pos, x_right));
        }
    }

    // Если меток меньше 6, уменьшаем интервал
    if labels.len() < 6 && time_interval_ms >= 86_400_000 {
        time_interval_ms = 43_200_000; // Переходим на 12 часов
        labels.clear();
        let new_first_time_rounded = (first_time / time_interval_ms) * time_interval_ms;
        for time_ms in (new_first_time_rounded..=first_time + time_span_ms + time_interval_ms).step_by(time_interval_ms as usize) {
            if let Some((bar_idx, bar)) = data_window
                .bars
                .iter()
                .enumerate()
                .skip(start as usize)
                .find(|(_, bar)| bar.time >= time_ms)
            {
                if bar_idx as i64 > end {
                    break;
                }
                let normalized_pos = (bar_idx as f64 - start as f64) / (end - start) as f64;
                let x_right = rect.left() + (normalized_pos as f32) * rect.width();

                if x_right < rect.left() + left_margin || x_right > rect.right() - 10.0 {
                    continue;
                }

                labels.push((time_ms, bar_idx, normalized_pos, x_right));
            }
        }
    }

    // Рисуем метки
    for (time_ms, bar_idx, normalized_pos, x_right) in labels {
        // Проверка минимального зазора
        if let Some(prev_x) = last_x_right {
            let pixel_gap = if drawn_lines < 6 { 30.0 } else { min_pixel_gap };
            if (x_right - prev_x).abs() < pixel_gap {
                continue;
            }
        }

        println!(
            "Time grid: time_ms = {}, bar_idx = {}, normalized_pos = {}, x_right = {}",
            time_ms, bar_idx, normalized_pos, x_right
        );
        painter.line_segment(
            [egui::pos2(x_right, rect.top()), egui::pos2(x_right, rect.bottom())],
            (0.5, grid_color),
        );

        let dt = DateTime::<Utc>::from_timestamp_millis(time_ms).unwrap_or(Utc::now());
        let label = match time_interval_ms {
            // Годы (только если два года)
            i if i >= 31_536_000_000 && has_two_years => dt.format("%Y").to_string(), // 2025
            // Месяцы (только если два месяца)
            i if i >= 2_592_000_000 && has_two_months => dt.format("%b").to_string(), // Apr
            // Дни
            i if i >= 86_400_000 => dt.format("%d").to_string(), // 9
            // Промежуточные (12 часов)
            i if i >= 43_200_000 => {
                if dt.hour() == 0 {
                    dt.format("%d").to_string() // Полночь — день
                } else {
                    dt.format("%H:%M").to_string() // Например, "6 12:00"
                }
            }
            // Часы
            i if i >= 3_600_000 => dt.format("%H:%M").to_string(), // 12:00
            // Минуты и секунды
            _ => dt.format("%H:%M:%S").to_string(), // 12:00:00
        };

        let galley = painter.layout_no_wrap(label.clone(), egui::FontId::proportional(10.0), text_color);
        let text_width = galley.size().x;
        let text_x = if x_right < rect.left() + text_width / 2.0 {
            rect.left() + text_width / 2.0
        } else if x_right > rect.right() - text_width / 2.0 {
            rect.right() - text_width / 2.0
        } else {
            x_right
        };

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