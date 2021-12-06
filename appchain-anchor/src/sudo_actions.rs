use crate::{message_decoder::AppchainMessage, validator_set::ValidatorSetActions};
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;

use crate::*;

pub trait SudoActions {
    /// Apply a certain `AppchainMessage`
    fn apply_appchain_messages(
        &mut self,
        appchain_messages: Vec<AppchainMessage>,
    ) -> Vec<AppchainMessageProcessingResult>;
    ///
    fn set_metadata_of_wrapped_appchain_token(&mut self, metadata: FungibleTokenMetadata);
    ///
    fn set_premined_balance_of_wrapped_appchain_token(
        &mut self,
        premined_beneficiary: AccountId,
        value: U128,
    );
    ///
    fn remove_validator_set_of(&mut self, era_number: U64);
    ///
    fn reset_validator_set_histories_to(&mut self, era_number: U64);
    ///
    fn reset_staking_histories(&mut self);
    ///
    fn reset_anchor_event_histories(&mut self);
    ///
    fn reset_appchain_notification_histories(&mut self);
    ///
    fn reset_beefy_light_client(&mut self, initial_public_keys: Vec<String>);
    ///
    fn clear_reward_distribution_records(&mut self);
    ///
    fn clear_unbonded_stakes(&mut self);
    ///
    fn clear_unwithdrawn_rewards(&mut self);
}

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
    fn remove_validator_set_of(&mut self, era_number: U64) {
        self.assert_owner();
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        validator_set_histories.remove(&era_number.0);
        self.validator_set_histories.set(&validator_set_histories);
    }
    //
    fn reset_validator_set_histories_to(&mut self, era_number: U64) {
        self.assert_owner();
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        validator_set_histories.reset_to(&era_number.0);
        self.validator_set_histories.set(&validator_set_histories);
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
    }
    //
    fn reset_staking_histories(&mut self) {
        self.assert_owner();
        let mut staking_histories = self.staking_histories.get().unwrap();
        staking_histories.reset();
        self.staking_histories.set(&staking_histories);
    }
    //
    fn reset_anchor_event_histories(&mut self) {
        self.assert_owner();
        let mut anchor_event_histories = self.anchor_event_histories.get().unwrap();
        anchor_event_histories.reset();
        self.anchor_event_histories.set(&anchor_event_histories);
    }
    //
    fn reset_appchain_notification_histories(&mut self) {
        self.assert_owner();
        let mut appchain_notification_histories =
            self.appchain_notification_histories.get().unwrap();
        appchain_notification_histories.reset();
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
    fn clear_reward_distribution_records(&mut self) {
        let mut reward_distribution_records = self.reward_distribution_records.get().unwrap();
        let next_validator_set = self.next_validator_set.get().unwrap();
        reward_distribution_records.clear(&next_validator_set);
        self.reward_distribution_records
            .set(&reward_distribution_records);
    }
    //
    fn clear_unbonded_stakes(&mut self) {
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
}
