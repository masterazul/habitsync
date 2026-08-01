use std::collections::BTreeMap;

use crate::model::{Checkin, Habit};

pub trait Item: Clone {
    fn id(&self) -> &str;
    fn updated_at(&self) -> u64;
    fn deleted(&self) -> bool;
    fn seq(&self) -> u64;
    fn set_seq(&mut self, seq: u64);
}

impl Item for Habit {
    fn id(&self) -> &str {
        &self.id
    }
    fn updated_at(&self) -> u64 {
        self.updated_at
    }
    fn deleted(&self) -> bool {
        self.deleted
    }
    fn seq(&self) -> u64 {
        self.seq
    }
    fn set_seq(&mut self, seq: u64) {
        self.seq = seq;
    }
}

impl Item for Checkin {
    fn id(&self) -> &str {
        &self.id
    }
    fn updated_at(&self) -> u64 {
        self.updated_at
    }
    fn deleted(&self) -> bool {
        self.deleted
    }
    fn seq(&self) -> u64 {
        self.seq
    }
    fn set_seq(&mut self, seq: u64) {
        self.seq = seq;
    }
}

pub fn apply<T: Item>(
    store: &mut BTreeMap<String, T>,
    incoming: Vec<T>,
    counter: &mut u64,
) -> usize {
    let mut applied = 0;
    for mut record in incoming {
        let accept = match store.get(record.id()) {
            None => true,
            Some(existing) => {
                record.updated_at() > existing.updated_at()
                    || (record.updated_at() == existing.updated_at()
                        && record.deleted()
                        && !existing.deleted())
            }
        };
        if accept {
            *counter += 1;
            record.set_seq(*counter);
            store.insert(record.id().to_string(), record);
            applied += 1;
        }
    }
    applied
}

pub fn changes_since<T: Item>(store: &BTreeMap<String, T>, cursor: u64) -> Vec<T> {
    let mut changes: Vec<T> = store
        .values()
        .filter(|r| r.seq() > cursor)
        .cloned()
        .collect();
    changes.sort_by_key(|r| r.seq());
    changes
}
