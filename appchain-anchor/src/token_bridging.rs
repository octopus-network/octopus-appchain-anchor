use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TokenBridgingHistories {
    /// The token bridging history data happened in this contract.
    pub histories: LookupMap<u64, TokenBridgingHistory>,
    /// The start index of valid token bridging history in `token_bridging_histories`.
    pub start_index: u64,
    /// The end index of valid token bridging history in `token_bridging_histories`.
    pub end_index: u64,
}

pub trait TokenBridgingHistoryManager {
    ///
    fn clear_token_bridging_history_before(&mut self, timestamp: Timestamp);
}
