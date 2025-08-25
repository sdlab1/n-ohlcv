// db.rs - Database operations, data aggregation system, OHLCV storage
// See CONVENTIONS.md for project structure and workflow

use crate::fetch::KLine;
use crate::fetch::PRICE_MULTIPLIER;
use crate::settings::AGGREGATION_VERSION;
use chrono::{DateTime, Local, TimeZone, Timelike};
use sled;
use std::collections::BTreeMap;
use std::error::Error;

pub struct Database {
    db: sled::Db,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, sled::Error> {
        let config = sled::Config::default()
            .path(path)
            .cache_capacity(4 * 1024 * 1024)
            .use_compression(false);
        config.open().map(|db| Self { db })
    }

    pub fn insert_block(
        &self,
        symbol: &str,
        timestamp: i64,
        data: &[u8],
    ) -> Result<(), Box<dyn Error>> {
        let key = format!("{}_{}", symbol, timestamp);
        self.db.transaction(|tx| {
            // First insert
            match tx.insert(key.as_bytes(), data) {
                Ok(_) => {}
                Err(e) => return Err(sled::transaction::ConflictableTransactionError::Abort(e)),
            }

            // Second insert
            match tx.insert(
                format!("last_{}", symbol).as_bytes(),
                &timestamp.to_be_bytes(),
            ) {
                Ok(_) => {}
                Err(e) => return Err(sled::transaction::ConflictableTransactionError::Abort(e)),
            }

            Ok(())
        })?;

        Ok(())
    }

    pub fn get_block(&self, symbol: &str, timestamp: i64) -> Result<Option<Vec<u8>>, sled::Error> {
        let key = format!("{}_{}", symbol, timestamp);
        match self.db.get(key.as_bytes())? {
            Some(data) => Ok(Some(data.to_vec())),
            None => Ok(None),
        }
    }

    pub fn get_last_timestamp(&self, symbol: &str) -> Result<i64, sled::Error> {
        match self.db.get(format!("last_{}", symbol))? {
            Some(bytes) => Ok(i64::from_be_bytes(bytes.as_ref().try_into().unwrap())),
            None => Ok(0),
        }
    }

