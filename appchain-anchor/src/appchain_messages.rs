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
pub struct AppchainMessages {
    ///
    message_nonces: LookupArray<u32>,
    ///
    message_map: LookupMap<u32, AppchainMessage>,
    ///
    processing_result_map: LookupMap<u32, AppchainMessageProcessingResult>,
    ///
    min_nonce: u32,
    ///
    max_nonce: u32,
}

impl AppchainMessages {
    ///
    pub fn new() -> Self {
        Self {
            message_nonces: LookupArray::new(StorageKey::AppchainMessageNonces),
            message_map: LookupMap::new(StorageKey::AppchainMessageMap.into_bytes()),
            processing_result_map: LookupMap::new(
                StorageKey::AppchainMessageProcessingResultMap.into_bytes(),
            ),
            min_nonce: 0,
            max_nonce: 0,
        }
    }
    ///
    pub fn min_nonce(&self) -> u32 {
        self.min_nonce
    }
    ///
    pub fn max_nonce(&self) -> u32 {
        self.max_nonce
    }
    ///
    pub fn insert_message(&mut self, appchain_message: &AppchainMessage) {
        let nonce = appchain_message.nonce;
        if !self.message_map.contains_key(&nonce) {
            self.message_map.insert(&nonce, appchain_message);
            if self.min_nonce == 0 && self.max_nonce == 0 {
                self.min_nonce = nonce;
                self.max_nonce = nonce;
            } else {
                if nonce < self.min_nonce {
                    self.min_nonce = nonce;
                }
                if nonce > self.max_nonce {
                    self.max_nonce = nonce;
                }
            }
        }
    }
    ///
    pub fn insert_processing_result(
        &mut self,
        appchain_message_nonce: u32,
        processing_result: &AppchainMessageProcessingResult,
    ) {
        if self.message_map.contains_key(&appchain_message_nonce) {
            self.processing_result_map
                .insert(&appchain_message_nonce, &processing_result);
        }
    }
    ///
    pub fn get_message(&self, appchain_message_nonce: u32) -> Option<AppchainMessage> {
        self.message_map.get(&appchain_message_nonce)
    }
    ///
    pub fn get_messages(&self, start_nonce: &u32, quantity: Option<u32>) -> Vec<AppchainMessage> {
        let mut results = Vec::<AppchainMessage>::new();
        let end_nonce = start_nonce
            + match quantity {
                Some(quantity) => quantity,
                None => 50,
            };
        for nonce in *start_nonce..end_nonce {
            if let Some(message) = self.message_map.get(&nonce) {
                results.push(message);
            }
        }
        results
    }
    ///
    pub fn get_processing_result(&self, nonce: &u32) -> Option<AppchainMessageProcessingResult> {
        self.processing_result_map.get(nonce)
    }
    ///
    pub fn get_processing_results(
        &self,
        start_nonce: &u32,
        quantity: Option<u32>,
    ) -> Vec<AppchainMessageProcessingResult> {
        let mut results = Vec::<AppchainMessageProcessingResult>::new();
        let end_nonce = start_nonce
            + match quantity {
                Some(quantity) => quantity,
                None => 50,
            };
        for nonce in *start_nonce..end_nonce {
            if let Some(processing_result) = self.processing_result_map.get(&nonce) {
                results.push(processing_result);
            }
        }
        results
    }
    ///
    pub fn clear(&mut self) -> MultiTxsOperationProcessingResult {
        log!(
            "Nonce range of appchain messsages: {} - {}",
            self.min_nonce,
            self.max_nonce
        );
        let mut nonce = self.min_nonce + 1;
        while nonce <= self.max_nonce + 1 && env::used_gas() < GAS_CAP_FOR_MULTI_TXS_PROCESSING {
            self.remove_messages_before(&nonce);
            nonce += 1;
        }
        if nonce <= self.max_nonce + 1 {
            self.min_nonce = nonce - 1;
            log!(
                "Nonce range of appchain messsages after clear: {} - {}",
                self.min_nonce,
                self.max_nonce
            );
            MultiTxsOperationProcessingResult::NeedMoreGas
        } else {
            self.min_nonce = 0;
            self.max_nonce = 0;
            MultiTxsOperationProcessingResult::Ok
        }
    }
    ///
    pub fn remove_messages_before(&mut self, nonce: &u32) {
        for nonce in self.min_nonce..*nonce {
            self.message_map.remove_raw(&nonce.try_to_vec().unwrap());
            self.processing_result_map
                .remove_raw(&nonce.try_to_vec().unwrap());
        }
        self.min_nonce = *nonce;
    }
}
