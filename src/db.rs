// db.rs
use sled;
use crate::fetch::KLine;
use crate::compress;
use std::error::Error;

pub struct Database {
    db: sled::Db,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, sled::Error> {
        sled::open(path).map(|db| Self { db })
    }

    pub fn insert_block(
        &self,
        symbol: &str,
        timestamp: i64,
        data: &[KLine],
    ) -> Result<(), Box<dyn Error>> {
        let key = format!("{}_{}", symbol, timestamp);
        let compressed = compress::compress_klines(data)?;
        self.db.transaction(|tx| {
            // First insert
            match tx.insert(key.as_bytes(), compressed.clone()) {
                Ok(_) => {},
                Err(e) => return Err(sled::transaction::ConflictableTransactionError::Abort(e)),
            }
            
            // Second insert
            match tx.insert(format!("last_{}", symbol).as_bytes(), &timestamp.to_be_bytes()) {
                Ok(_) => {},
                Err(e) => return Err(sled::transaction::ConflictableTransactionError::Abort(e)),
            }
            
            Ok(())
        })?;
        
        Ok(())
    }

    pub fn get_block(&self, symbol: &str, timestamp: i64) -> Result<Option<Vec<KLine>>, Box<dyn std::error::Error>> {
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