use crate::message_decoder::AppchainMessage;
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
    fn reset_validator_set_histories(&mut self);
    ///
    fn reset_staking_histories(&mut self);
    ///
    fn reset_anchor_event_histories(&mut self);
    ///
    fn reset_appchain_notification_histories(&mut self);
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
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        validator_set_histories.remove(&era_number.0);
        self.validator_set_histories.set(&validator_set_histories);
    }
    //
    fn reset_validator_set_histories(&mut self) {
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        validator_set_histories.reset();
        self.validator_set_histories.set(&validator_set_histories);
    }
    //
    fn reset_staking_histories(&mut self) {
        let mut staking_histories = self.staking_histories.get().unwrap();
        staking_histories.reset();
        self.staking_histories.set(&staking_histories);
    }
    //
    fn reset_anchor_event_histories(&mut self) {
        let mut anchor_event_histories = self.anchor_event_histories.get().unwrap();
        anchor_event_histories.reset();
        self.anchor_event_histories.set(&anchor_event_histories);
    }
    //
    fn reset_appchain_notification_histories(&mut self) {
        let mut appchain_notification_histories =
            self.appchain_notification_histories.get().unwrap();
        appchain_notification_histories.reset();
        self.appchain_notification_histories
            .set(&appchain_notification_histories);
    }
}
