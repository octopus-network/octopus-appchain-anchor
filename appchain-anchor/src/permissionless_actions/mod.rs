mod distributing_rewards;
mod switching_era;

use crate::*;
use crate::{interfaces::PermissionlessActions, message_decoder::AppchainMessage};
use core::convert::TryInto;

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainEvent {
    /// The fact that a certain amount of bridge token has been burnt in the appchain.
    NearFungibleTokenBurnt {
        symbol: String,
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
}

enum ResultOfLoopingValidatorSet {
    NoMoreDelegator,
    NoMoreValidator,
    NeedToContinue,
}

#[near_bindgen]
impl PermissionlessActions for AppchainAnchor {
    ///
    fn start_updating_state_of_beefy_light_client(
        &mut self,
        signed_commitment: Vec<u8>,
        validator_proofs: Vec<ValidatorMerkleProof>,
        mmr_leaf: Vec<u8>,
        mmr_proof: Vec<u8>,
    ) {
        let anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            !anchor_settings.beefy_light_client_witness_mode,
            "Beefy light client is in witness mode."
        );
        self.assert_light_client_is_ready();
        let mut light_client = self.beefy_light_client_state.get().unwrap();
        if let Err(err) = light_client.start_updating_state(
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
            &mmr_leaf,
            &mmr_proof,
        ) {
            panic!(
                "Failed to start updating state of beefy light client: {:?}",
                err
            );
        }
        self.beefy_light_client_state.set(&light_client);
    }
    //
    fn try_complete_updating_state_of_beefy_light_client(
        &mut self,
    ) -> MultiTxsOperationProcessingResult {
        let anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            !anchor_settings.beefy_light_client_witness_mode,
            "Beefy light client is in witness mode."
        );
        self.assert_light_client_initialized();
        let mut light_client = self.beefy_light_client_state.get().unwrap();
        if !light_client.is_updating_state() {
            return MultiTxsOperationProcessingResult::Ok;
        }
        loop {
            match light_client.complete_updating_state(1) {
                Ok(flag) => match flag {
                    true => {
                        self.beefy_light_client_state.set(&light_client);
                        return MultiTxsOperationProcessingResult::Ok;
                    }
                    false => (),
                },
                Err(err) => {
                    self.beefy_light_client_state.set(&light_client);
                    return MultiTxsOperationProcessingResult::Error(format!("{:?}", err));
                }
            }
            if env::used_gas() > GAS_CAP_FOR_MULTI_TXS_PROCESSING {
                break;
            }
        }
        self.beefy_light_client_state.set(&light_client);
        MultiTxsOperationProcessingResult::NeedMoreGas
    }
    //
    fn verify_and_apply_appchain_messages(
        &mut self,
        encoded_messages: Vec<u8>,
        header: Vec<u8>,
        mmr_leaf: Vec<u8>,
        mmr_proof: Vec<u8>,
    ) -> Vec<AppchainMessageProcessingResult> {
        let anchor_settings = self.anchor_settings.get().unwrap();
        if anchor_settings.beefy_light_client_witness_mode {
            assert!(
                env::predecessor_account_id().eq(&anchor_settings.relayer_account),
                "Only relayer account can perform this action while beefy light client is in witness mode."
            );
        } else {
            self.assert_light_client_is_ready();
            let light_client = self.beefy_light_client_state.get().unwrap();
            if let Err(err) = light_client.verify_solochain_messages(
                &encoded_messages,
                &header,
                &mmr_leaf,
                &mmr_proof,
            ) {
                panic!("Failed in verifying appchain messages: {:?}", err);
            }
        }
        let messages = message_decoder::decode(encoded_messages);
        messages
            .iter()
            .map(|m| self.internal_apply_appchain_message(m.clone()))
            .collect::<Vec<AppchainMessageProcessingResult>>()
    }
    //
    fn try_complete_switching_era(&mut self) -> MultiTxsOperationProcessingResult {
        match self
            .permissionless_actions_status
            .get()
            .unwrap()
            .switching_era_number
        {
            Some(era_number) => {
                let completed = self.complete_switching_era(era_number.0);
                if completed {
                    let mut permissionless_actions_status =
                        self.permissionless_actions_status.get().unwrap();
                    permissionless_actions_status.switching_era_number = None;
                    self.permissionless_actions_status
                        .set(&permissionless_actions_status);
                    MultiTxsOperationProcessingResult::Ok
                } else {
                    MultiTxsOperationProcessingResult::NeedMoreGas
                }
            }
            None => MultiTxsOperationProcessingResult::Ok,
        }
    }
    //
    fn try_complete_distributing_reward(&mut self) -> MultiTxsOperationProcessingResult {
        match self
            .permissionless_actions_status
            .get()
            .unwrap()
            .distributing_reward_era_number
        {
            Some(era_number) => {
                let completed = self.complete_distributing_reward_of_era(era_number.0);
                if completed {
                    let mut permissionless_actions_status =
                        self.permissionless_actions_status.get().unwrap();
                    permissionless_actions_status.distributing_reward_era_number = None;
                    self.permissionless_actions_status
                        .set(&permissionless_actions_status);
                    MultiTxsOperationProcessingResult::Ok
                } else {
                    MultiTxsOperationProcessingResult::NeedMoreGas
                }
            }
            None => MultiTxsOperationProcessingResult::Ok,
        }
    }
}

