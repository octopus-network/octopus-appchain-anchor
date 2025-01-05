use core::convert::TryFrom;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::json_types::I128;

use crate::*;

impl Default for WrappedAppchainToken {
    fn default() -> Self {
        Self {
            metadata: FungibleTokenMetadata {
                spec: "ft-1.0.0".to_string(),
                symbol: String::new(),
                name: String::new(),
                decimals: 0,
                icon: None,
                reference: None,
                reference_hash: None,
            },
            contract_account: None,
            premined_beneficiary: None,
            premined_balance: U128::from(0),
            changed_balance: I128::from(0),
            price_in_usd: U128::from(0),
            total_supply: U128::from(0),
        }
    }
}

impl WrappedAppchainToken {
    ///
    pub fn total_market_value(&self) -> Balance {
        let total_balance: i128 =
            i128::try_from(self.premined_balance.0).unwrap() + self.changed_balance.0;
        u128::try_from(total_balance).unwrap() / u128::pow(10, u32::from(self.metadata.decimals))
            * self.price_in_usd.0
    }
    ///
    pub fn get_market_value_of(&self, amount: u128) -> Balance {
        amount / u128::pow(10, u32::from(self.metadata.decimals)) * self.price_in_usd.0
    }
}
