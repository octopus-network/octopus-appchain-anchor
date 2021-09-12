use crate::*;

/// The bridging state of NEP-141 token.
pub enum BridgingState {
    /// The state which this contract is bridging the bridge token to the appchain.
    Active,
    /// The state which this contract has stopped bridging the bridge token to the appchain.
    Closed,
}

pub struct Nep141TokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
}

pub struct Nep141Token {
    pub metadata: Nep141TokenMetadata,
    pub contract_account: AccountId,
    pub price: U64,
    pub price_decimals: u8,
    /// The total balance locked in this contract
    pub locked_balance: Balance,
    pub bridging_state: BridgingState,
}

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
