use reqwest::blocking::Client;
use serde;
use serde_json;
use std::error::Error;

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
    //println!("fetch url: {url}");
    let response = client.get(&url).send()?;
    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()).into());
    }

    let klines = response
        .json::<Vec<Vec<serde_json::Value>>>()?
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
    // A multiplier to convert the decimal part to an integer.
    // For PRICE_MULTIPLIER = 2, this is 100.
    const MULT: u64 = 10u64.pow(PRICE_MULTIPLIER);

    if let Some(dot_pos) = price_str.find('.') {
        // Integer part
        let integer_part = price_str[..dot_pos].parse::<u64>().unwrap_or(0);

        // Decimal part
        let decimal_part_str = &price_str[dot_pos + 1..];
        // Take only the required number of decimals
        let num_decimals = decimal_part_str.len().min(PRICE_MULTIPLIER as usize);
        let decimal_part = if num_decimals > 0 {
            decimal_part_str[..num_decimals].parse::<u64>().unwrap_or(0)
        } else {
            0
        };

        // If the provided decimal part is shorter, pad with zeros mathematically.
        // e.g., if price is "1.2" and PRICE_MULTIPLIER is 2,
        // decimal_part is 2, num_decimals is 1.
        // We need to make it 20. So, 2 * 10^(2-1) = 20.
        let padding_power = (PRICE_MULTIPLIER as usize).saturating_sub(num_decimals);
        let adjusted_decimal = decimal_part * 10u64.pow(padding_power as u32);

        integer_part * MULT + adjusted_decimal
    } else {
        // No decimal point, just integer.
        // e.g., if price is "12" and PRICE_MULTIPLIER is 2, result is 1200.
        price_str.parse::<u64>().unwrap_or(0) * MULT
    }
}
