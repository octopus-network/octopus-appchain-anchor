use crate::*;
use codec::Decode;
use core::panic;

#[near_bindgen]
impl AppchainAnchor {
    //
    pub fn stage_appchain_message(&mut self, appchain_message: AppchainMessage) {
        self.assert_owner();
        let mut processing_status = self.permissionless_actions_status.get().unwrap();
        let mut appchain_messages = self.appchain_messages.get().unwrap();
        appchain_messages.insert_message(&appchain_message);
        self.appchain_messages.set(&appchain_messages);
        processing_status.max_nonce_of_staged_appchain_messages = appchain_messages.max_nonce();
        self.permissionless_actions_status.set(&processing_status);
    }
    //
    pub fn stage_appchain_encoded_messages(&mut self, encoded_messages: Vec<u8>) {
        self.assert_owner();
        let messages = Decode::decode(&mut &encoded_messages[..]).unwrap();
        self.internal_stage_appchain_messages(&messages);
    }
    //
    pub fn reset_validator_set_histories_to(
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
    pub fn remove_validator_set_history_of(
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
    pub fn remove_validator_set_histories_before(
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
    pub fn reset_staking_histories_to(
        &mut self,
        era_number: U64,
    ) -> MultiTxsOperationProcessingResult {
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
    pub fn clear_user_staking_histories(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let mut user_staking_histories = self.user_staking_histories.get().unwrap();
        let result = user_staking_histories.clear();
        if !result.is_error() {
            self.user_staking_histories.set(&user_staking_histories);
        }
        result
    }
    //
    // While using this function to reset data of next validator set, this contract must
    // refuse any other actions which will change the state of next validator set.
    //
    pub fn reset_next_validator_set_to(
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
    pub fn clear_appchain_notification_histories(&mut self) -> MultiTxsOperationProcessingResult {
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
    pub fn reset_validator_profiles_to(&mut self, era_number: U64) {
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
    pub fn force_change_account_id_in_appchain_of_staking_history(
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
    pub fn remove_duplicated_message_nonces_in_reward_distribution_records(
        &mut self,
        era_number: U64,
    ) {
        self.assert_owner();
        let mut reward_distribution_records = self.reward_distribution_records.get().unwrap();
        reward_distribution_records.remove_duplicated_message_nonces(era_number.0);
        self.reward_distribution_records
            .set(&reward_distribution_records);
    }
}
