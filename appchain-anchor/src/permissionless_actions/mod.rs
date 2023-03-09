mod distributing_rewards;
mod switching_era;

use crate::appchain_messages::Offender;
use crate::assets::native_near_token::CONTRACT_ACCOUNT_FOR_NATIVE_NEAR_TOKEN;
use crate::interfaces::PermissionlessActions;
use crate::*;
use core::convert::TryFrom;
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_sdk::env::sha256;
use parity_scale_codec::Decode;
use std::ops::Add;
use std::str::FromStr;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainEvent {
    /// The fact that a certain amount of bridged fungible token has been burnt in the appchain.
    NearFungibleTokenBurnt {
        contract_account: String,
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        amount: U128,
        fee: U128,
    },
    /// The fact that a certain amount of appchain native token has been locked in the appchain.
    NativeTokenLocked {
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        amount: U128,
        fee: U128,
    },
    /// The fact that the era switch is planed in the appchain.
    EraSwitchPlaned { era_number: u32 },
    /// The fact that the total reward and unprofitable validator list
    /// is concluded in the appchain.
    EraRewardConcluded {
        era_number: u32,
        unprofitable_validator_ids: Vec<String>,
        offenders: Vec<Offender>,
    },
    /// The fact that a certain non-fungible token is locked in the appchain.
    NonFungibleTokenLocked {
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        class_id: String,
        instance_id: String,
        token_metadata: TokenMetadata,
        fee: U128,
    },
}

pub struct AppchainMessagesProcessingContext {
    processing_status: PermissionlessActionsStatus,
    prepaid_gas_for_extra_actions: Gas,
}

impl AppchainMessagesProcessingContext {
    ///
    pub fn new(status: PermissionlessActionsStatus) -> Self {
        Self {
            processing_status: status,
            prepaid_gas_for_extra_actions: Gas::from(0),
        }
    }
    ///
    pub fn add_prepaid_gas(&mut self, gas: Gas) {
        self.prepaid_gas_for_extra_actions = self.prepaid_gas_for_extra_actions.add(gas);
    }
    ///
    pub fn set_processing_nonce(&mut self, nonce: u32) {
        self.processing_status.processing_appchain_message_nonce = Some(nonce);
    }
    ///
    pub fn clear_processing_nonce(&mut self) {
        self.processing_status.processing_appchain_message_nonce = None;
    }
    ///
    pub fn set_latest_applied_nonce(&mut self, nonce: u32) {
        self.processing_status.latest_applied_appchain_message_nonce = nonce;
    }
    ///
    pub fn set_switching_era_number(&mut self, era_number: u64) {
        self.processing_status.switching_era_number = Some(U64::from(era_number));
    }
    ///
    pub fn clear_switching_era_number(&mut self) {
        self.processing_status.switching_era_number = None;
    }
    ///
    pub fn set_distributing_reward_era_number(&mut self, era_number: u64) {
        self.processing_status.distributing_reward_era_number = Some(U64::from(era_number));
    }
    ///
    pub fn clear_distributing_reward_era_number(&mut self) {
        self.processing_status.distributing_reward_era_number = None;
    }
    ///
    pub fn processing_status(&self) -> &PermissionlessActionsStatus {
        &self.processing_status
    }
    ///
    pub fn processing_nonce(&self) -> Option<u32> {
        self.processing_status.processing_appchain_message_nonce
    }
    ///
    pub fn max_nonce(&self) -> u32 {
        self.processing_status.max_nonce_of_staged_appchain_messages
    }
    ///
    pub fn latest_applied_nonce(&self) -> u32 {
        self.processing_status.latest_applied_appchain_message_nonce
    }
    ///
    pub fn switching_era_number(&self) -> Option<u64> {
        self.processing_status.switching_era_number.map(|n| n.0)
    }
    ///
    pub fn distributing_reward_era_number(&self) -> Option<u64> {
        self.processing_status
            .distributing_reward_era_number
            .map(|n| n.0)
    }
    ///
    pub fn used_gas_of_current_function_call(&self) -> Gas {
        env::used_gas() - self.prepaid_gas_for_extra_actions
    }
}

