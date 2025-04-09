use sled;

pub struct Database {
    db: sled::Db,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        Ok(Database { db })
    }

    pub fn insert_ticker_data(&self, symbol: &str, timestamp: i64, data: &[u8]) -> Result<(), sled::Error> {
        let key = format!("{}_{}", symbol, timestamp);
        self.db.insert(key.as_bytes(), data)?;
        Ok(())
    }

    pub fn get_ticker_data(&self, symbol: &str, timestamp: i64) -> Result<Option<sled::IVec>, sled::Error> {
        let key = format!("{}_{}", symbol, timestamp);
        self.db.get(key.as_bytes())
    }

    pub fn set_last_block(&self, symbol: &str, timestamp: i64) -> Result<(), sled::Error> {
        let key = format!("last_block_{}", symbol);
        self.db.insert(key.as_bytes(), &timestamp.to_be_bytes())?;
        Ok(())
    }

    pub fn get_last_block(&self, symbol: &str) -> Result<i64, sled::Error> {
        let key = format!("last_block_{}", symbol);
        match self.db.get(key.as_bytes())? {
            Some(bytes) => {
                let arr: [u8; 8] = bytes.as_ref().try_into().unwrap();
                Ok(i64::from_be_bytes(arr))
            },
            None => Ok(0)
        }
    }

    pub fn get_all_symbols(&self) -> Result<Vec<String>, sled::Error> {
        let mut symbols = std::collections::HashSet::new();
        for key in self.db.scan_prefix("last_block_") {
            let key_entry = key?;
            let key_str = String::from_utf8_lossy(&key_entry.0);
            if let Some(symbol) = key_str.strip_prefix("last_block_") {
                symbols.insert(symbol.to_string());
            }
        }
        Ok(symbols.into_iter().collect())
    }
}