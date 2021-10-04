use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NearFungibleTokens {
    /// The set of symbols of NEP-141 tokens.
    pub symbols: UnorderedSet<String>,
    /// The NEP-141 tokens data, mapped by the symbol of the token.
    pub tokens: LookupMap<String, NearFungibleToken>,
}

impl NearFungibleTokens {
    ///
    pub fn new() -> Self {
        Self {
            symbols: UnorderedSet::new(StorageKey::NearFungibleTokenSymbols.into_bytes()),
            tokens: LookupMap::new(StorageKey::NearFungibleTokensMap.into_bytes()),
        }
    }
}

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

#[near_bindgen]
impl NearFungibleTokenManager for AppchainAnchor {
    fn register_near_fungible_token(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        contract_account: AccountId,
        price: U64,
        price_decimals: u8,
    ) {
        todo!()
    }

    fn set_price_of_near_fungible_token(&mut self, symbol: String, price: U64) {
        todo!()
    }

    fn open_bridging_of_near_fungible_token(&mut self, symbol: String) {
        todo!()
    }

    fn close_bridging_of_near_fungible_token(&mut self, symbol: String) {
        todo!()
    }
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
