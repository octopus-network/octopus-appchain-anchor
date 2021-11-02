use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AnchorEventHistories {
    /// The anchor event data map.
    histories: LookupMap<u64, AnchorEventHistory>,
    /// The start index of valid anchor event.
    start_index: u64,
    /// The end index of valid anchor event.
    end_index: u64,
}

impl AnchorEventHistories {
    ///
    pub fn new() -> Self {
        Self {
            histories: LookupMap::new(StorageKey::AnchorEventHistoriesMap.into_bytes()),
            start_index: 0,
            end_index: 0,
        }
    }
    ///
    pub fn get(&self, index: &u64) -> Option<AnchorEventHistory> {
        self.histories.get(index)
    }
    ///
    pub fn index_range(&self) -> IndexRange {
        IndexRange {
            start_index: U64::from(self.start_index),
            end_index: U64::from(self.end_index),
        }
    }
    ///
    pub fn append(&mut self, anchor_event: AnchorEvent) -> AnchorEventHistory {
        let index = match self.histories.contains_key(&0) {
            true => self.end_index + 1,
            false => 0,
        };
        self.histories.insert(
            &index,
            &AnchorEventHistory {
                anchor_event,
                block_height: env::block_index(),
                timestamp: env::block_timestamp(),
                index: U64::from(index),
            },
        );
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
    pub fn reset(&mut self) {
        for index in self.start_index..self.end_index + 1 {
            self.histories.remove(&index);
        }
        self.start_index = 0;
        self.end_index = 0;
    }
}
