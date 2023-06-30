use crate::interfaces::SudoActions;
use crate::*;
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
    fn reset_beefy_light_client(&mut self, initial_public_keys: Vec<String>) {
        self.assert_owner();
        self.beefy_light_client_state
            .set(&beefy_light_client::new(initial_public_keys));
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
    fn set_latest_applied_appchain_message_nonce(&mut self, nonce: u32) {
        self.assert_owner();
        let mut permissionless_actions_status = self.permissionless_actions_status.get().unwrap();
        permissionless_actions_status.latest_applied_appchain_message_nonce = nonce;
        permissionless_actions_status.processing_appchain_message_nonce = None;
        self.permissionless_actions_status
            .set(&permissionless_actions_status);
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
    //
    fn remove_oldest_validator_set(&mut self) -> String {
        self.assert_owner();
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        let anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            validator_set_histories.len() > anchor_settings.min_length_of_validator_set_history.0,
            "The length of validator set histories must not be less than {}.",
            anchor_settings.min_length_of_validator_set_history.0
        );
        let max_gas = Gas::ONE_TERA.mul(170);
        let mut era_number = validator_set_histories.index_range().start_index;
        while env::used_gas() < max_gas && validator_set_histories.get(&era_number.0).is_none() {
            validator_set_histories.remove_first(max_gas);
            era_number = validator_set_histories.index_range().start_index;
        }
        if validator_set_histories.len() <= anchor_settings.min_length_of_validator_set_history.0 {
            self.validator_set_histories.set(&validator_set_histories);
            return format!(
                "Era {}: {:?}",
                era_number.0,
                MultiTxsOperationProcessingResult::Ok
            );
        }
        let mut validator_set_of_era = validator_set_histories.get(&era_number.0).unwrap();
        let mut result = (MultiTxsOperationProcessingResult::NeedMoreGas, None);
        while env::used_gas() < max_gas && result.0.is_need_more_gas() {
            result = match RemovingValidatorSetSteps::recover() {
                RemovingValidatorSetSteps::ClearingRewardDistributionRecords {
                    appchain_message_nonce_index,
                    validator_index,
                    delegator_index,
                } => {
                    let mut reward_distribution_records =
                        self.reward_distribution_records.get().unwrap();
                    let mut result = reward_distribution_records.clear(
                        &validator_set_of_era,
                        &era_number.0,
                        appchain_message_nonce_index,
                        validator_index,
                        delegator_index,
                        max_gas,
                    );
                    self.reward_distribution_records
                        .set(&reward_distribution_records);
                    if result.is_ok() {
                        RemovingValidatorSetSteps::ClearingRewardDistributionRecordsInValidatorSet {
                            validator_index: 0,
                            delegator_index: 0,
                        }
                        .save();
                        result = MultiTxsOperationProcessingResult::NeedMoreGas;
                    }
                    (result, Some(RemovingValidatorSetSteps::recover()))
                }
                RemovingValidatorSetSteps::ClearingRewardDistributionRecordsInValidatorSet {
                    validator_index,
                    delegator_index,
                } => {
                    let mut result = validator_set_of_era.clear_reward_distribution_records(
                        validator_index,
                        delegator_index,
                        max_gas,
                    );
                    validator_set_histories.insert(&era_number.0, &validator_set_of_era);
                    if result.is_ok() {
                        RemovingValidatorSetSteps::ClearingUnwithdrawnRewardRecordsForValidatorSet {
                            validator_index: 0,
                            delegator_index: 0,
                        }
                        .save();
                        result = MultiTxsOperationProcessingResult::NeedMoreGas;
                    }
                    (result, Some(RemovingValidatorSetSteps::recover()))
                }
                RemovingValidatorSetSteps::ClearingUnwithdrawnRewardRecordsForValidatorSet {
                    validator_index,
                    delegator_index,
                } => {
                    let mut result = self.clear_unwithdrawn_reward_records(
                        &validator_set_of_era,
                        validator_index,
                        delegator_index,
                        max_gas,
                    );
                    if result.is_ok() {
                        RemovingValidatorSetSteps::ClearingOldestValidatorSet.save();
                        result = MultiTxsOperationProcessingResult::NeedMoreGas;
                    }
                    (result, Some(RemovingValidatorSetSteps::recover()))
                }
                RemovingValidatorSetSteps::ClearingOldestValidatorSet => {
                    let result = validator_set_histories.remove_first(max_gas);
                    if result.is_ok() {
                        RemovingValidatorSetSteps::clear();
                        (result, None)
                    } else {
                        (result, Some(RemovingValidatorSetSteps::recover()))
                    }
                }
            };
        }
        self.validator_set_histories.set(&validator_set_histories);
        format!("Era {}: {:?}", era_number.0, result)
    }
    //
    fn remove_old_appchain_messages(&mut self) -> String {
        self.assert_owner();
        let mut appchain_messages = self.appchain_messages.get().unwrap();
        let reward_distribution_records = self.reward_distribution_records.get().unwrap();
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        let nonces = reward_distribution_records
            .get_message_nonces_of_era(&validator_set_histories.index_range().start_index.0);
        let result = match nonces.len() == 0 {
            true => MultiTxsOperationProcessingResult::Error(format!(
                "Missing reward distribution records of era {}. Can not proceed.",
                validator_set_histories.index_range().start_index.0
            )),
            false => appchain_messages.remove_messages_before(&nonces[0], Gas::ONE_TERA.mul(180)),
        };
        self.appchain_messages.set(&appchain_messages);
        format!(
            "Result: {:?}, ({}, {})",
            result,
            appchain_messages.min_nonce(),
            appchain_messages.max_nonce()
        )
    }
    //
    fn remove_old_appchain_notification(&mut self) -> String {
        self.assert_owner();
        let mut appchain_notification_histories =
            self.appchain_notification_histories.get().unwrap();
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        let oldest_validator_set = validator_set_histories.get_first().unwrap();
        let max_gas = Gas::ONE_TERA.mul(180);
        let mut result = MultiTxsOperationProcessingResult::NeedMoreGas;
        while env::used_gas() <= max_gas {
            if let Some(notification) = appchain_notification_histories.get_first() {
                if notification.timestamp.0 >= oldest_validator_set.start_timestamp() {
                    result = MultiTxsOperationProcessingResult::Ok;
                    break;
                }
            }
            appchain_notification_histories.remove_first(max_gas);
        }
        self.appchain_notification_histories
            .set(&appchain_notification_histories);
        let index_range = appchain_notification_histories.index_range();
        format!(
            "Result: {:?}, ({}, {})",
            result, index_range.start_index.0, index_range.end_index.0
        )
    }
    //
    fn remove_staged_wasm(&mut self) {
        self.assert_owner();
        log!(
            "AnchorContractWasm: {}",
            env::storage_remove(&StorageKey::AnchorContractWasm.into_bytes())
        );
        log!(
            "WrappedAppchainNFTContractWasm: {}",
            env::storage_remove(&StorageKey::WrappedAppchainNFTContractWasm.into_bytes())
        );
        log!(
            "NearVaultContractWasm: {}",
            env::storage_remove(&StorageKey::NearVaultContractWasm.into_bytes())
        );
    }
}

