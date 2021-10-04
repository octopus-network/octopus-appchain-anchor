use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenBridgingHistories {
    /// The token bridging history data happened in this contract.
    histories: LookupMap<u64, TokenBridgingHistory>,
    /// The start index of valid token bridging history.
    start_index: u64,
    /// The end index of valid token bridging history.
    end_index: u64,
}

impl TokenBridgingHistories {
    ///
    pub fn new() -> Self {
        Self {
            histories: LookupMap::new(StorageKey::TokenBridgingHistoriesMap.into_bytes()),
            start_index: 0,
            end_index: 0,
        }
    }
    ///
    pub fn get(&self, index: &u64) -> Option<TokenBridgingHistory> {
        self.histories.get(index)
    }
    ///
    pub fn index_range(&self) -> IndexRange {
        IndexRange {
            start_index: U64::from(self.start_index),
            end_index: U64::from(self.end_index),
        }
    }
}

pub trait TokenBridgingHistoryManager {
    ///
    fn clear_token_bridging_history_before(&mut self, timestamp: Timestamp);
}