enum ResultOfLoopingValidatorSet {
    NoMoreDelegator,
    NoMoreValidator,
    NeedToContinue,
}

#[near_bindgen]
impl PermissionlessActions for AppchainAnchor {
    //
    fn stage_and_apply_appchain_messages(
        &mut self,
        encoded_messages: Vec<u8>,
        verification_proxy_signature: Option<Vec<u8>>,
    ) {
        self.assert_relayer();
        let anchor_settings = self.anchor_settings.get().unwrap();
        if !anchor_settings.witness_mode {
            // Check the signature
            let signature =
                verification_proxy_signature.expect("Missing signature of verification proxy.");
            let signature = secp256k1::ecdsa::Signature::from_compact(&signature)
                .expect("Invalid ecdsa signature.");
            let anchor_settings = self.anchor_settings.get().unwrap();
            let secp = secp256k1::Secp256k1::new();
            let message = sha256(&encoded_messages);
            assert!(secp
                .verify_ecdsa(
                    &secp256k1::Message::from_slice(&message).unwrap(),
                    &signature,
                    &secp256k1::PublicKey::from_slice(
                        &anchor_settings.verification_proxy_pubkey.unwrap()
                    )
                    .unwrap(),
                )
                .is_ok());
        }
        match Decode::decode(&mut &encoded_messages[..]) {
            Ok(messages) => {
                self.internal_stage_appchain_messages(&messages);
                self.internal_process_appchain_message(Some(messages[0].nonce()));
            }
            Err(err) => panic!("Failed to decode messages: {}", err),
        }
    }
    //
    fn process_appchain_messages(&mut self) -> MultiTxsOperationProcessingResult {
        self.internal_process_appchain_message(None)
    }
    //
    fn commit_appchain_challenge(
        &mut self,
        appchain_challenge: AppchainChallenge,
        verification_proxy_signature: Vec<u8>,
    ) {
        // Check the signature
        let signature = secp256k1::ecdsa::Signature::from_der(&verification_proxy_signature)
            .expect("Invalid ecdsa signature.");
        let anchor_settings = self.anchor_settings.get().unwrap();
        let secp = secp256k1::Secp256k1::new();
        let message = sha256(&appchain_challenge.try_to_vec().unwrap());
        assert!(secp
            .verify_ecdsa(
                &secp256k1::Message::from_slice(&message).unwrap(),
                &signature,
                &secp256k1::PublicKey::from_slice(
                    &anchor_settings.verification_proxy_pubkey.unwrap()
                )
                .unwrap(),
            )
            .is_ok());
        // Store challenge data
        let mut appchain_challenges = self.appchain_challenges.get().unwrap();
        appchain_challenges.append(&mut appchain_challenge.clone());
        self.appchain_challenges.set(&appchain_challenges);
    }
}