impl AppchainAnchor {
    pub fn clear_unwithdrawn_reward_records(
        &mut self,
        validator_set_of_era: &ValidatorSetOfEra,
        validator_index_start: u64,
        delegator_index_start: u64,
        max_gas: Gas,
    ) -> MultiTxsOperationProcessingResult {
        let era_number = validator_set_of_era.era_number();
        let validator_ids = validator_set_of_era.get_validator_ids();
        let mut validator_index = 0;
        let mut delegator_index = 0;
        for validator_id in validator_ids {
            if validator_index < validator_index_start {
                validator_index += 1;
                continue;
            }
            let delegator_ids = validator_set_of_era.get_delegator_ids_of(&validator_id);
            for delegator_id in delegator_ids {
                if validator_index == validator_index_start
                    && delegator_index < delegator_index_start
                {
                    delegator_index += 1;
                    continue;
                }
                self.unwithdrawn_delegator_rewards.remove(&(
                    era_number,
                    delegator_id.clone(),
                    validator_id.clone(),
                ));
                delegator_index += 1;
                if env::used_gas() >= max_gas {
                    RemovingValidatorSetSteps::ClearingUnwithdrawnRewardRecordsForValidatorSet {
                        validator_index,
                        delegator_index,
                    }
                    .save();
                    return MultiTxsOperationProcessingResult::NeedMoreGas;
                }
            }
            self.unwithdrawn_validator_rewards
                .remove(&(era_number, validator_id.clone()));
            validator_index += 1;
            delegator_index = 0;
            if env::used_gas() >= max_gas {
                RemovingValidatorSetSteps::ClearingUnwithdrawnRewardRecordsForValidatorSet {
                    validator_index,
                    delegator_index,
                }
                .save();
                return MultiTxsOperationProcessingResult::NeedMoreGas;
            }
        }
        MultiTxsOperationProcessingResult::Ok
    }
}
