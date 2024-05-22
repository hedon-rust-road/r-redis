use std::{collections::HashSet, ops::Deref, sync::Arc};

use dashmap::{DashMap, DashSet};

use crate::{BulkString, RespFrame};

#[derive(Debug, Clone)]
pub struct Backend(Arc<BackendInner>);

#[derive(Debug)]
pub struct BackendInner {
    pub(crate) map: DashMap<String, RespFrame>,
    pub(crate) hmap: DashMap<String, DashMap<String, RespFrame>>,
    pub(crate) set: DashMap<String, DashSet<BulkString>>,
}

impl Deref for Backend {
    type Target = BackendInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Backend {
    fn default() -> Self {
        Self(Arc::new(BackendInner::default()))
    }
}

impl Default for BackendInner {
    fn default() -> Self {
        Self {
            map: DashMap::new(),
            hmap: DashMap::new(),
            set: DashMap::new(),
        }
    }
}

impl Backend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, key: &str) -> Option<RespFrame> {
        self.map.get(key).map(|v| v.value().clone())
    }

    pub fn set(&self, key: String, value: RespFrame) {
        self.map.insert(key, value);
    }

    pub fn hget(&self, key: &str, field: &str) -> Option<RespFrame> {
        self.hmap
            .get(key)
            .and_then(|v| v.get(field).map(|v| v.value().clone()))
    }

    pub fn hset(&self, key: String, field: String, value: RespFrame) {
        let hmap = self.hmap.entry(key).or_default();
        hmap.insert(field, value);
    }

    pub fn hgetall(&self, key: &str) -> Option<DashMap<String, RespFrame>> {
        self.hmap.get(key).map(|v| v.clone())
    }

    pub fn hmget(&self, key: &str, fields: &[String]) -> DashMap<String, RespFrame> {
        let map = DashMap::new();
        if let Some(v) = self.hmap.get(key) {
            for field in fields {
                if let Some(v) = v.get(field) {
                    map.insert(field.clone(), v.value().clone());
                }
            }
        }
        map
    }

    pub fn sadd(&self, key: String, member: HashSet<BulkString>) -> i64 {
        let mut res = 0;
        let set = self.set.entry(key).or_default();
        for k in member {
            if set.insert(k) {
                res += 1
            }
        }
        res
    }

    pub fn is_member(&self, key: String, member: BulkString) -> i64 {
        if let Some(set) = self.set.get(&key) {
            if set.contains(&member) {
                return 1;
            } else {
                return 0;
            }
        }
        0
    }
}
