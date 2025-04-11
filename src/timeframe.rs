use crate::db::Database;
use crate::fetch::{KLine, PRICE_MULTIPLIER};
use chrono::{Duration, Utc};
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

const BLOCK_SIZE: usize = 1000;
const UPDATE_INTERVAL: u64 = 300;
const MAX_RETRIES: u32 = 3;

lazy_static! {
    static ref MEMORY_CACHE: Arc<Mutex<BTreeMap<(String, i64), Vec<KLine>>>> = 
        Arc::new(Mutex::new(BTreeMap::new()));
}

pub struct Timeframe;

impl Timeframe {
    pub fn update_loop(client: &Client, db: &Database, symbol: &str) -> Result<(), Box<dyn Error>> {
        let mut timer = time::Instant::now();
        let mut retry_count = 0;

        loop {
            if timer.elapsed().as_secs() >= UPDATE_INTERVAL {
                match Self::fetch_with_retry(client, symbol, MAX_RETRIES) {
                    Ok(data) => {
                        retry_count = 0;
                        Self::process_data_chunk(symbol, data, db)?;
                    },
                    Err(e) => {
                        retry_count += 1;
                        if retry_count >= MAX_RETRIES {
                            return Err(e);
                        }
                    }
                }
                timer = time::Instant::now();
            }
            thread::sleep(time::Duration::from_secs(10));
        }
    }

    pub fn get_data_window(
        db: &Database,
        symbol: &str,
        start_time: i64,
        end_time: i64,
        timeframe_minutes: i32,
    ) -> Result<crate::DataWindow, Box<dyn Error>> {
        println!("get_data_window: symbol = {}, start_time = {}, end_time = {}, timeframe = {}", symbol, start_time, end_time, timeframe_minutes);

        Self::init_db_if_needed(3, db, symbol, start_time, end_time)?;

        let mut bars = Vec::new();
        let mut current_block_start = start_time + start_time % (BLOCK_SIZE as i64 * 60_000);

        while current_block_start <= end_time {
            println!("Checking block at timestamp: {}", current_block_start);
            if let Some(block) = db.get_block(symbol, current_block_start)? {
                let converted = Self::convert_to_timeframe(block, timeframe_minutes)?;
                println!("Block at {} has {} bars after conversion", current_block_start, converted.len());
                bars.extend(converted);
            } else {
                println!("No data for block at {}", current_block_start);
            }
            current_block_start += BLOCK_SIZE as i64 * 60_000;
        }
        println!("Total bars collected: {}", bars.len());
        Ok(crate::DataWindow {
            bars,
            visible_range: (0.0, 1.0),
            volume_height_ratio: 0.2,
        })
    }

    fn init_db_if_needed(
        pause_between_requests: u64,
        db: &Database,
        symbol: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<(), Box<dyn Error>> {
        let last_timestamp = db.get_last_timestamp(symbol).unwrap_or(0);
        if last_timestamp == 0 {
            println!("No data found for {}, initializing with 7 days of data", symbol);
            let client = Client::new();
            let mut current_time = start_time  + start_time % (BLOCK_SIZE as i64 * 60_000);
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
                    Some((current_time + 60_000_000)),
                )?;
                Self::process_data_chunk(symbol, klines, db)?;
                println!("Initialized data for {} from {}", symbol, current_time);
                current_time += 60_000_000;

            }
        }
        Ok(())
    }

    fn convert_to_timeframe(klines: Vec<KLine>, timeframe_minutes: i32) -> Result<Vec<crate::Bar>, Box<dyn Error>> {
        let mut result = Vec::new();
        let mut current_open_time = 0;
        let mut current_open = 0.0;
        let mut current_high = f64::MIN;
        let mut current_low = f64::MAX;
        let mut current_volume = 0.0;
        let mut count = 0;

        for kline in klines {
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

            if count >= timeframe_minutes as usize {
                result.push(crate::Bar {
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

        Ok(result)
    }

    fn fetch_with_retry(client: &Client, symbol: &str, max_retries: u32) -> Result<Vec<KLine>, Box<dyn Error>> {
        for attempt in 0..max_retries {
            match Self::fetch_data_chunk(client, symbol) {
                Ok(data) => return Ok(data),
                Err(e) if attempt == max_retries - 1 => return Err(e),
                Err(_) => thread::sleep(time::Duration::from_secs(2u64.pow(attempt) * 5)),
            }
        }
        unreachable!()
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

    fn process_data_chunk(symbol: &str, data: Vec<KLine>, db: &Database) -> Result<(), Box<dyn Error>> {
        let mut cache = MEMORY_CACHE.lock().unwrap();

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

        for kline in data {
            let block_start = (kline.open_time / 60_000 / BLOCK_SIZE as i64) * BLOCK_SIZE as i64 * 60_000;
            let cache_key = (symbol.to_string(), block_start);

            cache.entry(cache_key.clone())
                .and_modify(|v| v.push(kline.clone()))
                .or_insert_with(|| vec![kline.clone()]);
        }

        for (cache_key, block) in cache.iter() {
            if !block.is_empty() {
                println!("Saving block for {} at {} with {} items", symbol, cache_key.1, block.len());
                db.insert_block(symbol, cache_key.1, block)?;
            }
        }
        cache.clear();
        Ok(())
    }
}