impl AppchainAnchor {
    /// Process a certain `AppchainMesaage`
    pub fn internal_process_appchain_message(
        &mut self,
        nonce: Option<u32>,
    ) -> MultiTxsOperationProcessingResult {
        let processing_status = self.permissionless_actions_status.get().unwrap();
        let appchain_messages = self.appchain_messages.get().unwrap();
        let mut processing_context = AppchainMessagesProcessingContext::new(processing_status);
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        let mut result = MultiTxsOperationProcessingResult::Ok;
        if processing_context.processing_nonce().is_none() {
            if let Some(nonce) = nonce {
                processing_context.set_processing_nonce(nonce);
            } else if processing_context.latest_applied_nonce() < processing_context.max_nonce() {
                processing_context
                    .set_processing_nonce(processing_context.latest_applied_nonce() + 1);
            }
        }
        if let Some(processing_nonce) = processing_context.processing_nonce() {
            if appchain_messages
                .get_processing_result(&processing_nonce)
                .is_none()
            {
                if let Some(appchain_message) = appchain_messages.get_message(&processing_nonce) {
                    let start_gas = env::used_gas();
                    result = self.internal_apply_appchain_message(
                        &mut processing_context,
                        &mut validator_set_histories,
                        &appchain_message,
                    );
                    log!(
                        "Gas used for appchain message '{}': {}",
                        appchain_message.nonce,
                        (env::used_gas() - start_gas).0
                    );
                    match result {
                        MultiTxsOperationProcessingResult::Ok => {
                            processing_context.clear_processing_nonce();
                            processing_context.set_latest_applied_nonce(processing_nonce);
                        }
                        MultiTxsOperationProcessingResult::NeedMoreGas => (),
                        MultiTxsOperationProcessingResult::Error(..) => {
                            // The loop should continue even if it fails to apply a certain message
                            processing_context.clear_processing_nonce();
                            processing_context.set_latest_applied_nonce(processing_nonce);
                            result = MultiTxsOperationProcessingResult::Ok;
                        }
                    }
                } else {
                    result = MultiTxsOperationProcessingResult::Error(format!(
                        "Missing appchain message with nonce '{}'.",
                        processing_nonce
                    ));
                }
            } else {
                processing_context.clear_processing_nonce();
            }
        }
        self.permissionless_actions_status
            .set(processing_context.processing_status());
        self.validator_set_histories.set(&validator_set_histories);
        if result.is_ok()
            && processing_context.latest_applied_nonce() < processing_context.max_nonce()
        {
            result = MultiTxsOperationProcessingResult::NeedMoreGas;
        }
        result
    }
    /// Apply a certain `AppchainMessage`
    pub fn internal_apply_appchain_message(
        &mut self,
        processing_context: &mut AppchainMessagesProcessingContext,
        validator_set_histories: &mut LookupArray<ValidatorSetOfEra>,
        appchain_message: &AppchainMessage,
    ) -> MultiTxsOperationProcessingResult {
        match &appchain_message.appchain_event {
            AppchainEvent::NearFungibleTokenBurnt {
                contract_account,
                owner_id_in_appchain,
                receiver_id_in_near,
                amount,
                fee,
            } => {
                if self.asset_transfer_is_paused {
                    let message = format!("Asset transfer is now paused.");
                    let result = AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message: message.clone(),
                    };
                    self.record_appchain_message_processing_result(&result);
                    return MultiTxsOperationProcessingResult::Error(message);
                }
                //
                if contract_account
                    .eq(&String::from_str(CONTRACT_ACCOUNT_FOR_NATIVE_NEAR_TOKEN).unwrap())
                {
                    let mut native_near_token = self.native_near_token.get().unwrap();
                    let result = native_near_token.unlock_near(
                        receiver_id_in_near,
                        amount,
                        processing_context,
                    );
                    self.native_near_token.set(&native_near_token);
                    self.record_appchain_message_processing_result(
                        &AppchainMessageProcessingResult::Ok {
                            nonce: appchain_message.nonce,
                            message: None,
                        },
                    );
                    return result;
                }
                //
                let contract_account_id = AccountId::from_str(&contract_account);
                if contract_account_id.is_err() {
                    let message = format!("Invalid contract account: '{}'.", contract_account);
                    let result = AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message: message.clone(),
                    };
                    self.record_appchain_message_processing_result(&result);
                    return MultiTxsOperationProcessingResult::Error(message);
                }
                let mut result = self.internal_unlock_near_fungible_token(
                    owner_id_in_appchain,
                    &contract_account_id.unwrap(),
                    receiver_id_in_near,
                    amount,
                    appchain_message.nonce,
                    processing_context,
                );
                if result.is_ok() {
                    let anchor_settings = self.anchor_settings.get().unwrap();
                    result = self.internal_mint_wrapped_appchain_token(
                        Some(owner_id_in_appchain),
                        &anchor_settings.relayer_account.unwrap(),
                        fee,
                        appchain_message.nonce,
                        processing_context,
                    );
                }
                result
            }
            AppchainEvent::NativeTokenLocked {
                owner_id_in_appchain,
                receiver_id_in_near,
                amount,
                fee,
            } => {
                if self.asset_transfer_is_paused {
                    let message = format!("Asset transfer is now paused.");
                    let result = AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message: message.clone(),
                    };
                    self.record_appchain_message_processing_result(&result);
                    return MultiTxsOperationProcessingResult::Error(message);
                }
                let wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
                if i128::try_from(wrapped_appchain_token.premined_balance.0).unwrap()
                    + wrapped_appchain_token.changed_balance.0
                    + i128::try_from(amount.0).unwrap()
                    > i128::try_from(wrapped_appchain_token.total_supply.0).unwrap()
                {
                    let message = format!("Too much wrapped appchain token to mint.");
                    let result = AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message: message.clone(),
                    };
                    self.record_appchain_message_processing_result(&result);
                    return MultiTxsOperationProcessingResult::Error(message);
                }
                let mut result = self.internal_mint_wrapped_appchain_token(
                    Some(owner_id_in_appchain),
                    receiver_id_in_near,
                    amount,
                    appchain_message.nonce,
                    processing_context,
                );
                if result.is_ok() {
                    let anchor_settings = self.anchor_settings.get().unwrap();
                    result = self.internal_mint_wrapped_appchain_token(
                        Some(owner_id_in_appchain),
                        &anchor_settings.relayer_account.unwrap(),
                        fee,
                        appchain_message.nonce,
                        processing_context,
                    );
                }
                result
            }
            AppchainEvent::EraSwitchPlaned { era_number } => {
                if let Some(era_number) = processing_context.switching_era_number() {
                    self.complete_switching_era(
                        processing_context,
                        validator_set_histories,
                        era_number,
                    )
                } else {
                    let index_range = validator_set_histories.index_range();
                    if u64::from(*era_number) <= index_range.end_index.0 {
                        let message = format!("Switching era number '{}' is too old.", era_number);
                        let result = AppchainMessageProcessingResult::Error {
                            nonce: appchain_message.nonce,
                            message: message.clone(),
                        };
                        self.record_appchain_message_processing_result(&result);
                        return MultiTxsOperationProcessingResult::Error(message);
                    }
                    self.internal_start_switching_era(
                        processing_context,
                        validator_set_histories,
                        u64::from(*era_number),
                    )
                }
            }
            AppchainEvent::EraRewardConcluded {
                era_number,
                unprofitable_validator_ids,
                offenders: _,
            } => {
                if let Some(era_number) = processing_context.distributing_reward_era_number() {
                    self.complete_distributing_reward_of_era(
                        processing_context,
                        validator_set_histories,
                        era_number,
                    )
                } else {
                    self.internal_start_distributing_reward_of_era(
                        processing_context,
                        validator_set_histories,
                        appchain_message.nonce,
                        u64::from(*era_number),
                        unprofitable_validator_ids,
                    )
                }
            }
            AppchainEvent::NonFungibleTokenLocked {
                owner_id_in_appchain,
                receiver_id_in_near,
                class_id,
                instance_id,
                token_metadata,
                fee,
            } => {
                if self.asset_transfer_is_paused {
                    let message = format!("Asset transfer is now paused.");
                    let result = AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message: message.clone(),
                    };
                    self.record_appchain_message_processing_result(&result);
                    return MultiTxsOperationProcessingResult::Error(message);
                }
                let mut result = self.internal_process_locked_nft_in_appchain(
                    processing_context,
                    appchain_message.nonce,
                    owner_id_in_appchain,
                    receiver_id_in_near,
                    class_id,
                    instance_id,
                    token_metadata,
                );
                if result.is_ok() {
                    let anchor_settings = self.anchor_settings.get().unwrap();
                    result = self.internal_mint_wrapped_appchain_token(
                        Some(owner_id_in_appchain),
                        &anchor_settings.relayer_account.unwrap(),
                        fee,
                        appchain_message.nonce,
                        processing_context,
                    );
                }
                result
            }
        }
    }
    ///
    pub fn record_appchain_message_processing_result(
        &mut self,
        processing_result: &AppchainMessageProcessingResult,
    ) {
        let mut appchain_messages = self.appchain_messages.get().unwrap();
        appchain_messages.insert_processing_result(processing_result.nonce(), processing_result);
        self.appchain_messages.set(&appchain_messages);
        log!(
            "Processing result of appchain message '{}': '{}'",
            serde_json::to_string::<AppchainMessage>(
                &appchain_messages
                    .get_message(&processing_result.nonce())
                    .unwrap_or_else(|| {
                        if processing_result.nonce() > 0 {
                            panic!(
                                "Missing staged message with nonce '{}'.",
                                processing_result.nonce()
                            )
                        } else {
                            AppchainMessage {
                                appchain_event: AppchainEvent::EraSwitchPlaned { era_number: 0 },
                                nonce: 0,
                            }
                        }
                    })
            )
            .unwrap(),
            serde_json::to_string::<AppchainMessageProcessingResult>(&processing_result).unwrap(),
        );
    }
}

