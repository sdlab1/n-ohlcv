use sled;

pub struct Database {
    db: sled::Db,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        println!("Database initialized at {}", path);
        Ok(Database { db })
    }

    pub fn insert(&self, key: &[u8], value: &[u8]) -> Result<(), sled::Error> {
        self.db.insert(key, value)?;
        println!("Inserted data with key: {:?}", key);
        Ok(())
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<sled::IVec>, sled::Error> {
        self.db.get(key)
    }

    pub fn len(&self) -> usize {
        self.db.len()
    }

    // Добавленные методы для работы с lasttime
    pub fn set_last_time(&self, timestamp: i64) -> Result<(), sled::Error> {
        self.db.insert(b"lasttime", &timestamp.to_be_bytes())?;
        Ok(())
    }

    pub fn get_last_time(&self) -> Result<Option<i64>, sled::Error> {
        match self.db.get(b"lasttime")? {
            Some(bytes) => {
                let bytes: [u8; 8] = bytes.as_ref().try_into().unwrap();
                Ok(Some(i64::from_be_bytes(bytes)))
            }
            None => Ok(None),
        }
    }
}