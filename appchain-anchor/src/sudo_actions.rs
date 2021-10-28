use crate::message_decoder::AppchainMessage;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;

use crate::*;

pub trait SudoActions {
    /// Apply a certain `AppchainMessage`
    fn apply_appchain_message(&mut self, appchain_message: AppchainMessage);
    ///
    fn set_metadata_of_wrapped_appchain_token(&mut self, metadata: FungibleTokenMetadata);
    ///
    fn set_premined_balance_of_wrapped_appchain_token(
        &mut self,
        premined_beneficiary: AccountId,
        value: U128,
    );
}

#[near_bindgen]
impl SudoActions for AppchainAnchor {
    //
    fn apply_appchain_message(&mut self, appchain_message: AppchainMessage) {
        self.assert_owner();
        self.internal_apply_appchain_message(appchain_message);
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
}
