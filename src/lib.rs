use std::collections::BTreeMap;

pub struct KvStore {
    pub map: BTreeMap<String, String>,
}

impl KvStore {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
    pub fn set(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }
    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }
    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(&key).map(|value| value.clone())
    }
}
