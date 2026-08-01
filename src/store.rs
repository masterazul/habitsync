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
    pub fn open(path: Option<PathBuf>) -> Result<Self, String> {
        let data = match &path {
            Some(p) => match std::fs::read(p) {
                Ok(bytes) => serde_json::from_slice(&bytes)
                    .map_err(|e| format!("could not parse {}: {e}", p.display()))?,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Data::default(),
                Err(e) => return Err(format!("could not read {}: {e}", p.display())),
            },
            None => Data::default(),
        };
        Ok(Store {
            path,
            data: Mutex::new(data),
        })
    }

    pub fn write<R>(&self, f: impl FnOnce(&mut Data) -> R) -> Result<R, String> {
        let mut guard = self.data.lock().unwrap();
        let result = f(&mut guard);
        if let Some(path) = &self.path {
            let bytes = serde_json::to_vec_pretty(&*guard).map_err(|e| e.to_string())?;
            let mut tmp = path.clone().into_os_string();
            tmp.push(format!(".{}.tmp", std::process::id()));
            let tmp = PathBuf::from(tmp);
            if let Err(e) = std::fs::write(&tmp, &bytes) {
                let _ = std::fs::remove_file(&tmp);
                return Err(format!("could not write {}: {e}", tmp.display()));
            }
            if let Err(e) = std::fs::rename(&tmp, path) {
                let _ = std::fs::remove_file(&tmp);
                return Err(format!("could not persist {}: {e}", path.display()));
            }
        }
        Ok(result)
    }

    pub fn read<R>(&self, f: impl FnOnce(&Data) -> R) -> R {
        f(&self.data.lock().unwrap())
    }
}
