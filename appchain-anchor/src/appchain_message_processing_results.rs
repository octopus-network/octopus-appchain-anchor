use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainMessageProcessingResults {
    ///
    message_nonce_histories: LookupArray<u32>,
    ///
    processing_result_map: LookupMap<u32, AppchainMessageProcessingResult>,
}

impl AppchainMessageProcessingResults {
    ///
    pub fn new() -> Self {
        Self {
            message_nonce_histories: LookupArray::new(
                StorageKey::AppchainMessageProcessingResultsNonceSet,
            ),
            processing_result_map: LookupMap::new(
                StorageKey::AppchainMessageProcessingResultsMap.into_bytes(),
            ),
        }
    }
    ///
    pub fn index_range(&self) -> IndexRange {
        self.message_nonce_histories.index_range()
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
            .get_slice_of(start_index, quantity);
        let mut results = Vec::<AppchainMessageProcessingResult>::new();
        nonces.iter().for_each(|nonce| {
            if let Some(processing_result) = self.processing_result_map.get(nonce) {
                results.push(processing_result);
            }
        });
        results
    }
    ///
    pub fn clear(&mut self) -> MultiTxsOperationProcessingResult {
        let index_range = self.message_nonce_histories.index_range();
        log!(
            "Index range of appchain message processing results: {} - {}",
            index_range.start_index.0,
            index_range.end_index.0
        );
        let mut index = index_range.start_index.0;
        while index < index_range.end_index.0 && env::used_gas() < GAS_CAP_FOR_MULTI_TXS_PROCESSING
        {
            if let Some(nonce) = self.message_nonce_histories.get(&index) {
                self.processing_result_map.remove(&nonce);
            }
            index += 1;
            self.message_nonce_histories.remove_before(&index);
        }
        if env::used_gas() > GAS_CAP_FOR_MULTI_TXS_PROCESSING {
            MultiTxsOperationProcessingResult::NeedMoreGas
        } else {
            self.message_nonce_histories.clear()
        }
    }
}
