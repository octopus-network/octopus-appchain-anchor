use crate::*;

impl AppchainAnchor {
    //
    pub fn assert_testnet(&self) {
        let contract_account = env::current_account_id().to_string();
        assert!(
            contract_account.ends_with(".testnet"),
            "This function can only by used on testnet account."
        );
    }
}

#[near_bindgen]
impl AppchainAnchor {
    //
    pub fn clear_contract_level_lazy_option_values(&mut self) {
        self.assert_testnet();
        self.assert_owner();
        let near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        let next_validator_set = self.next_validator_set.get().unwrap();
        let validator_profiles = self.validator_profiles.get().unwrap();
        let staking_histories = self.staking_histories.get().unwrap();
        let appchain_notification_histories = self.appchain_notification_histories.get().unwrap();
        let reward_distribution_records = self.reward_distribution_records.get().unwrap();
        let user_staking_histories = self.user_staking_histories.get().unwrap();
        let appchain_messages = self.appchain_messages.get().unwrap();
        let appchain_challenges = self.appchain_challenges.get().unwrap();
        let wrapped_appchain_nfts = self.wrapped_appchain_nfts.get().unwrap();
        assert!(
            near_fungible_tokens.is_empty(),
            "'near_fungible_tokens' is not empty."
        );
        assert!(
            validator_set_histories.is_empty(),
            "'validator_set_histories' is not empty."
        );
        assert!(
            next_validator_set.is_empty(),
            "'next_validator_set' is not empty."
        );
        assert!(
            validator_profiles.is_empty(),
            "'validator_profiles' is not empty."
        );
        assert!(
            staking_histories.is_empty(),
            "'staking_histories' is not empty."
        );
        assert!(
            appchain_notification_histories.is_empty(),
            "'appchain_notification_histories' is not empty."
        );
        assert!(
            reward_distribution_records.is_empty(),
            "'reward_distribution_records' is not empty."
        );
        assert!(
            user_staking_histories.is_empty(),
            "'user_staking_histories' is not empty."
        );
        assert!(
            appchain_messages.is_empty(),
            "'appchain_messages' is not empty."
        );
        assert!(
            appchain_challenges.is_empty(),
            "'appchain_challenges' is not empty."
        );
        assert!(
            wrapped_appchain_nfts.is_empty(),
            "'wrapped_appchain_nfts' is not empty."
        );
        self.oct_token.remove();
        self.wrapped_appchain_token.remove();
        self.near_fungible_tokens.remove();
        self.validator_set_histories.remove();
        self.next_validator_set.remove();
        self.validator_profiles.remove();
        self.appchain_settings.remove();
        self.anchor_settings.remove();
        self.protocol_settings.remove();
        self.staking_histories.remove();
        self.appchain_notification_histories.remove();
        self.permissionless_actions_status.remove();
        self.beefy_light_client_state.remove();
        self.reward_distribution_records.remove();
        self.user_staking_histories.remove();
        self.appchain_messages.remove();
        self.appchain_challenges.remove();
        self.wrapped_appchain_nfts.remove();
    }
    //
    pub fn clear_external_assets_registration(&mut self) {
        self.assert_testnet();
        self.assert_owner();
        let mut near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        near_fungible_tokens.clear();
        self.near_fungible_tokens.set(&near_fungible_tokens);
        let mut wrapped_appchain_nfts = self.wrapped_appchain_nfts.get().unwrap();
        wrapped_appchain_nfts.clear();
        self.wrapped_appchain_nfts.set(&wrapped_appchain_nfts);
    }
    //
    pub fn remove_validator_set_history_of(
        &mut self,
        era_number: U64,
    ) -> MultiTxsOperationProcessingResult {
        self.assert_testnet();
        self.assert_owner();
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        let result = validator_set_histories.remove_at(&era_number.0);
        if !result.is_error() {
            self.validator_set_histories.set(&validator_set_histories);
        }
        result
    }
    //
    pub fn clear_staking_histories(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_testnet();
        self.assert_owner();
        let mut staking_histories = self.staking_histories.get().unwrap();
        let result = staking_histories.clear();
        if !result.is_error() {
            self.staking_histories.set(&staking_histories);
        }
        result
    }
    //
    pub fn clear_user_staking_histories(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_testnet();
        self.assert_owner();
        let mut user_staking_histories = self.user_staking_histories.get().unwrap();
        let result = user_staking_histories.clear();
        if !result.is_error() {
            self.user_staking_histories.set(&user_staking_histories);
        }
        result
    }
    //
    pub fn clear_next_validator_set(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_testnet();
        self.assert_owner();
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        let result = next_validator_set.clear();
        if !result.is_error() {
            self.next_validator_set.set(&next_validator_set);
        }
        result
    }
    //
    pub fn clear_appchain_notification_histories(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_testnet();
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
    pub fn clear_appchain_messages(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_testnet();
        self.assert_owner();
        let mut appchain_messages = self.appchain_messages.get().unwrap();
        let result = appchain_messages.clear();
        if !result.is_error() {
            self.appchain_messages.set(&appchain_messages);
        }
        result
    }
    //
    pub fn clear_reward_distribution_records(&mut self, era_number: U64) {
        self.assert_testnet();
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
    pub fn clear_unbonded_stakes(&mut self) {
        self.assert_testnet();
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
    pub fn clear_unwithdrawn_rewards(&mut self, era_number: U64) {
        self.assert_testnet();
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
    pub fn clear_validator_profiles(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_testnet();
        self.assert_owner();
        let mut validator_profiles = self.validator_profiles.get().unwrap();
        let result = validator_profiles.clear();
        if !result.is_error() {
            self.validator_profiles.set(&validator_profiles);
        }
        result
    }
    //
    pub fn clear_appchain_challenges(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_testnet();
        self.assert_owner();
        let mut appchain_challenges = self.appchain_challenges.get().unwrap();
        let result = appchain_challenges.clear();
        if !result.is_error() {
            self.appchain_challenges.set(&appchain_challenges);
        }
        result
    }
    //
    pub fn remove_validator_from_next_validator_set(&mut self, validator_id: AccountId) {
        self.assert_testnet();
        self.assert_owner();
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        next_validator_set.remove_validator(&validator_id);
        self.next_validator_set.set(&next_validator_set);
    }
    //
    pub fn clear_unbonding_flag_of_next_validator_set(&mut self) {
        self.assert_testnet();
        self.assert_owner();
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        next_validator_set.clear_auto_unbonding_validator_ids();
        next_validator_set.clear_unbonding_validator_ids();
        self.next_validator_set.set(&next_validator_set);
    }
}

impl NextValidatorSet {
    //
    pub fn remove_validator(&mut self, validator_id: &AccountId) {
        assert!(
            self.validator_set.validator_id_set.contains(validator_id),
            "Invalid validator id."
        );
        if self.auto_unbonding_validator_ids.contains(&validator_id) {
            let new_vec = self
                .auto_unbonding_validator_ids
                .drain(..)
                .filter(|e| !e.eq(validator_id))
                .collect();
            self.auto_unbonding_validator_ids = new_vec;
        }
        self.validator_set.validator_id_set.remove(validator_id);
        self.validator_set
            .validator_id_to_delegator_id_set
            .remove(&validator_id);
        let validator = self.validator_set.validators.remove(validator_id).unwrap();
        self.validator_set.total_stake -= validator.total_stake;
    }
}
