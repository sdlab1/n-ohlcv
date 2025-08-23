use crate::db::Database;
use crate::fetch::KLine;
use crate::rsi::WilderRSI;
use crate::timeframe;
use crate::timeframe::Bar;
use chrono::Timelike;
use std::error::Error;

#[derive(Debug)]
pub struct DataWindow {
    pub bars: Vec<Bar>,
    pub visible_range: (i64, i64),
    pub price: (f64, f64),
    pub min_indexes: Option<Vec<usize>>,
    pub max_indexes: Option<Vec<usize>>,
    pub recent_data: Vec<KLine>,
    pub timeframe_remainder: Vec<KLine>,
    pub volume_height_ratio: f32,
    pub pixel_offset: f32,
    cached_visible_range: Option<(i64, i64)>,
    cached_max_volume: Option<f64>,
}

pub const BLOCK_SIZE: usize = 1000;

impl DataWindow {
    pub fn get_data_window(
        db: &Database,
        symbol: &str,
        start_time: i64,
        end_time: i64,
        timeframe_minutes: i32,
        data_window: &mut DataWindow,
    ) -> Result<(), Box<dyn Error>> {
        println!(
            "get_data_window: symbol = {}, start_time = {}, end_time = {}, timeframe = {}",
            symbol, start_time, end_time, timeframe_minutes
        );
        timeframe::Timeframe::sync_data(3, db, symbol, start_time, end_time, data_window)?;

        let mut bars = Vec::new();
        let mut current_block_start = timeframe::Timeframe::get_dbtimestamp(start_time);
        let period = 14;
        let mut rsi_calculator = WilderRSI::new(period);
        while current_block_start <= end_time {
            println!("Get block from db, timestamp: {}", current_block_start);
            if let Some(mut block) = db.get_block(symbol, current_block_start)? {
                if bars.is_empty() {
                    if let Some(i) = block.iter().position(|k| {
                        chrono::DateTime::from_timestamp_millis(k.open_time)
                            .map_or(false, |dt| dt.minute() == 0)
                    }) {
                        block = block.split_off(i); // cut  "hh:00"
                    }
                }
                let converted = timeframe::Timeframe::convert_to_timeframe(
                    block,
                    timeframe_minutes,
                    false,
                    data_window,
                    &mut rsi_calculator,
                )?;
                println!(
                    "Block at {} has {} bars after conversion, remainder.len: {}",
                    current_block_start,
                    converted.len(),
                    data_window.timeframe_remainder.len()
                );
                bars.extend(converted);
            } else {
                println!("No data for block at {}", current_block_start);
            }
            current_block_start += BLOCK_SIZE as i64 * 60_000;
        }
        println!("bars.len: {}", bars.len());
        println!(
            "data_window.recent_data (minutes): {}",
            data_window.recent_data.len()
        );
        bars.extend(timeframe::Timeframe::convert_to_timeframe(
            data_window.recent_data.to_vec(),
            timeframe_minutes,
            true,
            data_window,
            &mut rsi_calculator,
        )?);
        data_window.bars = bars;
        println!("data_window.bars.len: {}", data_window.bars.len());
        let len = data_window.bars.len() as i64;
        let window_size = 200.min(data_window.bars.len()) as i64;
        data_window.visible_range = (
            (len - window_size).max(0), // start
            len,                        // end
        );
        data_window.build_extrema_indexes();
        data_window.update_price_range_extrema();
        /*for bar in  &data_window.bars[data_window.bars.len()-50 ..] {
            println!("{:?}", bar);
        }*/
        Ok(())
    }

    pub fn update_price_range_extrema(&mut self) {
        // Check if we need to recalculate
        if let Some(cached_range) = self.cached_visible_range {
            if cached_range == self.visible_range {
                return; // No change, use cached values
            }
        }

        let (start, end) = self.visible_range;
        let start = start.max(0) as usize;
        let end = end.min(self.bars.len() as i64) as usize;

        if start >= end {
            self.price = (0.0, 1.0);
            self.cached_visible_range = Some(self.visible_range);
            return;
        }

        let mut min_price = None;
        let mut max_price = None;

        if let Some(min_indexes) = &self.min_indexes {
            for &i in min_indexes {
                if i >= start && i < end {
                    min_price = Some(self.bars[i].low);
                    break;
                }
            }
        }

        if let Some(max_indexes) = &self.max_indexes {
            for &i in max_indexes {
                if i >= start && i < end {
                    max_price = Some(self.bars[i].high);
                    break;
                }
            }
        }

        // Fallback: перебор по visible_range если не нашли
        if min_price.is_none() || max_price.is_none() {
            let mut fallback_min = f64::MAX;
            let mut fallback_max = f64::MIN;

            for bar in &self.bars[start..end] {
                fallback_min = fallback_min.min(bar.low);
                fallback_max = fallback_max.max(bar.high);
            }

            min_price = Some(fallback_min);
            max_price = Some(fallback_max);
        }

        let min = min_price.unwrap_or(0.0);
        let max = max_price.unwrap_or(1.0);

        self.price = if min >= max {
            (min, min + 1.0)
        } else {
            (min, max)
        };

        // Cache the range we just calculated for
        self.cached_visible_range = Some(self.visible_range);
    }

    pub fn get_max_volume(&mut self) -> f64 {
        // Check if we need to recalculate max volume
        if let Some(cached_range) = self.cached_visible_range {
            if cached_range == self.visible_range {
                if let Some(max_vol) = self.cached_max_volume {
                    return max_vol;
                }
            }
        }

        let (start, end) = self.visible_range;
        let start = start.max(0) as usize;
        let end = end.min(self.bars.len() as i64) as usize;

        if start >= end {
            self.cached_max_volume = Some(0.0);
            return 0.0;
        }

        let max_volume = self.bars[start..end]
            .iter()
            .map(|b| b.volume)
            .fold(0.0, f64::max);

        self.cached_max_volume = Some(max_volume);
        max_volume
    }

    fn build_extrema_indexes(&mut self) {
        let mut mins: Vec<usize> = (0..self.bars.len()).collect();
        let mut maxs: Vec<usize> = (0..self.bars.len()).collect();

        mins.sort_unstable_by(|&a, &b| self.bars[a].low.partial_cmp(&self.bars[b].low).unwrap());

        maxs.sort_unstable_by(|&a, &b| self.bars[b].high.partial_cmp(&self.bars[a].high).unwrap());

        self.min_indexes = Some(mins);
        self.max_indexes = Some(maxs);
    }
}
