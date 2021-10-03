use near_sdk::json_types::I128;

use crate::*;

impl Default for WrappedAppchainTokenMetadata {
    fn default() -> Self {
        Self {
            symbol: String::new(),
            name: String::new(),
            decimals: 0,
            spec: String::new(),
            icon: Option::None,
            reference: Option::None,
            reference_hash: Option::None,
        }
    }
}

impl Default for WrappedAppchainToken {
    fn default() -> Self {
        Self {
            metadata: WrappedAppchainTokenMetadata::default(),
            contract_account: AccountId::new(),
            initial_balance: 0,
            changed_balance: I128::from(0),
            price_in_usd: U64::from(0),
        }
    }
}

pub trait WrappedAppchainTokenManager {
    ///
    fn set_metadata_of_wrapped_appchain_token(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        spec: String,
        icon: Option<Vec<u8>>,
        reference: Option<Vec<u8>>,
        reference_hash: Option<Vec<u8>>,
    );
    ///
    fn set_initial_balance_of_wrapped_appchain_token(&mut self, value: Balance);
    ///
    fn set_account_of_wrapped_appchain_token(&mut self, contract_account: AccountId);
    ///
    fn set_price_of_wrapped_appchain_token(&mut self, price: U64);
    ///
    fn burn_wrapped_appchain_token(&mut self, receiver_id: AccountIdInAppchain, amount: U64);
}

impl AppchainAnchor {
    //
    fn mint_wrapped_appchain_token(
        &mut self,
        request_id: String,
        receiver_id: AccountId,
        amount: U64,
    ) {
        todo!()
    }
}
