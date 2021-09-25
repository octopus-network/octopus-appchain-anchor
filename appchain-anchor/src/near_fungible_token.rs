use crate::*;

pub trait NearFungibleTokenManager {
    ///
    fn register_near_fungible_token(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        contract_account: AccountId,
        price: U64,
        price_decimals: u8,
    );
    ///
    fn set_price_of_near_fungible_token(&mut self, symbol: String, price: U64);
    ///
    fn open_bridging_of_near_fungible_token(&mut self, symbol: String);
    ///
    fn close_bridging_of_near_fungible_token(&mut self, symbol: String);
}

impl AppchainAnchor {
    //
    fn total_market_value_of_near_fungible_tokens(&self) -> Balance {
        todo!()
    }
    //
    fn lock_near_fungible_token(
        &mut self,
        contract_account: AccountId,
        sender_id: AccountId,
        receiver_id: AccountIdInAppchain,
        amount: U64,
    ) {
        todo!()
    }
    //
    fn unlock_near_fungible_token(
        &mut self,
        request_id: String,
        symbol: String,
        receiver_id: AccountId,
        amount: U64,
    ) {
        todo!()
    }
}
