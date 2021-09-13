use crate::*;

pub trait Nep141TokenManager {
    ///
    fn register_nep141_token(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        contract_account: AccountId,
        price: U64,
        price_decimals: u8,
    );
    ///
    fn set_price_of_nep141_token(&mut self, symbol: String, price: U64);
    ///
    fn open_bridging_of_nep141_token(&mut self, symbol: String);
    ///
    fn close_bridging_of_nep141_token(&mut self, symbol: String);
}

impl AppchainAnchor {
    //
    fn total_market_value_of_nep141_tokens(&self) -> Balance {
        todo!()
    }
    //
    fn lock_nep141_token(
        &mut self,
        contract_account: AccountId,
        sender_id: AccountId,
        receiver_id: AccountIdInAppchain,
        amount: U64,
    ) {
        todo!()
    }
    //
    fn unlock_nep141_token(
        &mut self,
        request_id: String,
        symbol: String,
        receiver_id: AccountId,
        amount: U64,
    ) {
        todo!()
    }
}