impl AppchainAnchor {
    /// Apply a certain `AppchainMessage`
    pub fn internal_apply_appchain_message(
        &mut self,
        appchain_message: AppchainMessage,
    ) -> AppchainMessageProcessingResult {
        match appchain_message.appchain_event {
            permissionless_actions::AppchainEvent::NearFungibleTokenBurnt {
                symbol,
                owner_id_in_appchain,
                receiver_id_in_near,
                amount,
            } => {
                if self.asset_transfer_is_paused {
                    return AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message: format!("Asset transfer is now paused."),
                    };
                }
                self.internal_unlock_near_fungible_token(
                    owner_id_in_appchain,
                    symbol,
                    receiver_id_in_near,
                    amount,
                    appchain_message.nonce,
                )
            }
            permissionless_actions::AppchainEvent::NativeTokenLocked {
                owner_id_in_appchain,
                receiver_id_in_near,
                amount,
            } => {
                if self.asset_transfer_is_paused {
                    return AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message: format!("Asset transfer is now paused."),
                    };
                }
                let wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
                let protocol_settings = self.protocol_settings.get().unwrap();
                let owner_id = AccountIdInAppchain::new(Some(owner_id_in_appchain.clone()));
                owner_id.assert_valid();
                if wrapped_appchain_token.total_market_value()
                    + wrapped_appchain_token.get_market_value_of(amount.0)
                    > self.get_market_value_of_staked_oct_token().0
                        * u128::from(
                            protocol_settings
                                .maximum_market_value_percent_of_wrapped_appchain_token,
                        )
                        / 100
                {
                    let message = format!("Too much wrapped appchain token to mint.");
                    self.internal_append_anchor_event(
                        AnchorEvent::FailedToMintWrappedAppchainToken {
                            sender_id_in_appchain: Some(owner_id.to_string()),
                            receiver_id_in_near,
                            amount,
                            appchain_message_nonce: appchain_message.nonce,
                            reason: message.clone(),
                        },
                    );
                    AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message,
                    }
                } else {
                    self.internal_mint_wrapped_appchain_token(
                        Some(owner_id.to_string()),
                        receiver_id_in_near,
                        amount,
                        appchain_message.nonce,
                    )
                }
            }
            permissionless_actions::AppchainEvent::EraSwitchPlaned { era_number } => {
                self.assert_era_number_is_valid(u64::from(era_number));
                self.internal_start_switching_era(u64::from(era_number), appchain_message.nonce)
            }
            permissionless_actions::AppchainEvent::EraRewardConcluded {
                era_number,
                unprofitable_validator_ids,
            } => {
                self.assert_era_number_is_valid(u64::from(era_number));
                self.internal_start_distributing_reward_of_era(
                    appchain_message.nonce,
                    u64::from(era_number),
                    unprofitable_validator_ids,
                )
            }
        }
    }
    //
    fn assert_era_number_is_valid(&self, era_number: u64) {
        let protocol_settings = self.protocol_settings.get().unwrap();
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        let latest_era_number = validator_set_histories.index_range().end_index.0;
        if latest_era_number
            > protocol_settings
                .maximum_era_count_of_valid_appchain_message
                .0
        {
            assert!(
                era_number
                    >= latest_era_number
                        - protocol_settings
                            .maximum_era_count_of_valid_appchain_message
                            .0,
                "Message is too old."
            );
        }
    }
}
