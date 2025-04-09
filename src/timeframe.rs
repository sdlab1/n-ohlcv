use crate::db::Database;
use crate::fetch::KLine;
use crate::compress::compress_klines;
use chrono::{Duration, Utc};
use lazy_static::lazy_static;
use reqwest::blocking::Client;
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

const BLOCK_SIZE: usize = 1000;
const UPDATE_INTERVAL: u64 = 300; // 5 minutes between updates
const MAX_RETRIES: u32 = 3;

lazy_static! {
    static ref MEMORY_CACHE: Arc<Mutex<BTreeMap<(String, i64), Vec<KLine>>>> = 
        Arc::new(Mutex::new(BTreeMap::new()));
}

pub struct Timeframe;

impl Timeframe {
    pub fn run_forever(
        client: &Client,
        db: &Database,
        symbol: &str,
    ) -> Result<(), Box<dyn Error>> {
        let _last_processed_block = db.get_last_block(symbol)?;
        let mut timer = time::Instant::now();
        let mut retry_count = 0;

        loop {
            if timer.elapsed().as_secs() >= UPDATE_INTERVAL {
                match Self::fetch_with_retry(client, symbol, MAX_RETRIES) {
                    Ok(data) => {
                        retry_count = 0;
                        if let Err(e) = Self::process_data_chunk(symbol, data, db) {
                            eprintln!("[{}] Processing error: {}", symbol, e);
                        }
                    },
                    Err(e) => {
                        retry_count += 1;
                        eprintln!("[{}] Fetch error (attempt {}/{}): {}", 
                            symbol, retry_count, MAX_RETRIES, e);
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

    pub fn fetch_and_process(
        client: &Client,
        db: &Database,
        symbol: &str,
    ) -> Result<(), Box<dyn Error>> {
        let data = Self::fetch_data_chunk(client, symbol)?;
        Self::process_data_chunk(symbol, data, db)
    }

    fn fetch_with_retry(
        client: &Client,
        symbol: &str,
        max_retries: u32,
    ) -> Result<Vec<KLine>, Box<dyn Error>> {
        let mut last_error = None;
        
        for attempt in 0..max_retries {
            match Self::fetch_data_chunk(client, symbol) {
                Ok(data) => return Ok(data),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < max_retries - 1 {
                        let delay = 2u64.pow(attempt) * 5; // Exponential backoff
                        thread::sleep(time::Duration::from_secs(delay));
                    }
                }
            }
        }
        
        Err(last_error.unwrap())
    }

    fn fetch_data_chunk(
        client: &Client,
        symbol: &str,
    ) -> Result<Vec<KLine>, Box<dyn Error>> {
        let now = Utc::now().timestamp_millis();
        let start_time = now - Duration::minutes(5).num_milliseconds();

        let data = crate::fetch::fetch_klines(client, symbol, "1m", 300, Some(start_time))?;
        
        if data.is_empty() {
            return Err("Empty data chunk received".into());
        }

        // Validate timestamps
        for i in 1..data.len() {
            if data[i].open_time != data[i-1].open_time + 60_000 {
                return Err(format!(
                    "Time gap detected in {} between {} and {}", 
                    symbol, 
                    data[i-1].open_time, 
                    data[i].open_time
                ).into());
            }
        }

        Ok(data)
    }

    fn process_data_chunk(
        symbol: &str,
        data: Vec<KLine>,
        db: &Database,
    ) -> Result<(), Box<dyn Error>> {
        let mut cache = MEMORY_CACHE.lock().unwrap();

        for kline in data {
            let block_start = (kline.open_time / 60_000) / BLOCK_SIZE as i64 * BLOCK_SIZE as i64;
            let cache_key = (symbol.to_string(), block_start);

            // Check for duplicates
            if let Some(existing) = cache.get(&cache_key) {
                if existing.iter().any(|x| x.open_time == kline.open_time) {
                    continue;
                }
            }

            cache.entry(cache_key.clone())
                .and_modify(|v| v.push(kline.clone()))
                .or_insert_with(|| vec![kline]);

            // Process full blocks
            if let Some(block) = cache.get(&cache_key) {
                if block.len() >= BLOCK_SIZE {
                    let compressed = compress_klines(block);
                    
                    // Atomic check and insert
                    if db.get_ticker_data(symbol, block_start)?.is_none() {
                        db.insert_ticker_data(symbol, block_start, &compressed)?;
                        db.set_last_block(symbol, block_start)?;
                        println!("[{}] Saved block {} ({} candles)", 
                            symbol, block_start, block.len());
                    }

                    cache.remove(&cache_key);
                }
            }
        }
        
        Ok(())
    }

    pub fn get_cached_blocks(symbol: &str) -> Vec<(i64, usize)> {
        MEMORY_CACHE.lock().unwrap()
            .iter()
            .filter(|((s, _), _)| s == symbol)
            .map(|((_, ts), data)| (*ts, data.len()))
            .collect()
    }
}