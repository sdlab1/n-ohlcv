// db.rs
use crate::compress;
use crate::fetch::KLine;
use sled;
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
        data: &[KLine],
    ) -> Result<(), Box<dyn Error>> {
        let key = format!("{}_{}", symbol, timestamp);
        let compressed = compress::compress_klines(data)?;
        let last_key = format!("last_{}", symbol);
        let key_bytes = key.as_bytes();
        let last_ts_bytes = timestamp.to_be_bytes();
        self.db.transaction(|tx| {
            tx.insert(key_bytes, &compressed[..])?;
            tx.insert(last_key.as_bytes(), &last_ts_bytes)?;
            Ok::<(), sled::transaction::ConflictableTransactionError<sled::Error>>(())
        })?;
        Ok(())
    }

    pub fn get_block(
        &self,
        symbol: &str,
        timestamp: i64,
    ) -> Result<Option<Vec<KLine>>, Box<dyn std::error::Error>> {
        let key = format!("{}_{}", symbol, timestamp);
        match self.db.get(key.as_bytes())? {
            Some(data) => Ok(Some(compress::decompress_klines(&data)?)),
            None => Ok(None),
        }
    }

    pub fn get_last_timestamp(&self, symbol: &str) -> Result<i64, sled::Error> {
        match self.db.get(format!("last_{}", symbol))? {
            Some(bytes) => Ok(i64::from_be_bytes(bytes.as_ref().try_into().unwrap())),
            None => Ok(0),
        }
    }
}
