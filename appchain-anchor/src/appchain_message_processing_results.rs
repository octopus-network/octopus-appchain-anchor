use crate::*;

impl IndexedAndClearable for u32 {
    //
    fn set_index(&mut self, _index: &u64) {
        ()
    }
    //
    fn clear_extra_storage(&mut self) {
        ()
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainMessageProcessingResults {
    ///
    message_nonce_histories: IndexedHistories<u32>,
    ///
    processing_result_map: LookupMap<u32, AppchainMessageProcessingResult>,
}

impl AppchainMessageProcessingResults {
    ///
    pub fn new() -> Self {
        Self {
            message_nonce_histories: IndexedHistories::new(
                StorageKey::AppchainMessageProcessingResultsNonceSet,
            ),
            processing_result_map: LookupMap::new(
                StorageKey::AppchainMessageProcessingResultsMap.into_bytes(),
            ),
        }
    }
    ///
    pub fn insert_processing_result(
        &mut self,
        mut appchain_message_nonce: u32,
        processing_result: &AppchainMessageProcessingResult,
    ) {
        if !self
            .processing_result_map
            .contains_key(&appchain_message_nonce)
        {
            self.message_nonce_histories
                .append(&mut appchain_message_nonce);
        }
        self.processing_result_map
            .insert(&appchain_message_nonce, &processing_result);
    }
    ///
    pub fn get_processing_result(
        &self,
        appchain_message_nonce: u32,
    ) -> Option<AppchainMessageProcessingResult> {
        self.processing_result_map.get(&appchain_message_nonce)
    }
    ///
    pub fn get_processing_results(
        &self,
        start_index: &u64,
        quantity: Option<u64>,
    ) -> Vec<AppchainMessageProcessingResult> {
        let nonces = self
            .message_nonce_histories
            .get_histories(start_index, quantity);
        let mut results = Vec::<AppchainMessageProcessingResult>::new();
        nonces.iter().for_each(|nonce| {
            if let Some(processing_result) = self.processing_result_map.get(nonce) {
                results.push(processing_result);
            }
        });
        results
    }
    ///
    pub fn clear(&mut self) {
        let index_range = self.message_nonce_histories.index_range();
        for index in index_range.start_index.0..index_range.end_index.0 + 1 {
            if let Some(nonce) = self.message_nonce_histories.get(&index) {
                self.processing_result_map.remove(&nonce);
            }
        }
        self.message_nonce_histories.clear();
    }
}
