//axes_util.rs
use crate::{settings, datawindow::DataWindow};
use chrono::{DateTime, Utc, Datelike, Timelike};

pub fn create_scale_price_fn(
    data_window: &DataWindow,
    rect: egui::Rect,
) -> impl Fn(f64) -> f32 {
    let (min_price, max_price) = data_window.price;
    let range = (max_price - min_price).max(1e-9);
    let height = rect.height();
    let bottom = rect.bottom();

    move |price: f64| -> f32 {
        bottom - ((price - min_price) / range) as f32 * height
    }
}

pub fn format_price(price: f64) -> String {
    let abs_price = price.abs();
    let (value, suffix, decimals): (f64, &str, usize) = if abs_price >= 1_000_000.0 {
        (price / 1_000_000.0, "m", 1)
    } else if abs_price >= 1_000.0 {
        (price / 1_000.0, "k", 1)
    } else {
        (price, "", 2)
    };

    let tolerance = if suffix.is_empty() { 1e-9 } else { 10f64.powi(-(decimals as i32 + 1)) };
    let is_round = value.fract().abs() < tolerance;

    if is_round {
        format!("{:.0}{}", value, suffix)
    } else if suffix.is_empty() && abs_price > 1.0 && value.fract().abs() < settings::PRICE_FRACTION_THRESHOLD {
        format!("{:.0}", value)
    } else {
        format!("{:.prec$}{}", value, suffix, prec = decimals)
    }
}

pub fn format_price_high_precision(price: f64) -> String {
    let abs_price = price.abs();
    let (value, suffix, decimals): (f64, &str, usize) = if abs_price >= 1_000_000.0 {
        (price / 1_000_000.0, "m", 2)
    } else if abs_price >= 1_000.0 {
        (price / 1_000.0, "k", 2)
    } else {
        (price, "", 3)
    };
    format!("{:.prec$}{}", value, suffix, prec = decimals)
}

pub fn nice_range(min: f64, max: f64, ticks: usize) -> (f64, f64, f64) {
    let range = (max - min).max(1e-9);
    if range <= 1e-9 { return (min, max, 1.0); }

    let tick_spacing = range / (ticks.max(2) - 1) as f64;
    let magnitude = 10f64.powf(tick_spacing.log10().floor());
    let normalized = tick_spacing / magnitude;

    let nice_tick = match normalized {
        n if n <= 1.5 => 1.0,
        n if n <= 3.0 => 2.0,
        n if n <= 7.0 => 5.0,
        _ => 10.0,
    } * magnitude;

    let nice_tick = nice_tick.max(1e-9);
    let nice_min = (min / nice_tick).floor() * nice_tick;
    let nice_max = (max / nice_tick).ceil() * nice_tick;

    if (nice_max - nice_min) / nice_tick > 1000.0 {
        return (min, max, range / 4.0);
    }

    (nice_min, nice_max, nice_tick)
}

pub fn generate_price_labels(
    min: f64,
    max: f64,
    scale_price: &impl Fn(f64) -> f32,
    height_limit_top: f32,
    height_limit_bottom: f32,
) -> Vec<(f64, String, f32)> {
    let price_range = (max - min).max(1e-9);
    let (nice_min, nice_max, tick_spacing) = nice_range(
        min - price_range * 0.05,
        max + price_range * 0.05,
        6,
    );

    if nice_max <= nice_min || tick_spacing <= 1e-9 {
        return vec![];
    }

    let tick_count = (((nice_max - nice_min) / tick_spacing).round() as i32).min(100);
    let mut labels = vec![];

    for i in 0..=tick_count {
        let price = nice_min + i as f64 * tick_spacing;
        let y = scale_price(price);
        if y < height_limit_top - 10.0 || y > height_limit_bottom {
            continue;
        }
        labels.push((price, format_price(price), y));
    }

    labels
}

pub fn deduplicate_price_labels(labels: Vec<(f64, String, f32)>) -> Vec<(f64, String, f32)> {
    if labels.len() < 2 {
        return labels;
    }

    let mut final_labels = labels.clone();
    let mut changed = false;

    for i in 1..final_labels.len() {
        if final_labels[i].1 == final_labels[i - 1].1 {
            final_labels[i - 1].1 = format_price_high_precision(final_labels[i - 1].0);
            final_labels[i].1 = format_price_high_precision(final_labels[i].0);
            changed = true;
        }
    }

    if changed {
        final_labels
    } else {
        labels
    }
}

pub fn choose_time_interval(time_span_ms: i64, target_lines: usize) -> i64 {
    let intervals = [
        1_000, 60_000, 300_000, 900_000, 1_800_000, 3_600_000, 14_400_000,
        43_200_000, 86_400_000, 604_800_000, 2_592_000_000, 31_536_000_000,
    ];

    let mut interval = intervals
        .iter()
        .find(|&&i| i >= time_span_ms / target_lines.max(1) as i64)
        .copied()
        .unwrap_or(*intervals.last().unwrap());

    if interval > 0 && time_span_ms / interval > (target_lines * 2) as i64 {
        interval = intervals
            .iter()
            .find(|&&i| i >= time_span_ms / (target_lines * 2).max(1) as i64)
            .copied()
            .unwrap_or(*intervals.last().unwrap());
    }

    interval.max(1000)
}

pub fn format_time_label(dt: DateTime<Utc>, interval_ms: i64, has_two_years: bool, has_two_months: bool, has_two_days: bool) -> String {
    match interval_ms {
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
    }
}
