use crate::db::Database;
use crate::fetch::{KLine, PRICE_MULTIPLIER};
use crate::DataWindow; //  Исправлен импорт
use chrono::{Duration, Utc};
use reqwest::blocking::Client;
use std::error::Error;
use std::thread;
use std::time;
use crate::Bar;

const BLOCK_SIZE: usize = 1000;
const UPDATE_INTERVAL: u64 = 300;

pub struct Timeframe;

impl Timeframe {
    pub fn update_loop(client: &Client, db: &Database, symbol: &str, data_window: &mut DataWindow) -> Result<(), Box<dyn Error>> {
        let mut timer = time::Instant::now();

        loop {
            if timer.elapsed().as_secs() >= UPDATE_INTERVAL {
                match Self::fetch_data_chunk(client, symbol) {
                    Ok(data) => {
                        Self::process_data_chunk(symbol, data, db, data_window)?;
                    },
                    Err(e) => {
                            return Err(e);
                    }
                }
                timer = time::Instant::now();
            }
            thread::sleep(time::Duration::from_secs(10));
        }
    }

    pub fn sync_data(
        pause_between_requests: u64,
        db: &Database,
        symbol: &str,
        start_time: i64,
        end_time: i64,
        data_window: &mut DataWindow,
    ) -> Result<(), Box<dyn Error>> {
        let client = Client::new();
        let mut current_time;
        let last_timestamp = db.get_last_timestamp(symbol).unwrap_or(0);
        if last_timestamp == 0 {
            println!("No data found for {}, initializing with data", symbol);
            current_time = Self::get_dbtimestamp(start_time);
        }
        else {
            current_time = last_timestamp + 60_000_000;
        }
            while current_time < end_time {
                if current_time != start_time {
                    thread::sleep(std::time::Duration::from_secs(pause_between_requests));
                }
                let klines = crate::fetch::fetch_klines(
                    &client,
                    symbol,
                    "1m",
                    1000,
                    Some(current_time),
                    Some(current_time + 60_000_000),
                )?;
                Self::process_data_chunk(symbol, klines, db, data_window)?;
                println!("Initialized data for {} from {}", symbol, current_time);
                current_time += 60_000_000;
            }
        
        Ok(())
    }

    pub fn convert_to_timeframe(
        klines: Vec<KLine>,
        timeframe_minutes: i32,
        dolastbar: bool,
        data_window: &mut DataWindow,
    ) -> Result<Vec<Bar>, Box<dyn Error>> {
        let mut result = Vec::new();
        let mut current_open_time = 0;
        let mut current_open = 0.0;
        let mut current_high = f64::MIN;
        let mut current_low = f64::MAX;
        let mut current_volume = 0.0;
        let mut count = 0;

        // Объединяем остаток с текущими данными
        let mut combined_klines = data_window.timeframe_remainder.to_vec();
        combined_klines.extend(klines);

        for (index, kline) in combined_klines.iter().enumerate() {
            let price_open = kline.open as f64 / 10f64.powi(PRICE_MULTIPLIER as i32);
            let price_high = kline.high as f64 / 10f64.powi(PRICE_MULTIPLIER as i32);
            let price_low = kline.low as f64 / 10f64.powi(PRICE_MULTIPLIER as i32);
            let price_close = kline.close as f64 / 10f64.powi(PRICE_MULTIPLIER as i32);

            if count == 0 {
                current_open_time = kline.open_time;
                current_open = price_open;
            }

            current_high = current_high.max(price_high);
            current_low = current_low.min(price_low);
            current_volume += kline.volume;
            count += 1;

            if count >= timeframe_minutes as usize
            || (dolastbar && index == combined_klines.len() - 1) {
                result.push(Bar {
                    time: current_open_time,
                    open: current_open,
                    high: current_high,
                    low: current_low,
                    close: price_close,
                    volume: current_volume,
                });
                count = 0;
                current_high = f64::MIN;
                current_low = f64::MAX;
                current_volume = 0.0;
            }
        }

        // Обновляем остаток в data_window
        data_window.timeframe_remainder = if count > 0 {
            combined_klines[combined_klines.len() - count..].to_vec()
        } else {
            Vec::new()
        };

        Ok(result)
    }

    pub fn get_dbtimestamp(timestamp_ms: i64) -> i64 {
        timestamp_ms - timestamp_ms % (BLOCK_SIZE as i64 * 60_000)
    }

    fn fetch_data_chunk(client: &Client, symbol: &str) -> Result<Vec<KLine>, Box<dyn Error>> {
        let now = Utc::now().timestamp_millis();
        crate::fetch::fetch_klines(
            client,
            symbol,
            "1m",
            300,
            Some(now - Duration::minutes(5).num_milliseconds()),
            Some(now - 60_000),
        )
    }

    pub fn process_data_chunk(
        symbol: &str,
        data: Vec<KLine>,
        db: &Database,
        dw: &mut DataWindow,
    ) -> Result<(), Box<dyn Error>> {
        if data.len() < 1000 {
            dw.recent_data = data;
            println!("DataWindow.recent_data len {}",
                dw.recent_data.len()
            );
            return Ok(());
        }
        for i in 1..data.len() {
            let time_diff = data[i].open_time - data[i-1].open_time;
            if time_diff != 60_000 {
                return Err(format!(
                    "Consistency check failed for {}: gap between {} and {} is {}ms (expected 60000ms)",
                    symbol,
                    data[i-1].open_time,
                    data[i].open_time,
                    time_diff
                ).into());
            }
        }
        db.insert_block(symbol, data[0].open_time, &data)?;
        Ok(())
    }
}