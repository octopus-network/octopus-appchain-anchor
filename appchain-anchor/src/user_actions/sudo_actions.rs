use crate::permissionless_actions::AppchainMessagesProcessingContext;
use crate::*;
use crate::{appchain_messages::AppchainMessage, interfaces::SudoActions};
use codec::Decode;
use core::panic;
use near_contract_standards::fungible_token::core::ext_ft_core;
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
    fn reset_validator_set_histories_to(
        &mut self,
        era_number: U64,
    ) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        let result = validator_set_histories.reset_to(&era_number.0);
        if !result.is_error() {
            self.validator_set_histories.set(&validator_set_histories);
        }
        result
    }
    //
    fn reset_staking_histories_to(&mut self, era_number: U64) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        if let Some(validator_set_of_era) = validator_set_histories.get(&era_number.0) {
            let mut staking_histories = self.staking_histories.get().unwrap();
            let result = staking_histories.reset_to(&validator_set_of_era.staking_history_index());
            if !result.is_error() {
                self.staking_histories.set(&staking_histories);
            }
            return result;
        } else {
            panic!(
                "Missing validator set history of era_number '{}'.",
                era_number.0
            );
        }
    }
    //
    fn clear_user_staking_histories(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let mut user_staking_histories = self.user_staking_histories.get().unwrap();
        let result = user_staking_histories.clear();
        if !result.is_error() {
            self.user_staking_histories.set(&user_staking_histories);
        }
        result
    }
    //
    fn regenerate_user_staking_histories(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let mut user_staking_histories = self.user_staking_histories.get().unwrap();
        let staking_histories = self.staking_histories.get().unwrap();
        let index_range = staking_histories.index_range();
        for index in index_range.start_index.0..index_range.end_index.0 + 1 {
            if let Some(staking_history) = staking_histories.get(&index) {
                user_staking_histories.add_staking_history(&staking_history);
            }
            if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
                self.user_staking_histories.set(&user_staking_histories);
                return MultiTxsOperationProcessingResult::NeedMoreGas;
            }
        }
        self.user_staking_histories.set(&user_staking_histories);
        MultiTxsOperationProcessingResult::Ok
    }
    //
    // While using this function to reset data of next validator set, this contract must
    // refuse any other actions which will change the state of next validator set.
    //
    fn reset_next_validator_set_to(
        &mut self,
        era_number: U64,
    ) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        if let Some(validator_set_of_era) = validator_set_histories.get(&era_number.0) {
            let mut next_validator_set = self.next_validator_set.get().unwrap();
            let result = next_validator_set.clear();
            match result {
                MultiTxsOperationProcessingResult::Ok => (),
                MultiTxsOperationProcessingResult::NeedMoreGas => {
                    self.next_validator_set.set(&next_validator_set);
                    return result;
                }
                MultiTxsOperationProcessingResult::Error(_) => {
                    return result;
                }
            }
            let staking_history_index = validator_set_of_era.staking_history_index();
            let staking_histories = self.staking_histories.get().unwrap();
            for index in 0..staking_history_index + 1 {
                if let Some(staking_history) = staking_histories.get(&index) {
                    next_validator_set.apply_staking_fact(&staking_history.staking_fact);
                }
                if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
                    self.next_validator_set.set(&next_validator_set);
                    return MultiTxsOperationProcessingResult::NeedMoreGas;
                }
            }
            self.next_validator_set.set(&next_validator_set);
            MultiTxsOperationProcessingResult::Ok
        } else {
            panic!(
                "Missing validator set history of era_number '{}'.",
                era_number.0
            );
        }
    }
    //
    fn clear_appchain_notification_histories(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let mut appchain_notification_histories =
            self.appchain_notification_histories.get().unwrap();
        let result = appchain_notification_histories.clear();
        if !result.is_error() {
            self.appchain_notification_histories
                .set(&appchain_notification_histories);
        }
        result
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
        if !result.is_error() {
            self.appchain_messages.set(&appchain_messages);
        }
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
    //
    fn remove_validator_set_history_of(
        &mut self,
        era_number: U64,
    ) -> MultiTxsOperationProcessingResult {
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        let result = validator_set_histories.remove_at(&era_number.0);
        if !result.is_error() {
            self.validator_set_histories.set(&validator_set_histories);
        }
        result
    }
    //
    fn remove_validator_set_histories_before(
        &mut self,
        era_number: U64,
    ) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        let result = validator_set_histories.remove_before(&era_number.0);
        if !result.is_error() {
            self.validator_set_histories.set(&validator_set_histories);
        }
        result
    }
    //
    fn unlock_auto_unbonded_stake_of(
        &mut self,
        delegator_id: Option<AccountId>,
        validator_id: AccountId,
        staking_history_index: U64,
    ) {
        self.assert_owner();
        self.assert_asset_transfer_is_not_paused();
        let unbonded_stake_references = match delegator_id.clone() {
            Some(delegator_id) => self.unbonded_stakes.get(&delegator_id).unwrap(),
            None => self.unbonded_stakes.get(&validator_id).unwrap(),
        };
        let staking_histories = self.staking_histories.get().unwrap();
        let mut remained_stakes = Vec::<UnbondedStakeReference>::new();
        let mut found = false;
        for reference in unbonded_stake_references {
            if reference.staking_history_index == staking_history_index.0 {
                let staking_history = staking_histories.get(&staking_history_index.0).unwrap();
                match staking_history.staking_fact {
                    StakingFact::ValidatorAutoUnbonded {
                        validator_id: unbonded_validator_id @ _,
                        amount,
                    } => {
                        assert!(
                            validator_id.eq(&unbonded_validator_id),
                            "Invalid staking history for validator '{}'.",
                            validator_id
                        );
                        ext_ft_core::ext(self.oct_token.get().unwrap().contract_account)
                            .with_attached_deposit(1)
                            .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_FT_TRANSFER))
                            .with_unused_gas_weight(0)
                            .ft_transfer(validator_id.clone(), amount.clone(), None);
                        found = true;
                        break;
                    }
                    StakingFact::DelegatorAutoUnbonded {
                        delegator_id: unbonded_delegator_id @ _,
                        validator_id: unbonded_validator_id @ _,
                        amount,
                    } => {
                        let delegator_id = delegator_id
                            .clone()
                            .unwrap_or(AccountId::new_unchecked(String::new()));
                        assert!(
                            validator_id.eq(&unbonded_validator_id)
                                && delegator_id.eq(&unbonded_delegator_id),
                            "Invalid staking history for delegator '{}' of validator '{}'.",
                            delegator_id,
                            validator_id
                        );
                        ext_ft_core::ext(self.oct_token.get().unwrap().contract_account)
                            .with_attached_deposit(1)
                            .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_FT_TRANSFER))
                            .with_unused_gas_weight(0)
                            .ft_transfer(delegator_id.clone(), amount.clone(), None);
                        found = true;
                        break;
                    }
                    _ => {
                        remained_stakes.push(reference.clone());
                    }
                }
            } else {
                remained_stakes.push(reference.clone());
            }
        }
        assert!(found, "Specified staking history is not found.");
        if remained_stakes.len() > 0 {
            self.unbonded_stakes
                .insert(&delegator_id.unwrap_or(validator_id), &remained_stakes);
        } else {
            self.unbonded_stakes
                .remove(&delegator_id.unwrap_or(validator_id));
        }
    }
}
