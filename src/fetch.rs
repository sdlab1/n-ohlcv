use reqwest::blocking::Client;
use serde_json;
use std::error::Error;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, bincode::Encode, bincode::Decode)]
pub struct KLine {
    pub open_time: i64,
    pub open: u64,
    pub high: u64,
    pub low: u64,
    pub close: u64,
    pub volume: f64,
}

pub const PRICE_MULTIPLIER: u32 = 2;

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
        .map(|k| {
            let open_time = k[0].as_i64().unwrap_or(0);
            let open = convert_price_to_u64(k[1].as_str().unwrap_or("0"));
            let high = convert_price_to_u64(k[2].as_str().unwrap_or("0"));
            let low = convert_price_to_u64(k[3].as_str().unwrap_or("0"));
            let close = convert_price_to_u64(k[4].as_str().unwrap_or("0"));
            let volume = k[5].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
            KLine {
                open_time,
                open,
                high,
                low,
                close,
                volume,
            }
        })
        .collect();

    Ok(klines)
}

fn convert_price_to_u64(price_str: &str) -> u64 {
    let parts: Vec<&str> = price_str.split('.').collect();
    let integer_part = parts[0];
    let decimal_part = if parts.len() > 1 { parts[1] } else { "" };
    
    let mut result = integer_part.to_string();
    if PRICE_MULTIPLIER > 0 {
        let decimals_to_take = decimal_part.chars().take(PRICE_MULTIPLIER as usize).collect::<String>();
        let padding = "0".repeat((PRICE_MULTIPLIER as usize).saturating_sub(decimals_to_take.len()));
        result += &decimals_to_take;
        result += &padding;
    }
    
    result.parse::<u64>().unwrap_or(0)
}