impl AppchainMessageProcessingResult {
    pub fn nonce(&self) -> u32 {
        match self {
            AppchainMessageProcessingResult::Ok { nonce, .. }
            | AppchainMessageProcessingResult::Error { nonce, .. } => *nonce,
        }
    }
}

mod test {
    #[test]
    fn test_verifying_ecdsa_signature() {
        let encoded_messages = [
            4, 1, 0, 0, 0, 0, 0, 0, 0, 0, 161, 1, 66, 0, 0, 0, 48, 120, 100, 52, 51, 53, 57, 51,
            99, 55, 49, 53, 102, 100, 100, 51, 49, 99, 54, 49, 49, 52, 49, 97, 98, 100, 48, 52, 97,
            57, 57, 102, 100, 54, 56, 50, 50, 99, 56, 53, 53, 56, 56, 53, 52, 99, 99, 100, 101, 51,
            57, 97, 53, 54, 56, 52, 101, 55, 97, 53, 54, 100, 97, 50, 55, 100, 14, 0, 0, 0, 106,
            117, 108, 105, 97, 110, 115, 117, 110, 46, 110, 101, 97, 114, 0, 0, 100, 167, 179, 182,
            224, 13, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let signature = [
            48, 69, 2, 33, 0, 148, 148, 138, 36, 248, 214, 201, 135, 167, 254, 115, 159, 101, 127,
            104, 27, 245, 106, 222, 139, 252, 188, 172, 252, 155, 154, 228, 148, 129, 136, 7, 181,
            2, 32, 88, 249, 205, 111, 220, 171, 85, 15, 4, 35, 132, 199, 255, 249, 227, 152, 253,
            250, 152, 144, 225, 143, 35, 254, 226, 173, 173, 2, 58, 143, 188, 161,
        ];
        let signature =
            secp256k1::ecdsa::Signature::from_der(&signature).expect("Invalid ecdsa signature.");
        let pubkey_hex = "03e7eb789e168e736aadb317acc0cf7eeae531c0ddb95a25d4e4638550d2a9f49a";
        let pubkey_bytes = hex::decode(pubkey_hex).unwrap();
        let secp = secp256k1::Secp256k1::new();
        let message = near_sdk::env::sha256(&encoded_messages);
        assert!(secp
            .verify_ecdsa(
                &secp256k1::Message::from_slice(&message).unwrap(),
                &signature,
                &secp256k1::PublicKey::from_slice(&pubkey_bytes).unwrap(),
            )
            .is_ok());
    }
}