    pub fn get_first_timestamp(&self, symbol: &str) -> Result<i64, sled::Error> {
        let prefix = format!("{}_", symbol);
        let iter = self.db.scan_prefix(prefix.as_bytes());

        for result in iter {
            match result {
                Ok((key, _)) => {
                    let key_str = String::from_utf8_lossy(&key);
                    if let Some(timestamp_str) = key_str.strip_prefix(&prefix) {
                        if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                            return Ok(timestamp);
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(0)
    }

    pub fn get_range_data(
        &self,
        symbol: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<Vec<KLine>, Box<dyn Error>> {
        let mut klines = Vec::new();
        let prefix = format!("{}_", symbol);
        let iter = self.db.scan_prefix(prefix.as_bytes());

        for result in iter {
            match result {
                Ok((key, data)) => {
                    let key_str = String::from_utf8_lossy(&key);
                    if let Some(timestamp_str) = key_str.strip_prefix(&prefix) {
                        if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                            if timestamp >= start_time && timestamp <= end_time {
                                let kline: KLine =
                                    bincode::decode_from_slice(&data, bincode::config::standard())?
                                        .0;
                                klines.push(kline);
                            }
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        klines.sort_by_key(|k| k.open_time);
        Ok(klines)
    }

    pub fn aggregate_ohlcv_data(&self, symbol: &str) -> Result<(), Box<dyn Error>> {
        let aggr_symbol = format!("{}_aggr", symbol);
        let version_key = format!("version_{}", aggr_symbol);

        // Проверяем текущую версию
        let current_version = match self.db.get(version_key.as_bytes())? {
            Some(bytes) => i64::from_be_bytes(bytes.as_ref().try_into().unwrap_or([0; 8])),
            None => 0,
        };

        // Если версия новая - переделываем полностью
        if current_version != AGGREGATION_VERSION {
            println!(
                "New aggregation version ({}) detected, rebuilding data {}",
                AGGREGATION_VERSION, aggr_symbol
            );

            // Удаляем старые агрегированные данные
            let aggr_prefix = format!("{}_", aggr_symbol);
            let keys_to_delete: Vec<_> = self
                .db
                .scan_prefix(aggr_prefix.as_bytes())
                .filter_map(|result| result.ok())
                .map(|(key, _)| key)
                .collect();

            for key in keys_to_delete {
                self.db.remove(key)?;
            }

            // Удаляем метаданные
            self.db.remove(format!("last_{}", aggr_symbol).as_bytes())?;
            self.db
                .remove(format!("first_{}", aggr_symbol).as_bytes())?;

            // Устанавливаем новую версию
            self.db
                .insert(version_key.as_bytes(), &AGGREGATION_VERSION.to_be_bytes())?;
        }

        // Получаем границы исходных данных
        let first_timestamp = self.get_first_timestamp(symbol)?;
        let last_timestamp = self.get_last_timestamp(symbol)?;

        if first_timestamp == 0 || last_timestamp == 0 {
            println!("No data available for symbol {}", symbol);
            return Ok(());
        }

        // Получаем последний timestamp агрегированных данных
        let last_aggr_timestamp = self.get_last_timestamp(&aggr_symbol)?;

        let start_time = if last_aggr_timestamp == 0 {
            // Первый запуск - начинаем с начала данных, выравниваем по часам
            let first_dt = DateTime::from_timestamp_millis(first_timestamp).unwrap_or_default();
            let local_dt = first_dt.with_timezone(&Local);
            let hour_aligned = local_dt
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap();
            hour_aligned.timestamp_millis()
        } else {
            // Инкрементальное обновление - начинаем со следующего часа
            last_aggr_timestamp + 3600000 // +1 час в миллисекундах
        };

        // Округляем end_time до полного часа
        let end_dt = DateTime::from_timestamp_millis(last_timestamp).unwrap_or_default();
        let local_end_dt = end_dt.with_timezone(&Local);
        let end_time = local_end_dt
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap()
            .timestamp_millis();

        if start_time > end_time {
            if last_aggr_timestamp == 0 {
                println!("No complete hour data available for aggregation {}", symbol);
            } else {
                println!("No new complete hour data for {}", symbol);
            }
            return Ok(());
        }

        println!(
            "Starting aggregation for {} from {} to {}",
            symbol,
            Local
                .timestamp_millis_opt(start_time)
                .unwrap()
                .format("%H:%M %d %b %Y"),
            Local
                .timestamp_millis_opt(end_time)
                .unwrap()
                .format("%H:%M %d %b %Y")
        );

        // Получаем данные для агрегации
        let klines = self.get_range_data(symbol, start_time, end_time + 3599999)?; // +59:59.999 до конца часа

        if klines.is_empty() {
            println!("No data available for aggregation in specified range");
            return Ok(());
        }

        // Группируем по часам
        let mut hourly_groups: BTreeMap<i64, Vec<&KLine>> = BTreeMap::new();

        for kline in &klines {
            let dt = DateTime::from_timestamp_millis(kline.open_time).unwrap_or_default();
            let local_dt = dt.with_timezone(&Local);
            let hour_timestamp = local_dt
                .with_minute(0)
                .unwrap()
                .with_second(0)
                .unwrap()
                .with_nanosecond(0)
                .unwrap()
                .timestamp_millis();

            hourly_groups.entry(hour_timestamp).or_default().push(kline);
        }

        let mut records_created = 0;
        let mut first_aggr_timestamp = 0i64;
        let mut last_processed_timestamp = 0i64;

        // Создаем агрегированные записи
        for (hour_timestamp, group) in hourly_groups {
            if group.is_empty() {
                continue;
            }

            // Создаем агрегированную OHLCV запись
            let aggregated = KLine {
                open_time: hour_timestamp,
                open: group.first().unwrap().open,
                high: group.iter().map(|k| k.high).max().unwrap_or(0),
                low: group.iter().map(|k| k.low).min().unwrap_or(u64::MAX),
                close: group.last().unwrap().close,
                volume: group.iter().map(|k| k.volume).sum(),
            };

            // Сохраняем агрегированные данные
            let data = bincode::encode_to_vec(&aggregated, bincode::config::standard())?;
            self.insert_block(&aggr_symbol, hour_timestamp, &data)?;

            if first_aggr_timestamp == 0 {
                first_aggr_timestamp = hour_timestamp;
            }
            last_processed_timestamp = hour_timestamp;
            records_created += 1;
        }

        // Сохраняем метаданные о первом timestamp для aggr
        if first_aggr_timestamp > 0 {
            let current_first = match self.db.get(format!("first_{}", aggr_symbol).as_bytes())? {
                Some(bytes) => i64::from_be_bytes(bytes.as_ref().try_into().unwrap_or([0; 8])),
                None => 0,
            };

            if current_first == 0 || first_aggr_timestamp < current_first {
                self.db.insert(
                    format!("first_{}", aggr_symbol).as_bytes(),
                    &first_aggr_timestamp.to_be_bytes(),
                )?;
            }
        }

        if records_created > 0 {
            let final_first = match self.db.get(format!("first_{}", aggr_symbol).as_bytes())? {
                Some(bytes) => i64::from_be_bytes(bytes.as_ref().try_into().unwrap_or([0; 8])),
                None => first_aggr_timestamp,
            };

            println!("Aggregation {} completed successfully:", aggr_symbol);
            println!("  Created records: {}", records_created);
            println!(
                "  First data: {}",
                Local
                    .timestamp_millis_opt(final_first)
                    .unwrap()
                    .format("%H:%M %d %b %Y")
            );
            println!(
                "  Last data: {}",
                Local
                    .timestamp_millis_opt(last_processed_timestamp)
                    .unwrap()
                    .format("%H:%M %d %b %Y")
            );

            // Выводим 5 последних записей для проверки
            self.print_last_aggregated_records(&aggr_symbol, 5)?;
        }

        Ok(())
    }

    pub fn get_aggr_info(&self, symbol: &str) -> Result<(i64, i64), Box<dyn Error>> {
        let aggr_symbol = format!("{}_aggr", symbol);

        let first_timestamp = match self.db.get(format!("first_{}", aggr_symbol).as_bytes())? {
            Some(bytes) => i64::from_be_bytes(bytes.as_ref().try_into().unwrap_or([0; 8])),
            None => 0,
        };

        let last_timestamp = self.get_last_timestamp(&aggr_symbol)?;

        Ok((first_timestamp, last_timestamp))
    }

    fn print_last_aggregated_records(
        &self,
        aggr_symbol: &str,
        count: usize,
    ) -> Result<(), Box<dyn Error>> {
        // Получаем все записи агрегированного символа и сортируем по времени
        let prefix = format!("{}_", aggr_symbol);
        let iter = self.db.scan_prefix(prefix.as_bytes());

        let mut records = Vec::new();

        for result in iter {
            match result {
                Ok((key, data)) => {
                    let key_str = String::from_utf8_lossy(&key);
                    if let Some(timestamp_str) = key_str.strip_prefix(&prefix) {
                        if let Ok(timestamp) = timestamp_str.parse::<i64>() {
                            let kline: KLine =
                                bincode::decode_from_slice(&data, bincode::config::standard())?.0;
                            records.push((timestamp, kline));
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        // Сортируем по времени и берем последние записи
        records.sort_by_key(|(timestamp, _)| *timestamp);
        let last_records: Vec<_> = records.into_iter().rev().take(count).collect();

        if !last_records.is_empty() {
            println!("\n  Last {} records:", last_records.len());
            for (timestamp, kline) in last_records.iter().rev() {
                let datetime = Local.timestamp_millis_opt(*timestamp).unwrap();
                let open = kline.open as f64 / (10_u64.pow(PRICE_MULTIPLIER) as f64);
                let high = kline.high as f64 / (10_u64.pow(PRICE_MULTIPLIER) as f64);
                let low = kline.low as f64 / (10_u64.pow(PRICE_MULTIPLIER) as f64);
                let close = kline.close as f64 / (10_u64.pow(PRICE_MULTIPLIER) as f64);

                println!(
                    "    {} | O:{:.2} H:{:.2} L:{:.2} C:{:.2} V:{:.3}",
                    datetime.format("%H:%M %d.%m.%y"),
                    open,
                    high,
                    low,
                    close,
                    kline.volume
                );
            }
        }

        Ok(())
    }
}
