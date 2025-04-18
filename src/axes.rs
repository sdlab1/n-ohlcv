//axes.rs
use eframe::egui::{self, Color32, Rect, Ui};
use chrono::{DateTime, Utc, Datelike};
use crate::datawindow::DataWindow;
use crate::axes_util::{
    generate_price_labels, deduplicate_price_labels,
    choose_time_interval, format_time_label,
};

pub fn draw(
    ui: &mut Ui,
    rect: Rect,
    data_window: &DataWindow,
    scale_price: &impl Fn(f64) -> f32,
) {
    let painter = ui.painter();
    let text_color = ui.style().visuals.text_color();
    let grid_color = Color32::from_gray(60);

    let volume_height = rect.height() * data_window.volume_height_ratio;
    let price_rect = Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.max.y - volume_height));
    let (min_price, max_price) = data_window.price;
    if min_price >= max_price {
        return;
    }

    // --- Y Axis: Prices ---
    let mut price_labels_info = generate_price_labels(
        min_price,
        max_price,
        &scale_price,
        price_rect.top(),
        price_rect.bottom(),
    );
    price_labels_info = deduplicate_price_labels(price_labels_info);

    for (_price, label_text, y) in &price_labels_info {
        painter.line_segment(
            [egui::pos2(rect.left(), *y), egui::pos2(rect.right(), *y)],
            (0.5, grid_color),
        );

        let galley = painter.layout_no_wrap(
            label_text.clone(),
            egui::FontId::proportional(10.0),
            text_color,
        );
        let text_rect = Rect::from_min_size(
            egui::pos2(rect.left() + 5.0, *y - 2.0 - galley.size().y),
            galley.size() + egui::vec2(4.0, 4.0),
        );

        if text_rect.min.y >= price_rect.top() && text_rect.max.y <= price_rect.bottom() {
            painter.rect_filled(text_rect, 0.0, Color32::from_rgba_premultiplied(20, 20, 20, 220));
            painter.text(
                egui::pos2(rect.left() + 7.0, *y - 2.0),
                egui::Align2::LEFT_BOTTOM,
                label_text.clone(),
                egui::FontId::proportional(10.0),
                text_color,
            );
        }
    }

    // --- X Axis: Time ---
    let (start, end) = data_window.visible_range;
    let end = end.min(data_window.bars.len() as i64);
    if start < 0 || start >= end || end > data_window.bars.len() as i64 {
        return;
    }

    let visible_slice = &data_window.bars[start as usize..end as usize];
    if visible_slice.is_empty() {
        return;
    }

    let time_span_ms = visible_slice.last().map(|bar| bar.time).unwrap_or(0)
        - visible_slice.first().map(|bar| bar.time).unwrap_or(0);

    if time_span_ms <= 0 {
        return;
    }

    let avg_label_width = 40.0;
    let min_pixel_gap = 60.0;
    let max_labels = (rect.width() / (avg_label_width + min_pixel_gap)).floor().max(1.0) as usize;
    let target_lines = max_labels.clamp(4, 10);
    let time_interval_ms = choose_time_interval(time_span_ms, target_lines);

    let first_time = visible_slice.first().map(|bar| bar.time).unwrap_or(0);
    let first_time_rounded = first_time - first_time % time_interval_ms.max(1);
    let last_time = first_time + time_span_ms;

    let first_dt = DateTime::<Utc>::from_timestamp_millis(first_time).unwrap_or_else(Utc::now);
    let last_dt = DateTime::<Utc>::from_timestamp_millis(last_time).unwrap_or_else(Utc::now);
    let has_two_years = first_dt.year() != last_dt.year();
    let has_two_months = first_dt.month() != last_dt.month() || first_dt.year() != last_dt.year();
    let has_two_days = first_dt.ordinal() != last_dt.ordinal() || first_dt.year() != last_dt.year();

    let mut labels: Vec<(i64, usize, f32)> = vec![];
    let left_margin = 5.0;
    let right_margin = 5.0;

    let max_iterations = (time_span_ms / time_interval_ms + 20).min(2000);
    let mut current_time_check = first_time_rounded;

    for _ in 0..max_iterations {
        if current_time_check > last_time + time_interval_ms {
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
                    let last_idx_abs = start + visible_slice.len() as i64 - 1;
                    let normalized_pos = (last_idx_abs as f64 - start as f64) / visible_bar_count;
                    let x = rect.left() + (normalized_pos as f32) * rect.width() + data_window.pixel_offset;
                    if labels.last().map_or(true, |l| (x - l.2).abs() >= min_pixel_gap * 0.8) {
                        labels.push((last_bar.time, last_idx_abs as usize, x));
                    }
                }
                break;
            }

            let visible_bar_count = (end - start).max(1) as f64;
            let normalized_pos = (bar_idx as f64 - start as f64) / visible_bar_count;
            let x = rect.left() + (normalized_pos as f32) * rect.width() + data_window.pixel_offset;

            if x >= rect.left() + left_margin && x <= rect.right() - right_margin {
                if labels.last().map_or(true, |l| (x - l.2).abs() >= min_pixel_gap * 0.8) {
                    labels.push((current_time_check, bar_idx, x));
                }
            } else if bar_idx as i64 > start && x > rect.right() - right_margin {
                break;
            }
        }

        if time_interval_ms <= 0 {
            break;
        }

        current_time_check += time_interval_ms;
    }

    let mut last_drawn_x: Option<f32> = None;

    for (time_ms, _bar_idx, x) in &labels {
        if last_drawn_x.map_or(false, |last_x| (*x - last_x).abs() < min_pixel_gap) {
            continue;
        }

        painter.line_segment(
            [egui::pos2(*x, rect.top()), egui::pos2(*x, rect.bottom())],
            (0.5, grid_color),
        );

        let dt = DateTime::<Utc>::from_timestamp_millis(*time_ms).unwrap_or_else(Utc::now);
        let label = format_time_label(dt, time_interval_ms, has_two_years, has_two_months, has_two_days);

        let galley = painter.layout_no_wrap(label.clone(), egui::FontId::proportional(10.0), text_color);
        let text_x = x - galley.size().x / 2.0;

        painter.text(
            egui::pos2(text_x, rect.bottom() + 2.0),
            egui::Align2::CENTER_TOP,
            label,
            egui::FontId::proportional(10.0),
            text_color,
        );

        last_drawn_x = Some(*x);
    }
}
