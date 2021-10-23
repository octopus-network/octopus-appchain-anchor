use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AnchorEvents {
    /// The anchor event data map.
    events: LookupMap<u64, AnchorEvent>,
    /// The start index of valid anchor event.
    start_index: u64,
    /// The end index of valid anchor event.
    end_index: u64,
}

impl AnchorEvents {
    ///
    pub fn new() -> Self {
        Self {
            events: LookupMap::new(StorageKey::AnchorEventsMap.into_bytes()),
            start_index: 0,
            end_index: 0,
        }
    }
    ///
    pub fn get(&self, index: &u64) -> Option<AnchorEvent> {
        self.events.get(index)
    }
    ///
    pub fn index_range(&self) -> IndexRange {
        IndexRange {
            start_index: U64::from(self.start_index),
            end_index: U64::from(self.end_index),
        }
    }
    ///
    pub fn append(&mut self, anchor_event: AnchorEvent) {
        let index = match self.events.contains_key(&0) {
            true => self.end_index + 1,
            false => 0,
        };
        self.events.insert(&index, &anchor_event);
        self.end_index = index;
    }
}
