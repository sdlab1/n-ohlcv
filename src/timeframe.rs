use crate::fetch::fetch_klines; // Убрали неиспользуемый KLine
use crate::compress::compress;
use crate::db::Database;
use reqwest::blocking::Client;
use chrono::{DateTime, Utc, Duration};
use serde_json;

pub struct Timeframe {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub total_minutes: usize,
    pub current_price: String,
}

impl Timeframe {
    pub fn fetch_and_store(client: &Client, db: &Database) -> Result<Self, Box<dyn std::error::Error>> {
        let symbol = "BTCUSDT";
        let interval = "1m";
        let limit = 1000;
        let days_needed = 15;
        let minutes_needed = days_needed * 24 * 60;
        let mut klines = Vec::new();
        
        let mut current_time = match db.get_last_time()? {
            Some(ts) => DateTime::from_timestamp_millis(ts).unwrap(),
            None => Utc::now() - Duration::days(days_needed as i64),
        };

        let start_timestamp = current_time.timestamp_millis();
        let mut last_saved_time = db.get_last_time()?;
        let mut should_break = false;

        while klines.len() < minutes_needed && !should_break {
            // Исправлено: используем std::thread::sleep
            std::thread::sleep(std::time::Duration::from_secs(3));

            let batch = match fetch_klines(client, symbol, interval, limit, Some(current_time.timestamp_millis())) {
                Ok(b) => b,
                Err(e) => {
                    if let Some(status) = e.status() {
                        if status == 429 {
                            eprintln!("Binance API rate limit exceeded!");
                            should_break = true;
                            continue;
                        }
                    }
                    return Err(e.into());
                }
            };

            if let Some(last_ts) = last_saved_time {
                if !batch.is_empty() {
                    let expected = last_ts + 60_000;
                    if batch[0].open_time != expected {
                        eprintln!("Time gap detected! Expected: {}, Got: {}", expected, batch[0].open_time);
                        should_break = true;
                        continue;
                    }
                }
            }

            if !batch.is_empty() {
                klines.extend(batch);
                last_saved_time = klines.last().map(|k| k.open_time);
                current_time = DateTime::from_timestamp_millis(klines.last().unwrap().open_time).unwrap();
            }
        }

        for chunk in klines.chunks(1000) {
            let first = chunk[0].open_time;
            let last = chunk.last().unwrap().open_time;
            
            if (last - first) != (chunk.len() as i64 - 1) * 60_000 {
                return Err("Invalid time sequence in chunk".into());
            }

            let serialized = serde_json::to_vec(chunk)?;
            let compressed = compress(&serialized);
            db.insert(&first.to_be_bytes(), &compressed)?;
            db.set_last_time(last)?;
        }

        let start_date = DateTime::from_timestamp_millis(klines[0].open_time).unwrap();
        let end_date = DateTime::from_timestamp_millis(klines.last().unwrap().open_time).unwrap();
        let current_price = klines.last().unwrap().close.clone();

        Ok(Timeframe {
            start_date,
            end_date,
            total_minutes: klines.len(),
            current_price,
        })
    }
}