/*
 * axes_util.rs - Utility functions for axes rendering in OHLCV charting
 *
 * Copyleft (c) 2025 Grok 3
 *
 * Description:
 * This module contains utility functions for formatting prices and calculating
 * "nice" ranges for the price axis in an OHLCV financial chart.
 *
 * Dependencies:
 * - crate::settings: For PRICE_FRACTION_THRESHOLD
 *
 * Key Functions:
 * - format_price: Formats price values with k/m suffixes
 * - format_price_high_precision: Formats price values with increased precision
 * - nice_range: Calculates "nice" price boundaries and tick spacing
 *
 * Author: Grok 3, improved by Gemini 2.5
 * Date: April 12, 2025 // Updated April 13, 2025
 */

 use crate::settings;

 pub fn format_price(price: f64) -> String {
     let abs_price = price.abs();
     let (value, suffix, decimals): (f64, &str, usize) = if abs_price >= 1_000_000.0 {
         (price / 1_000_000.0, "m", 1)
     } else if abs_price >= 1_000.0 {
         (price / 1_000.0, "k", 1)
     } else {
         (price, "", 2)
     };
 
     let tolerance = if suffix.is_empty() { 1e-9 } else { 10.0_f64.powi(-( (decimals + 1) as i32 )) };
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
 
     let target_ticks = ticks.max(2);
     let tick_spacing = range / (target_ticks - 1) as f64;
     let magnitude = 10_f64.powf(tick_spacing.log10().floor());
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
 
     if (nice_max - nice_min) / nice_tick > 1000.0 {
         return (min, max, (max-min).max(1e-9)/4.0);
     }
 
     (nice_min, nice_max, nice_tick)
 }