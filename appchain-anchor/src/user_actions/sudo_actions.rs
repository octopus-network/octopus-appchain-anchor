use crate::permissionless_actions::AppchainMessagesProcessingContext;
use crate::*;
use crate::{appchain_messages::AppchainMessage, interfaces::SudoActions};
use codec::Decode;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;

#[near_bindgen]
impl SudoActions for AppchainAnchor {
    //
    fn set_owner_pk(&mut self, public_key: PublicKey) {
        self.assert_owner();
        self.owner_pk = public_key;
    }
    //
    fn stage_appchain_message(&mut self, appchain_message: AppchainMessage) {
        self.assert_owner();
        let mut processing_status = self.permissionless_actions_status.get().unwrap();
        let mut appchain_messages = self.appchain_messages.get().unwrap();
        appchain_messages.insert_message(&appchain_message);
        self.appchain_messages.set(&appchain_messages);
        processing_status.max_nonce_of_staged_appchain_messages = appchain_messages.max_nonce();
        self.permissionless_actions_status.set(&processing_status);
    }
    //
    fn stage_appchain_encoded_messages(&mut self, encoded_messages: Vec<u8>) {
        self.assert_owner();
        let messages = Decode::decode(&mut &encoded_messages[..]).unwrap();
        self.internal_stage_appchain_messages(&messages);
    }
    //
    fn set_metadata_of_wrapped_appchain_token(&mut self, metadata: FungibleTokenMetadata) {
        self.assert_owner();
        let mut wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
        wrapped_appchain_token.metadata = metadata;
        self.wrapped_appchain_token.set(&wrapped_appchain_token);
    }
    //
    fn set_premined_balance_of_wrapped_appchain_token(
        &mut self,
        premined_beneficiary: AccountId,
        premined_balance: U128,
    ) {
        self.assert_owner();
        let mut wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
        wrapped_appchain_token.premined_beneficiary = Some(premined_beneficiary);
        wrapped_appchain_token.premined_balance = premined_balance;
        self.wrapped_appchain_token.set(&wrapped_appchain_token);
    }
    //
    fn reset_validator_set_histories_to(&mut self, era_number: U64) {
        self.assert_owner();
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        validator_set_histories.reset_to(&era_number.0);
        self.validator_set_histories.set(&validator_set_histories);
    }
    //
    fn reset_staking_histories_to(&mut self, era_number: U64) {
        self.assert_owner();
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        if let Some(validator_set_of_era) = validator_set_histories.get(&era_number.0) {
            let mut staking_histories = self.staking_histories.get().unwrap();
            staking_histories.reset_to(&validator_set_of_era.staking_history_index());
            self.staking_histories.set(&staking_histories);
        }
    }
    //
    fn refresh_user_staking_histories(&mut self) {
        self.assert_owner();
        let mut user_staking_histories = self.user_staking_histories.get().unwrap();
        user_staking_histories.clear();
        let staking_histories = self.staking_histories.get().unwrap();
        let index_range = staking_histories.index_range();
        for index in index_range.start_index.0..index_range.end_index.0 + 1 {
            if let Some(staking_history) = staking_histories.get(&index) {
                user_staking_histories.add_staking_history(&staking_history);
            }
        }
        self.user_staking_histories.set(&user_staking_histories);
    }
    //
    fn reset_next_validator_set_to(&mut self, era_number: U64) {
        self.assert_owner();
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        if let Some(validator_set_of_era) = validator_set_histories.get(&era_number.0) {
            let mut next_validator_set = self.next_validator_set.get().unwrap();
            next_validator_set.clear();
            let staking_history_index = validator_set_of_era.staking_history_index();
            let staking_histories = self.staking_histories.get().unwrap();
            for index in 0..staking_history_index + 1 {
                if let Some(staking_history) = staking_histories.get(&index) {
                    next_validator_set.apply_staking_fact(&staking_history.staking_fact);
                }
            }
            self.next_validator_set.set(&next_validator_set);
        }
    }
    //
    fn clear_appchain_notification_histories(&mut self) {
        self.assert_owner();
        let mut appchain_notification_histories =
            self.appchain_notification_histories.get().unwrap();
        appchain_notification_histories.clear();
        self.appchain_notification_histories
            .set(&appchain_notification_histories);
    }
    //
    fn reset_beefy_light_client(&mut self, initial_public_keys: Vec<String>) {
        self.assert_owner();
        self.beefy_light_client_state
            .set(&beefy_light_client::new(initial_public_keys));
    }
    //
    fn clear_reward_distribution_records(&mut self, era_number: U64) {
        self.assert_owner();
        let mut reward_distribution_records = self.reward_distribution_records.get().unwrap();
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        if let Some(mut validator_set_of_era) = validator_set_histories.get(&era_number.0) {
            reward_distribution_records.clear(&validator_set_of_era, &era_number.0);
            self.reward_distribution_records
                .set(&reward_distribution_records);
            validator_set_of_era.clear_reward_distribution_records();
            validator_set_histories.insert(&era_number.0, &validator_set_of_era);
        }
    }
    //
    fn clear_unbonded_stakes(&mut self) {
        self.assert_owner();
        let staking_histories = self.staking_histories.get().unwrap();
        let index_range = staking_histories.index_range();
        for index in index_range.start_index.0..index_range.end_index.0 + 1 {
            if let Some(staking_history) = staking_histories.get(&index) {
                match staking_history.staking_fact {
                    StakingFact::StakeDecreased { validator_id, .. }
                    | StakingFact::ValidatorUnbonded { validator_id, .. }
                    | StakingFact::ValidatorAutoUnbonded { validator_id, .. } => {
                        self.unbonded_stakes.remove(&validator_id);
                    }
                    StakingFact::DelegationDecreased { delegator_id, .. }
                    | StakingFact::DelegatorUnbonded { delegator_id, .. }
                    | StakingFact::DelegatorAutoUnbonded { delegator_id, .. } => {
                        self.unbonded_stakes.remove(&delegator_id);
                    }
                    _ => (),
                }
            }
        }
    }
    //
    fn clear_unwithdrawn_rewards(&mut self, era_number: U64) {
        self.assert_owner();
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        if let Some(validator_set_of_era) = validator_set_histories.get(&era_number.0) {
            validator_set_of_era
                .get_validator_ids()
                .iter()
                .for_each(|validator_id| {
                    validator_set_of_era
                        .get_delegator_ids_of(validator_id)
                        .iter()
                        .for_each(|delegator_id| {
                            self.unwithdrawn_delegator_rewards.remove(&(
                                era_number.0,
                                delegator_id.clone(),
                                validator_id.clone(),
                            ));
                        });
                    self.unwithdrawn_validator_rewards
                        .remove(&(era_number.0, validator_id.clone()));
                });
        }
    }
    //
    fn reset_validator_profiles_to(&mut self, era_number: U64) {
        self.assert_owner();
        let mut validator_profiles = self.validator_profiles.get().unwrap();
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        let mut data_changed = false;
        if let Some(validator_set_of_era) = validator_set_histories.get(&era_number.0) {
            validator_profiles
                .get_validator_ids()
                .iter()
                .for_each(|validator_id| {
                    if !validator_set_of_era.contains_validator(validator_id) {
                        if validator_profiles.remove(validator_id) {
                            data_changed = true;
                        }
                    }
                });
        }
        if data_changed {
            self.validator_profiles.set(&validator_profiles);
        }
    }
    //
    fn pause_asset_transfer(&mut self) {
        self.assert_owner();
        assert!(
            !self.asset_transfer_is_paused,
            "Asset transfer is already paused."
        );
        self.asset_transfer_is_paused = true;
    }
    //
    fn resume_asset_transfer(&mut self) {
        self.assert_owner();
        assert!(
            self.asset_transfer_is_paused,
            "Asset transfer is already resumed."
        );
        self.asset_transfer_is_paused = false;
    }
    //
    fn remove_staking_history_at(&mut self, index: U64) {
        self.assert_owner();
        let mut staking_histories = self.staking_histories.get().unwrap();
        staking_histories.remove_at(&index.0);
    }
    //
    fn pause_rewards_withdrawal(&mut self) {
        self.assert_owner();
        assert!(
            !self.rewards_withdrawal_is_paused,
            "Rewards withdrawal is already paused."
        );
        self.rewards_withdrawal_is_paused = true;
    }
    //
    fn resume_rewards_withdrawal(&mut self) {
        self.assert_owner();
        assert!(
            self.rewards_withdrawal_is_paused,
            "Rewards withdrawal is already resumed."
        );
        self.rewards_withdrawal_is_paused = false;
    }
    //
    fn change_account_id_in_appchain_of_validator(
        &mut self,
        validator_id: AccountId,
        account_id_in_appchain: String,
    ) {
        self.assert_owner();
        self.internal_change_account_id_in_appchain_of_validator(
            &validator_id,
            &account_id_in_appchain,
        );
    }
    //
    fn force_change_account_id_in_appchain_of_staking_history(
        &mut self,
        index: U64,
        account_id_in_appchain: String,
    ) {
        self.assert_owner();
        let mut staking_histories = self.staking_histories.get().unwrap();
        if let Some(mut staking_history) = staking_histories.get(&index.0) {
            match staking_history.staking_fact {
                StakingFact::ValidatorRegistered {
                    validator_id,
                    validator_id_in_appchain: _,
                    amount,
                    can_be_delegated_to,
                } => {
                    staking_history.staking_fact = StakingFact::ValidatorRegistered {
                        validator_id,
                        validator_id_in_appchain: account_id_in_appchain,
                        amount,
                        can_be_delegated_to,
                    };
                    staking_histories.insert(&index.0, &staking_history);
                    self.staking_histories.set(&staking_histories);
                }
                _ => (),
            }
        }
    }
    //
    fn remove_duplicated_message_nonces_in_reward_distribution_records(&mut self, era_number: U64) {
        self.assert_owner();
        let mut reward_distribution_records = self.reward_distribution_records.get().unwrap();
        reward_distribution_records.remove_duplicated_message_nonces(era_number.0);
        self.reward_distribution_records
            .set(&reward_distribution_records);
    }
    //
    fn set_latest_applied_appchain_message_nonce(&mut self, nonce: u32) {
        self.assert_owner();
        let mut permissionless_actions_status = self.permissionless_actions_status.get().unwrap();
        permissionless_actions_status.latest_applied_appchain_message_nonce = nonce;
        permissionless_actions_status.processing_appchain_message_nonce = None;
        self.permissionless_actions_status
            .set(&permissionless_actions_status);
    }
    //
    fn clear_appchain_messages(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let mut appchain_messages = self.appchain_messages.get().unwrap();
        let result = appchain_messages.clear();
        self.appchain_messages.set(&appchain_messages);
        result
    }
    //
    fn try_complete_switching_era(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let processing_status = self.permissionless_actions_status.get().unwrap();
        let mut processing_context = AppchainMessagesProcessingContext::new(processing_status);
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        if let Some(era_number) = processing_context.switching_era_number() {
            let result = self.complete_switching_era(
                &mut processing_context,
                &mut validator_set_histories,
                era_number,
            );
            self.permissionless_actions_status
                .set(processing_context.processing_status());
            result
        } else {
            MultiTxsOperationProcessingResult::Ok
        }
    }
}
