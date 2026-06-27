use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::model::{Checkin, Habit};

#[derive(Default, Serialize, Deserialize)]
pub struct Data {
    pub counter: u64,
    pub habits: BTreeMap<String, Habit>,
    pub checkins: BTreeMap<String, Checkin>,
}

pub struct Store {
    path: Option<PathBuf>,
    data: Mutex<Data>,
}

impl Store {
    pub fn open(path: Option<PathBuf>) -> Self {
        let data = path
            .as_ref()
            .and_then(|p| std::fs::read(p).ok())
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Store {
            path,
            data: Mutex::new(data),
        }
    }

    pub fn write<R>(&self, f: impl FnOnce(&mut Data) -> R) -> R {
        let mut guard = self.data.lock().unwrap();
        let result = f(&mut guard);
        if let Some(path) = &self.path {
            if let Ok(bytes) = serde_json::to_vec_pretty(&*guard) {
                let _ = std::fs::write(path, bytes);
            }
        }
        result
    }

    pub fn read<R>(&self, f: impl FnOnce(&Data) -> R) -> R {
        f(&self.data.lock().unwrap())
    }
}
