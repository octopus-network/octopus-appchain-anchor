mod distributing_rewards;
mod switching_era;

use crate::*;
use crate::{interfaces::PermissionlessActions, message_decoder::AppchainMessage};
use core::convert::TryFrom;
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use std::ops::Add;
use std::str::FromStr;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainEvent {
    /// The fact that a certain amount of bridge token has been burnt in the appchain.
    NearFungibleTokenBurnt {
        contract_account: String,
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        amount: U128,
    },
    /// The fact that a certain amount of appchain native token has been locked in the appchain.
    NativeTokenLocked {
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        amount: U128,
    },
    /// The fact that the era switch is planed in the appchain.
    EraSwitchPlaned { era_number: u32 },
    /// The fact that the total reward and unprofitable validator list
    /// is concluded in the appchain.
    EraRewardConcluded {
        era_number: u32,
        unprofitable_validator_ids: Vec<String>,
    },
    /// The fact that a certain non-fungible token is locked in the appchain.
    NonFungibleTokenLocked {
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        class_id: String,
        instance_id: String,
        token_metadata: TokenMetadata,
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
    pub fn pop_nonce_in_queue(&mut self) -> Option<u32> {
        self.processing_status.nonces_in_queue.pop_front()
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

impl PermissionlessActionsStatus {
    //
    pub fn add_nonce_to_queue(&mut self, nonce: u32) {
        self.nonces_in_queue.push_back(nonce);
    }
    //
    pub fn sort_nonces_in_queue(&mut self) {
        if self.nonces_in_queue.len() > 0 {
            let mut nonces: Vec<u32> = self.nonces_in_queue.iter().map(|f| *f).collect();
            nonces.sort_by(|a, b| a.cmp(b));
            self.nonces_in_queue.clear();
            nonces
                .iter()
                .for_each(|f| self.nonces_in_queue.push_back(*f));
        }
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
    fn process_appchain_messages(&mut self) -> MultiTxsOperationProcessingResult {
        let processing_status = self.permissionless_actions_status.get().unwrap();
        let mut appchain_messages = self.appchain_messages.get().unwrap();
        let mut processing_context = AppchainMessagesProcessingContext::new(processing_status);
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        let mut result = MultiTxsOperationProcessingResult::Ok;
        while processing_context.used_gas_of_current_function_call()
            < Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING)
            && env::used_gas() < Gas::ONE_TERA.mul(T_GAS_CAP_FOR_PROCESSING_APPCHAIN_MESSAGES)
        {
            if let Some(processing_nonce) = processing_context.processing_nonce() {
                if let Some(appchain_message) = appchain_messages.get_message(processing_nonce) {
                    if appchain_messages
                        .get_processing_result(&processing_nonce)
                        .is_some()
                    {
                        processing_context.clear_processing_nonce();
                        processing_context.set_latest_applied_nonce(processing_nonce);
                        result = MultiTxsOperationProcessingResult::Ok;
                        continue;
                    }
                    result = self.internal_apply_appchain_message(
                        &mut processing_context,
                        &mut validator_set_histories,
                        &mut appchain_messages,
                        &appchain_message,
                    );
                    match result {
                        MultiTxsOperationProcessingResult::Ok => {
                            processing_context.clear_processing_nonce();
                        }
                        MultiTxsOperationProcessingResult::NeedMoreGas => (),
                        MultiTxsOperationProcessingResult::Error(..) => {
                            // The loop should continue even if it fails to apply a certain message
                            processing_context.clear_processing_nonce();
                            result = MultiTxsOperationProcessingResult::Ok;
                        }
                    }
                } else {
                    result = MultiTxsOperationProcessingResult::Error(format!(
                        "Missing appchain message with nonce '{}'.",
                        processing_nonce
                    ));
                    break;
                }
            } else {
                if processing_context.processing_status().nonces_in_queue.len() > 0 {
                    let nonce = processing_context.pop_nonce_in_queue().unwrap();
                    processing_context.set_processing_nonce(nonce);
                } else {
                    break;
                }
            }
        }
        self.permissionless_actions_status
            .set(processing_context.processing_status());
        self.validator_set_histories.set(&validator_set_histories);
        if result.eq(&MultiTxsOperationProcessingResult::Ok)
            && processing_context.processing_status().nonces_in_queue.len() > 0
        {
            result = MultiTxsOperationProcessingResult::NeedMoreGas;
        }
        result
    }
    //
    fn commit_appchain_challenge(&mut self, appchain_challenge: AppchainChallenge) {
        match &appchain_challenge {
            AppchainChallenge::EquivocationChallenge {
                submitter_account: _,
                proof,
            } => {
                assert!(proof.is_valid(), "Invalid equivocation challenge data.");
            }
            AppchainChallenge::ConspiracyMmr { .. } => (),
        }
        let mut appchain_challenges = self.appchain_challenges.get().unwrap();
        appchain_challenges.append(&mut appchain_challenge.clone());
        self.appchain_challenges.set(&appchain_challenges);
    }
    //
    fn process_appchain_messages_with_all_proofs(
        &mut self,
        signed_commitment: Vec<u8>,
        validator_proofs: Vec<ValidatorMerkleProof>,
        mmr_leaf_for_mmr_root: Vec<u8>,
        mmr_proof_for_mmr_root: Vec<u8>,
        encoded_messages: Vec<u8>,
        header: Vec<u8>,
        mmr_leaf_for_header: Vec<u8>,
        mmr_proof_for_header: Vec<u8>,
    ) -> MultiTxsOperationProcessingResult {
        let anchor_settings = self.anchor_settings.get().unwrap();
        if anchor_settings.beefy_light_client_witness_mode {
            self.assert_relayer();
        } else {
            self.assert_light_client_initialized();
            let mut light_client_state = self.beefy_light_client_state.get().unwrap();
            match beefy_light_client::verifier_for_external_state_data::verify_signed_commitment(
                &signed_commitment,
                &validator_proofs
                    .iter()
                    .map(|proof| beefy_light_client::ValidatorMerkleProof {
                        proof: proof.proof.clone(),
                        number_of_leaves: proof.number_of_leaves.try_into().unwrap_or_default(),
                        leaf_index: proof.leaf_index.try_into().unwrap_or_default(),
                        leaf: proof.leaf.clone(),
                    })
                    .collect::<Vec<beefy_light_client::ValidatorMerkleProof>>(),
                &mmr_leaf_for_mmr_root,
                &mmr_proof_for_mmr_root,
                &mut light_client_state,
            ) {
                Ok(commitment) => {
                    self.beefy_light_client_state.set(&light_client_state);
                    if let Err(err) =
                        beefy_light_client::verifier_for_external_state_data::verify_solochain_messages(
                            &encoded_messages,
                            &header,
                            &commitment,
                            &mmr_leaf_for_header,
                            &mmr_proof_for_header,
                        )
                    {
                        return MultiTxsOperationProcessingResult::Error(format!(
                            "Successfully verified the signed commitment, but failed to verify messages: {:?}",
                            err
                        ));
                    }
                }
                Err(err) => {
                    return MultiTxsOperationProcessingResult::Error(format!(
                        "Failed to verify signed commitment: {:?}",
                        err
                    ))
                }
            }
        }
        let messages = message_decoder::decode(encoded_messages);
        self.internal_stage_appchain_messages(&messages);
        let processing_status = self.permissionless_actions_status.get().unwrap();
        let mut processing_context = AppchainMessagesProcessingContext::new(processing_status);
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        let mut appchain_messages = self.appchain_messages.get().unwrap();
        for message in messages {
            if env::used_gas() < Gas::ONE_TERA.mul(T_GAS_CAP_FOR_PROCESSING_APPCHAIN_MESSAGES) {
                if appchain_messages
                    .get_processing_result(&message.nonce)
                    .is_none()
                {
                    self.internal_apply_appchain_message(
                        &mut processing_context,
                        &mut validator_set_histories,
                        &mut appchain_messages,
                        &message,
                    );
                }
            } else {
                processing_context
                    .processing_status
                    .add_nonce_to_queue(message.nonce);
            }
        }
        self.validator_set_histories.set(&validator_set_histories);
        self.appchain_messages.set(&appchain_messages);
        processing_context.processing_status.sort_nonces_in_queue();
        self.permissionless_actions_status
            .set(&processing_context.processing_status);
        if env::used_gas() < Gas::ONE_TERA.mul(T_GAS_CAP_FOR_PROCESSING_APPCHAIN_MESSAGES) {
            MultiTxsOperationProcessingResult::Ok
        } else {
            MultiTxsOperationProcessingResult::NeedMoreGas
        }
    }
}

impl AppchainAnchor {
    ///
    pub fn internal_stage_appchain_messages(&mut self, messages: &Vec<AppchainMessage>) {
        let mut processing_status = self.permissionless_actions_status.get().unwrap();
        let mut appchain_messages = self.appchain_messages.get().unwrap();
        let protocol_settings = self.protocol_settings.get().unwrap();
        messages
            .iter()
            .filter(|message| match message.appchain_event {
                AppchainEvent::EraRewardConcluded { era_number, .. } => !self
                    .era_number_is_too_old(
                        u64::from(era_number),
                        protocol_settings
                            .maximum_era_count_of_valid_appchain_message
                            .0,
                    ),
                _ => true,
            })
            .for_each(|message| {
                appchain_messages.insert_message(message);
                match message.appchain_event {
                    AppchainEvent::EraSwitchPlaned { .. }
                    | AppchainEvent::EraRewardConcluded { .. } => {
                        processing_status.add_nonce_to_queue(message.nonce)
                    }
                    _ => (),
                }
            });
        self.appchain_messages.set(&appchain_messages);
        processing_status.sort_nonces_in_queue();
        self.permissionless_actions_status.set(&processing_status);
    }
    /// Apply a certain `AppchainMessage`
    pub fn internal_apply_appchain_message(
        &mut self,
        processing_context: &mut AppchainMessagesProcessingContext,
        validator_set_histories: &mut LookupArray<ValidatorSetOfEra>,
        appchain_messages: &mut AppchainMessages,
        appchain_message: &AppchainMessage,
    ) -> MultiTxsOperationProcessingResult {
        match &appchain_message.appchain_event {
            AppchainEvent::NearFungibleTokenBurnt {
                contract_account,
                owner_id_in_appchain,
                receiver_id_in_near,
                amount,
            } => {
                if self.asset_transfer_is_paused {
                    let message = format!("Asset transfer is now paused.");
                    let result = AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message: message.clone(),
                    };
                    appchain_messages.insert_processing_result(&result);
                    return MultiTxsOperationProcessingResult::Error(message);
                }
                let contract_account_id = AccountId::from_str(&contract_account);
                if contract_account_id.is_err() {
                    let message = format!("Invalid contract account: '{}'.", contract_account);
                    let result = AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message: message.clone(),
                    };
                    appchain_messages.insert_processing_result(&result);
                    return MultiTxsOperationProcessingResult::Error(message);
                }
                self.internal_unlock_near_fungible_token(
                    owner_id_in_appchain,
                    &contract_account_id.unwrap(),
                    receiver_id_in_near,
                    amount,
                    appchain_message.nonce,
                    processing_context,
                    appchain_messages,
                )
            }
            AppchainEvent::NativeTokenLocked {
                owner_id_in_appchain,
                receiver_id_in_near,
                amount,
            } => {
                if self.asset_transfer_is_paused {
                    let message = format!("Asset transfer is now paused.");
                    let result = AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message: message.clone(),
                    };
                    appchain_messages.insert_processing_result(&result);
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
                    appchain_messages.insert_processing_result(&result);
                    return MultiTxsOperationProcessingResult::Error(message);
                }
                self.internal_mint_wrapped_appchain_token(
                    Some(owner_id_in_appchain),
                    receiver_id_in_near,
                    amount,
                    appchain_message.nonce,
                    processing_context,
                    appchain_messages,
                )
            }
            AppchainEvent::EraSwitchPlaned { era_number } => {
                if let Some(era_number) = processing_context.switching_era_number() {
                    self.complete_switching_era(
                        processing_context,
                        validator_set_histories,
                        appchain_messages,
                        era_number,
                    )
                } else {
                    let index_range = validator_set_histories.index_range();
                    if u64::from(*era_number) != index_range.end_index.0 + 1 {
                        let message = format!(
                            "Invalid era number '{}': it must be '{}'.",
                            era_number,
                            index_range.end_index.0 + 1
                        );
                        let result = AppchainMessageProcessingResult::Error {
                            nonce: appchain_message.nonce,
                            message: message.clone(),
                        };
                        appchain_messages.insert_processing_result(&result);
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
                        appchain_messages,
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
            } => {
                if self.asset_transfer_is_paused {
                    let message = format!("Asset transfer is now paused.");
                    let result = AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message: message.clone(),
                    };
                    appchain_messages.insert_processing_result(&result);
                    return MultiTxsOperationProcessingResult::Error(message);
                }
                self.internal_process_locked_nft_in_appchain(
                    processing_context,
                    appchain_messages,
                    appchain_message.nonce,
                    owner_id_in_appchain,
                    receiver_id_in_near,
                    class_id,
                    instance_id,
                    token_metadata,
                )
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

impl AppchainMessageProcessingResult {
    pub fn nonce(&self) -> u32 {
        match self {
            AppchainMessageProcessingResult::Ok { nonce, .. }
            | AppchainMessageProcessingResult::Error { nonce, .. } => *nonce,
        }
    }
}
