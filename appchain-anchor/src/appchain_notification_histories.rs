use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainNotificationHistories {
    /// The anchor event data map.
    histories: LookupMap<u64, AppchainNotificationHistory>,
    /// The start index of valid anchor event.
    start_index: u64,
    /// The end index of valid anchor event.
    end_index: u64,
}

impl AppchainNotificationHistories {
    ///
    pub fn new() -> Self {
        Self {
            histories: LookupMap::new(StorageKey::AppchainNotificationHistoriesMap.into_bytes()),
            start_index: 0,
            end_index: 0,
        }
    }
    ///
    pub fn get(&self, index: &u64) -> Option<AppchainNotificationHistory> {
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
    pub fn append(
        &mut self,
        appchain_notification: AppchainNotification,
    ) -> AppchainNotificationHistory {
        let index = match self.histories.contains_key(&0) {
            true => self.end_index + 1,
            false => 0,
        };
        self.histories.insert(
            &index,
            &AppchainNotificationHistory {
                appchain_notification,
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
    pub fn reset_to(&mut self, index: &u64) {
        assert!(
            *index >= self.start_index && *index <= self.end_index,
            "Invalid history data index."
        );
        for index in (*index + 1)..self.end_index + 1 {
            self.histories.remove(&index);
        }
        self.end_index = *index;
    }
    ///
    pub fn clear(&mut self) {
        for index in self.start_index..self.end_index + 1 {
            self.histories.remove(&index);
        }
        self.start_index = 0;
        self.end_index = 0;
    }
}
