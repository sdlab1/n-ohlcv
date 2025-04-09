// fetch.rs
use reqwest::blocking::Client;
use serde_json;
use std::error::Error;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KLine {
    pub open_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
}

pub fn fetch_klines(
    client: &Client,
    symbol: &str,
    interval: &str,
    limit: i64,
    start_time: Option<i64>,
    end_time: Option<i64>,
) -> Result<Vec<KLine>, Box<dyn Error>> {
    let mut url = format!(
        "https://api.binance.com/api/v3/klines?symbol={}&interval={}&limit={}",
        symbol, interval, limit
    );

    if let Some(start) = start_time {
        url.push_str(&format!("&startTime={}", start));
    }
    if let Some(end) = end_time {
        url.push_str(&format!("&endTime={}", end));
    }

    let response = client.get(&url).send()?;
    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()).into());
    }

    let klines = response.json::<Vec<Vec<serde_json::Value>>>()?
        .into_iter()
        .map(|k| KLine {
            open_time: k[0].as_i64().unwrap_or(0),
            open: k[1].as_str().unwrap_or("0").to_string(),
            high: k[2].as_str().unwrap_or("0").to_string(),
            low: k[3].as_str().unwrap_or("0").to_string(),
            close: k[4].as_str().unwrap_or("0").to_string(),
            volume: k[5].as_str().unwrap_or("0").to_string(),
        })
        .collect();

    Ok(klines)
}