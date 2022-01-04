use crate::*;
use crate::{
    interfaces::SudoActions, message_decoder::AppchainMessage, validator_set::ValidatorSetActions,
};

use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;

#[near_bindgen]
impl SudoActions for AppchainAnchor {
    //
    fn apply_appchain_messages(
        &mut self,
        appchain_messages: Vec<AppchainMessage>,
    ) -> Vec<AppchainMessageProcessingResult> {
        self.assert_owner();
        appchain_messages
            .iter()
            .map(|m| self.internal_apply_appchain_message(m.clone()))
            .collect::<Vec<AppchainMessageProcessingResult>>()
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
        wrapped_appchain_token.premined_beneficiary = premined_beneficiary;
        wrapped_appchain_token.premined_balance = premined_balance;
        self.wrapped_appchain_token.set(&wrapped_appchain_token);
    }
    //
    fn reset_validator_set_histories_to(&mut self, era_number: U64) {
        self.assert_owner();
        // Clear validator set histories after the `era_number`
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        validator_set_histories.reset_to(&era_number.0);
        self.validator_set_histories.set(&validator_set_histories);
        // Copy the target validator set to `next_validator_set`
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        next_validator_set.clear();
        let staking_history_index = validator_set_histories
            .get(&era_number.0)
            .unwrap()
            .staking_history_index;
        let staking_histories = self.staking_histories.get().unwrap();
        for index in 0..staking_history_index + 1 {
            next_validator_set.apply_staking_history(&staking_histories.get(&index).unwrap());
        }
        self.next_validator_set.set(&next_validator_set);
        // Clear staking histories after the index in target `validator set of era`
        let mut staking_histories = self.staking_histories.get().unwrap();
        staking_histories.reset_to(&staking_history_index);
        self.staking_histories.set(&staking_histories);
        // Reset validator profiles
        self.reset_validator_profiles_to(era_number);
    }
    //
    fn clear_anchor_event_histories(&mut self) {
        self.assert_owner();
        let mut anchor_event_histories = self.anchor_event_histories.get().unwrap();
        anchor_event_histories.clear();
        self.anchor_event_histories.set(&anchor_event_histories);
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
        let next_validator_set = self.next_validator_set.get().unwrap();
        reward_distribution_records.clear(&next_validator_set, &era_number.0);
        self.reward_distribution_records
            .set(&reward_distribution_records);
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        if let Some(mut validator_set_of_era) = validator_set_histories.get(&era_number.0) {
            validator_set_of_era.clear_reward_distribution_records();
            validator_set_histories.insert(&era_number.0, &validator_set_of_era);
        }
    }
    //
    fn clear_unbonded_stakes(&mut self) {
        self.assert_owner();
        let validator_profiles = self.validator_profiles.get().unwrap();
        validator_profiles
            .get_validator_ids()
            .iter()
            .for_each(|validator_id| {
                self.unbonded_stakes.remove(validator_id);
            });
    }
    //
    fn clear_unwithdrawn_rewards(&mut self) {
        self.assert_owner();
        let next_validator_set = self.next_validator_set.get().unwrap();
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        let index_range = validator_set_histories.index_range();
        next_validator_set
            .validator_id_set
            .to_vec()
            .iter()
            .for_each(|validator_id| {
                for era_number in index_range.start_index.0..index_range.end_index.0 + 1 {
                    if let Some(delegator_id_set) = next_validator_set
                        .validator_id_to_delegator_id_set
                        .get(validator_id)
                    {
                        delegator_id_set.to_vec().iter().for_each(|delegator_id| {
                            self.unwithdrawn_delegator_rewards.remove(&(
                                era_number,
                                delegator_id.clone(),
                                validator_id.clone(),
                            ));
                        });
                    }
                    self.unwithdrawn_validator_rewards
                        .remove(&(era_number, validator_id.clone()));
                }
            });
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
                    if !validator_set_of_era
                        .validator_set
                        .validator_id_set
                        .contains(validator_id)
                    {
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
}
