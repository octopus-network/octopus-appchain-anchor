use crate::{types::NearFungibleToken, AppchainAnchor};
use near_sdk::json_types::U128;

pub mod native_near_token;
pub mod near_fungible_tokens;
pub mod wrapped_appchain_nfts;
pub mod wrapped_appchain_token;

impl AppchainAnchor {
    ///
    pub fn assert_locked_asset_on_near_side(
        &self,
        near_fungible_token: Option<(&NearFungibleToken, &U128)>,
        native_near_token_amount: &U128,
    ) {
        let near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        let protocol_settings = self.protocol_settings.get().unwrap();
        if let Some(near_fungible_token) = near_fungible_token {
            assert!(
                near_fungible_tokens.total_market_value()
                    + near_fungible_tokens.get_market_value_of(
                        &near_fungible_token.0.metadata.symbol,
                        near_fungible_token.1 .0
                    )
                    <= self.get_market_value_of_staked_oct_token().0
                        * u128::from(
                            protocol_settings.maximum_market_value_percent_of_near_fungible_tokens
                        )
                        / 100,
                "Too much NEAR fungible token to lock. Return deposit."
            );
        }
        if native_near_token_amount.0 > 0 {
            let native_near_token = self.native_near_token.get().unwrap();
            assert!(
                near_fungible_tokens.total_market_value()
                    + (native_near_token.locked_balance.0 + native_near_token_amount.0)
                        * native_near_token.price_in_usd.0
                    <= self.get_market_value_of_staked_oct_token().0
                        * u128::from(
                            protocol_settings.maximum_market_value_percent_of_near_fungible_tokens
                        )
                        / 100,
                "Too much native NEAR token to lock."
            );
        }
    }
}
