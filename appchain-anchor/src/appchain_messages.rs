use crate::*;
use codec::{Decode, Encode};

#[derive(Encode, Decode, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub enum PayloadType {
    Lock,
    BurnAsset,
    PlanNewEra,
    EraPayout,
    LockNft,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BurnAssetPayload {
    token_id: String,
    sender: String,
    receiver_id: AccountId,
    amount: u128,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct LockPayload {
    sender: String,
    receiver_id: AccountId,
    amount: u128,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PlanNewEraPayload {
    pub new_era: u32,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Offender {
    pub kind: String,
    pub who: String,
    pub offences: u32,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EraPayoutPayload {
    pub end_era: u32,
    pub excluded_validators: Vec<String>,
    pub offenders: Vec<Offender>,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct LockNftPayload {
    pub sender: String,
    pub receiver_id: AccountId,
    pub class: u128,
    pub instance: u128,
    pub metadata: TokenMetadata,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainMessage {
    pub appchain_event: AppchainEvent,
    // pub block_height: U64,
    // pub timestamp: U64,
    pub nonce: u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum MessagePayload {
    BurnAsset(BurnAssetPayload),
    Lock(LockPayload),
    PlanNewEra(PlanNewEraPayload),
    EraPayout(EraPayoutPayload),
    LockNft(LockNftPayload),
}

#[derive(Encode, Decode, Clone)]
pub struct RawMessage {
    nonce: u64,
    payload_type: PayloadType,
    payload: Vec<u8>,
}

impl RawMessage {
    pub fn nonce(&self) -> u32 {
        self.nonce as u32
    }
}

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
        if !self
            .processing_result_map
            .contains_key(&appchain_message_nonce)
        {
            self.processing_result_map
                .insert(&appchain_message_nonce, &processing_result);
        }
    }
    ///
    pub fn get_message(&self, appchain_message_nonce: &u32) -> Option<AppchainMessage> {
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
        while nonce <= self.max_nonce + 1
            && env::used_gas() < Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING)
        {
            self.remove_messages_before(&nonce);
            nonce += 1;
        }
        if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
            self.min_nonce = nonce - 1;
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

impl AppchainAnchor {
    ///
    pub fn internal_stage_appchain_messages(&mut self, messages: &Vec<RawMessage>) {
        let mut processing_status = self.permissionless_actions_status.get().unwrap();
        let mut appchain_messages = self.appchain_messages.get().unwrap();
        messages
            .iter()
            .filter(|message| {
                message.nonce as u32 > processing_status.latest_applied_appchain_message_nonce
            })
            .for_each(|raw_message| {
                self.internal_stage_raw_message(&mut appchain_messages, raw_message)
            });
        self.appchain_messages.set(&appchain_messages);
        processing_status.max_nonce_of_staged_appchain_messages = appchain_messages.max_nonce();
        self.permissionless_actions_status.set(&processing_status);
    }
    //
    fn internal_stage_raw_message(
        &mut self,
        appchain_messages: &mut AppchainMessages,
        raw_message: &RawMessage,
    ) {
        match raw_message.payload_type {
            PayloadType::BurnAsset => {
                let payload_result: Result<BurnAssetPayload, std::io::Error> =
                    BorshDeserialize::deserialize(&mut &raw_message.payload[..]);
                match payload_result {
                    Ok(payload) => {
                        log!(
                            "Origin appchain message: '{}'",
                            serde_json::to_string(&payload).unwrap()
                        );
                        appchain_messages.insert_message(&AppchainMessage {
                            nonce: raw_message.nonce as u32,
                            appchain_event: AppchainEvent::NearFungibleTokenBurnt {
                                contract_account: payload.token_id,
                                owner_id_in_appchain: payload.sender,
                                receiver_id_in_near: payload.receiver_id,
                                amount: payload.amount.into(),
                            },
                        });
                    }
                    Err(err) => appchain_messages.insert_processing_result(
                        raw_message.nonce as u32,
                        &AppchainMessageProcessingResult::Error {
                            nonce: raw_message.nonce as u32,
                            message: format!("Failed to deserialize raw message paylod: {}", err),
                        },
                    ),
                }
            }
            PayloadType::Lock => {
                let payload_result: Result<LockPayload, std::io::Error> =
                    BorshDeserialize::deserialize(&mut &raw_message.payload[..]);
                match payload_result {
                    Ok(payload) => {
                        log!(
                            "Origin appchain message: '{}'",
                            serde_json::to_string(&payload).unwrap()
                        );
                        appchain_messages.insert_message(&AppchainMessage {
                            nonce: raw_message.nonce as u32,
                            appchain_event: AppchainEvent::NativeTokenLocked {
                                owner_id_in_appchain: payload.sender,
                                receiver_id_in_near: payload.receiver_id,
                                amount: payload.amount.into(),
                            },
                        });
                    }
                    Err(err) => appchain_messages.insert_processing_result(
                        raw_message.nonce as u32,
                        &AppchainMessageProcessingResult::Error {
                            nonce: raw_message.nonce as u32,
                            message: format!("Failed to deserialize raw message payload: {}", err),
                        },
                    ),
                }
            }
            PayloadType::PlanNewEra => {
                let payload_result: Result<PlanNewEraPayload, std::io::Error> =
                    BorshDeserialize::deserialize(&mut &raw_message.payload[..]);
                match payload_result {
                    Ok(payload) => {
                        log!(
                            "Origin appchain message: '{}'",
                            serde_json::to_string(&payload).unwrap()
                        );
                        appchain_messages.insert_message(&AppchainMessage {
                            nonce: raw_message.nonce as u32,
                            appchain_event: AppchainEvent::EraSwitchPlaned {
                                era_number: payload.new_era,
                            },
                        });
                    }
                    Err(err) => appchain_messages.insert_processing_result(
                        raw_message.nonce as u32,
                        &AppchainMessageProcessingResult::Error {
                            nonce: raw_message.nonce as u32,
                            message: format!("Failed to deserialize raw message payload: {}", err),
                        },
                    ),
                }
            }
            PayloadType::EraPayout => {
                let payload_result: Result<EraPayoutPayload, std::io::Error> =
                    BorshDeserialize::deserialize(&mut &raw_message.payload[..]);
                match payload_result {
                    Ok(payload) => {
                        log!(
                            "Origin appchain message: '{}'",
                            serde_json::to_string(&payload).unwrap()
                        );
                        let protocol_settings = self.protocol_settings.get().unwrap();
                        if self.era_number_is_too_old(
                            u64::from(payload.end_era),
                            protocol_settings
                                .maximum_era_count_of_valid_appchain_message
                                .0,
                        ) {
                            appchain_messages.insert_processing_result(
                                raw_message.nonce as u32,
                                &AppchainMessageProcessingResult::Error {
                                    nonce: raw_message.nonce as u32,
                                    message: format!(
                                        "Era number of message 'EraPayout' is too old."
                                    ),
                                },
                            );
                            return;
                        }
                        appchain_messages.insert_message(&AppchainMessage {
                            nonce: raw_message.nonce as u32,
                            appchain_event: AppchainEvent::EraRewardConcluded {
                                era_number: payload.end_era,
                                unprofitable_validator_ids: payload.excluded_validators,
                                offenders: payload.offenders,
                            },
                        });
                    }
                    Err(err) => appchain_messages.insert_processing_result(
                        raw_message.nonce as u32,
                        &AppchainMessageProcessingResult::Error {
                            nonce: raw_message.nonce as u32,
                            message: format!("Failed to deserialize raw message payload: {}", err),
                        },
                    ),
                }
            }
            PayloadType::LockNft => {
                let payload_result: Result<LockNftPayload, std::io::Error> =
                    BorshDeserialize::deserialize(&mut &raw_message.payload[..]);
                match payload_result {
                    Ok(payload) => {
                        log!(
                            "Origin appchain message: '{}'",
                            serde_json::to_string(&payload).unwrap()
                        );
                        appchain_messages.insert_message(&AppchainMessage {
                            nonce: raw_message.nonce as u32,
                            appchain_event: AppchainEvent::NonFungibleTokenLocked {
                                owner_id_in_appchain: payload.sender,
                                receiver_id_in_near: payload.receiver_id,
                                class_id: payload.class.to_string(),
                                instance_id: payload.instance.to_string(),
                                token_metadata: payload.metadata,
                            },
                        });
                    }
                    Err(err) => appchain_messages.insert_processing_result(
                        raw_message.nonce as u32,
                        &AppchainMessageProcessingResult::Error {
                            nonce: raw_message.nonce as u32,
                            message: format!("Failed to deserialize raw message payload: {}", err),
                        },
                    ),
                }
            }
        }
    }
    //
    fn era_number_is_too_old(&self, era_number: u64, range: u64) -> bool {
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        let index_range = validator_set_histories.index_range();
        if index_range.end_index.0 > range {
            era_number <= index_range.end_index.0 - range
        } else {
            era_number < index_range.start_index.0
        }
    }
}
