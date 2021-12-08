use crate::*;

pub trait IndexedAndClearable {
    ///
    fn set_index(&mut self, index: &u64);
    ///
    fn clear_extra_storage(&mut self);
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct IndexedHistories<T: BorshDeserialize + BorshSerialize + IndexedAndClearable> {
    /// The anchor event data map.
    histories: LookupMap<u64, T>,
    /// The start index of valid anchor event.
    start_index: u64,
    /// The end index of valid anchor event.
    end_index: u64,
}

impl<T> IndexedHistories<T>
where
    T: BorshDeserialize + BorshSerialize + IndexedAndClearable,
{
    ///
    pub fn new(storage_key: StorageKey) -> Self {
        Self {
            histories: LookupMap::new(storage_key.into_bytes()),
            start_index: 0,
            end_index: 0,
        }
    }
    ///
    pub fn migrate_from(storage_key: StorageKey, start_index: u64, end_index: u64) -> Self {
        Self {
            histories: LookupMap::new(storage_key.into_bytes()),
            start_index,
            end_index,
        }
    }
    ///
    pub fn get(&self, index: &u64) -> Option<T> {
        self.histories.get(index)
    }
    ///
    pub fn contains(&self, era_number: &u64) -> bool {
        self.histories.contains_key(era_number)
    }
    ///
    pub fn insert(&mut self, era_number: &u64, record: &T) {
        self.histories.insert(era_number, record);
        if *era_number > self.end_index {
            self.end_index = *era_number;
        }
    }
    ///
    pub fn index_range(&self) -> IndexRange {
        IndexRange {
            start_index: U64::from(self.start_index),
            end_index: U64::from(self.end_index),
        }
    }
    ///
    pub fn append(&mut self, record: &mut T) -> T {
        let index = match self.histories.contains_key(&0) {
            true => self.end_index + 1,
            false => 0,
        };
        record.set_index(&index);
        self.histories.insert(&index, &record);
        self.end_index = index;
        self.histories.get(&index).unwrap()
    }
    ///
    pub fn remove_before(&mut self, index: &u64) {
        if self.start_index >= *index {
            return;
        }
        for index in self.start_index..*index {
            self.histories.remove(&index);
        }
        self.start_index = *index;
    }
    ///
    pub fn reset_to(&mut self, index: &u64) {
        assert!(
            *index >= self.start_index && *index <= self.end_index,
            "Invalid history data index."
        );
        for index in (*index + 1)..self.end_index + 1 {
            self.remove_at(&index);
        }
        self.end_index = *index;
    }
    ///
    pub fn clear(&mut self) {
        for index in self.start_index..self.end_index + 1 {
            self.remove_at(&index);
        }
        self.start_index = 0;
        self.end_index = 0;
    }
    ///
    fn remove_at(&mut self, index: &u64) {
        if let Some(mut record) = self.histories.get(index) {
            record.clear_extra_storage();
            self.histories.remove(index);
        }
    }
}
