use crate::*;

pub trait IndexedAndClearable {
    ///
    fn set_index(&mut self, index: &u64);
    ///
    fn clear_extra_storage(&mut self);
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct LookupArray<T: BorshDeserialize + BorshSerialize + IndexedAndClearable> {
    /// The anchor event data map.
    pub lookup_map: LookupMap<u64, T>,
    /// The start index of valid anchor event.
    pub start_index: u64,
    /// The end index of valid anchor event.
    pub end_index: u64,
}

impl<T> LookupArray<T>
where
    T: BorshDeserialize + BorshSerialize + IndexedAndClearable,
{
    ///
    pub fn new(storage_key: StorageKey) -> Self {
        Self {
            lookup_map: LookupMap::new(storage_key.into_bytes()),
            start_index: 0,
            end_index: 0,
        }
    }
    ///
    pub fn migrate_from(storage_key: StorageKey, start_index: u64, end_index: u64) -> Self {
        Self {
            lookup_map: LookupMap::new(storage_key.into_bytes()),
            start_index,
            end_index,
        }
    }
    ///
    pub fn get(&self, index: &u64) -> Option<T> {
        self.lookup_map.get(index)
    }
    ///
    pub fn get_slice_of(&self, start_index: &u64, quantity: Option<u64>) -> Vec<T> {
        let mut results = Vec::<T>::new();
        let start_index = match self.start_index > *start_index {
            true => self.start_index,
            false => *start_index,
        };
        let mut end_index = start_index
            + match quantity {
                Some(quantity) => match quantity > 50 {
                    true => 49,
                    false => quantity - 1,
                },
                None => 49,
            };
        end_index = match end_index < self.end_index {
            true => end_index,
            false => self.end_index,
        };
        for index in start_index..end_index + 1 {
            if let Some(record) = self.get(&index) {
                results.push(record);
            }
        }
        results
    }
    ///
    pub fn contains(&self, index: &u64) -> bool {
        self.lookup_map.contains_key(index)
    }
    ///
    pub fn insert(&mut self, index: &u64, record: &T) {
        self.lookup_map.insert(index, record);
        if *index > self.end_index {
            self.end_index = *index;
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
        let index = match self.lookup_map.contains_key(&0) {
            true => self.end_index + 1,
            false => 0,
        };
        record.set_index(&index);
        self.lookup_map.insert(&index, &record);
        self.end_index = index;
        self.lookup_map.get(&index).unwrap()
    }
    ///
    pub fn remove_before(&mut self, index: &u64) {
        if self.start_index >= *index {
            return;
        }
        for index in self.start_index..*index {
            self.remove_at(&index);
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
    pub fn clear(&mut self) -> MultiTxsOperationProcessingResult {
        log!(
            "Index range of lookup array: {} - {}",
            self.start_index,
            self.end_index
        );
        let mut index = self.start_index;
        while index <= self.end_index
            && env::used_gas() < Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING)
        {
            self.remove_at(&index);
            index += 1;
        }
        if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
            self.start_index = index;
            MultiTxsOperationProcessingResult::NeedMoreGas
        } else {
            self.start_index = 0;
            self.end_index = 0;
            MultiTxsOperationProcessingResult::Ok
        }
    }
    ///
    pub fn remove_at(&mut self, index: &u64) {
        if let Some(mut record) = self.lookup_map.get(index) {
            record.clear_extra_storage();
            self.lookup_map.remove_raw(&index.try_to_vec().unwrap());
        }
    }
}
