use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;

use crate::{interfaces::NearFungibleTokenManager, *};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NearFungibleTokens {
    /// The set of symbols of NEP-141 tokens.
    symbols: UnorderedSet<String>,
    /// The NEP-141 tokens data, mapped by the symbol of the token.
    tokens: LookupMap<String, NearFungibleToken>,
}

impl NearFungibleTokens {
    ///
    pub fn new() -> Self {
        Self {
            symbols: UnorderedSet::new(StorageKey::NearFungibleTokenSymbols.into_bytes()),
            tokens: LookupMap::new(StorageKey::NearFungibleTokensMap.into_bytes()),
        }
    }
    ///
    pub fn to_vec(&self) -> Vec<NearFungibleToken> {
        let symbols = self.symbols.to_vec();
        symbols
            .iter()
            .map(|symbol| self.tokens.get(symbol).unwrap())
            .collect::<Vec<NearFungibleToken>>()
    }
    ///
    pub fn contains(&self, symbol: &String) -> bool {
        self.symbols.contains(symbol)
    }
    ///
    pub fn get(&self, symbol: &String) -> Option<NearFungibleToken> {
        self.tokens.get(symbol)
    }
    ///
    pub fn get_by_contract_account(&self, account_id: &AccountId) -> Option<NearFungibleToken> {
        let symbols = self.symbols.to_vec();
        for symbol in symbols {
            let near_fungible_token = self.tokens.get(&symbol).unwrap();
            if near_fungible_token.contract_account.eq(account_id) {
                return Some(near_fungible_token);
            }
        }
        None
    }
    ///
    pub fn insert(&mut self, near_fungible_token: &NearFungibleToken) {
        self.symbols.insert(&near_fungible_token.metadata.symbol);
        self.tokens
            .insert(&near_fungible_token.metadata.symbol, near_fungible_token);
    }
    ///
    pub fn total_market_value(&self) -> Balance {
        let mut total_market_value: u128 = 0;
        let symbols = self.symbols.to_vec();
        symbols.iter().for_each(|symbol| {
            let near_fungible_token = self.tokens.get(&symbol).unwrap();
            total_market_value += near_fungible_token.locked_balance.0
                / u128::pow(10, u32::from(near_fungible_token.metadata.decimals))
                * near_fungible_token.price_in_usd.0
        });
        total_market_value
    }
    ///
    pub fn get_market_value_of(&self, symbol: &String, amount: u128) -> Balance {
        if let Some(near_fungible_token) = self.tokens.get(&symbol) {
            amount / u128::pow(10, u32::from(near_fungible_token.metadata.decimals))
                * near_fungible_token.price_in_usd.0
        } else {
            0
        }
    }
    //
    pub fn clear(&mut self) {
        for symbol in self.symbols.to_vec() {
            self.tokens.remove(&symbol);
        }
        self.symbols.clear();
    }
}

#[near_bindgen]
impl NearFungibleTokenManager for AppchainAnchor {
    //
    fn register_near_fungible_token(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        contract_account: AccountId,
        price: U128,
    ) {
        self.assert_owner();
        let mut near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        assert!(
            !near_fungible_tokens.contains(&symbol),
            "Token '{}' is already registered.",
            &symbol
        );
        assert!(
            near_fungible_tokens
                .get_by_contract_account(&contract_account)
                .is_none(),
            "Token contract '{}' is already registered.",
            contract_account
        );
        near_fungible_tokens.insert(&NearFungibleToken {
            metadata: FungibleTokenMetadata {
                spec: "ft-1.0.0".to_string(),
                symbol,
                name,
                decimals,
                icon: None,
                reference: None,
                reference_hash: None,
            },
            contract_account,
            price_in_usd: price,
            locked_balance: U128::from(0),
            bridging_state: BridgingState::Closed,
        });
        self.near_fungible_tokens.set(&near_fungible_tokens);
    }
    //
    fn change_near_fungible_token_metadata(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        contract_account: AccountId,
    ) {
        self.assert_owner();
        let mut near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        assert!(
            near_fungible_tokens.contains(&symbol),
            "Token '{}' is not registered.",
            &symbol
        );
        assert!(
            near_fungible_tokens
                .get_by_contract_account(&contract_account)
                .is_none(),
            "Token contract '{}' is already registered.",
            contract_account
        );
        let mut near_fungible_token = near_fungible_tokens.get(&symbol).unwrap();
        near_fungible_token.metadata.name = name;
        near_fungible_token.metadata.decimals = decimals;
        near_fungible_token.contract_account = contract_account;
        near_fungible_tokens.insert(&near_fungible_token);
    }
    //
    fn set_price_of_near_fungible_token(&mut self, symbol: String, price: U128) {
        self.assert_token_price_maintainer();
        let mut near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        assert!(
            near_fungible_tokens.contains(&symbol),
            "Token '{}' is not registered.",
            &symbol
        );
        let mut near_fungible_token = near_fungible_tokens.get(&symbol).unwrap();
        near_fungible_token.price_in_usd = price;
        near_fungible_tokens.insert(&near_fungible_token);
    }
    //
    fn open_bridging_of_near_fungible_token(&mut self, symbol: String) {
        self.assert_owner();
        let mut near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        assert!(
            near_fungible_tokens.contains(&symbol),
            "Token '{}' is not registered.",
            &symbol
        );
        let mut near_fungible_token = near_fungible_tokens.get(&symbol).unwrap();
        near_fungible_token.bridging_state = BridgingState::Active;
        near_fungible_tokens.insert(&near_fungible_token);
    }
    //
    fn close_bridging_of_near_fungible_token(&mut self, symbol: String) {
        self.assert_owner();
        let mut near_fungible_tokens = self.near_fungible_tokens.get().unwrap();
        assert!(
            near_fungible_tokens.contains(&symbol),
            "Token '{}' is not registered.",
            &symbol
        );
        let mut near_fungible_token = near_fungible_tokens.get(&symbol).unwrap();
        near_fungible_token.bridging_state = BridgingState::Closed;
        near_fungible_tokens.insert(&near_fungible_token);
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
enum NearFungibleTokenDepositMessage {
    BridgeToAppchain { receiver_id_in_appchain: String },
}
