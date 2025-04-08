use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct KLine {
    pub open_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
}

pub fn fetch_klines(client: &Client, symbol: &str, interval: &str, limit: i64, start_time: Option<i64>) -> Result<Vec<KLine>, reqwest::Error> {
    let mut url = format!(
        "https://api.binance.com/api/v3/klines?symbol={}&interval={}&limit={}",
        symbol, interval, limit
    );
    if let Some(start) = start_time {
        url.push_str(&format!("&startTime={}", start));
    }

    let response = client.get(&url).send()?;
    
    // Проверка статуса ошибки
    if response.status() == 429 {
        return Err(response.error_for_status().unwrap_err());
    }

    let response = response.json::<Vec<Vec<serde_json::Value>>>()?;
    
    let klines: Vec<KLine> = response
        .into_iter()
        .map(|k| KLine {
            open_time: k[0].as_i64().unwrap(),
            open: k[1].as_str().unwrap().to_string(),
            high: k[2].as_str().unwrap().to_string(),
            low: k[3].as_str().unwrap().to_string(),
            close: k[4].as_str().unwrap().to_string(),
            volume: k[5].as_str().unwrap().to_string(),
        })
        .collect();

    let filtered_klines = klines[..klines.len().saturating_sub(1)].to_vec();
    println!("Fetched {} klines from Binance API", filtered_klines.len());
    Ok(filtered_klines)
}