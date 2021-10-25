use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{
    assert_one_yocto, env, ext_contract, near_bindgen, AccountId, PanicOnDefault, PromiseOrValue,
};

near_sdk::setup_alloc!();

#[ext_contract(ext_appchain_anchor)]
trait AppchainAnchor {
    fn sync_basedata_of_wrapped_appchain_token(
        &mut self,
        metadata: FungibleTokenMetadata,
        premined_beneficiary: AccountId,
        premined_balance: U128,
    );
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct MockWrappedAppchainToken {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    owner_id: AccountId,
}

#[near_bindgen]
impl MockWrappedAppchainToken {
    #[init]
    pub fn new(
        owner_id: ValidAccountId,
        premined_beneficiary: ValidAccountId,
        premined_balance: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized.");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            owner_id: owner_id.clone().into(),
        };
        this.token
            .internal_register_account(premined_beneficiary.as_ref());
        this.token
            .internal_deposit(premined_beneficiary.as_ref(), premined_balance.into());
        ext_appchain_anchor::sync_basedata_of_wrapped_appchain_token(
            metadata,
            premined_beneficiary.to_string(),
            premined_balance,
            &owner_id,
            0,
            80_000_000_000_000,
        );
        this
    }

    #[payable]
    pub fn mint(&mut self, account_id: ValidAccountId, amount: U128) {
        self.assert_owner();
        self.storage_deposit(Some(account_id.clone()), None);
        self.token
            .internal_deposit(account_id.as_ref(), amount.into());
    }

    #[payable]
    pub fn burn(&mut self, account_id: ValidAccountId, amount: U128) {
        assert_one_yocto();
        self.assert_owner();
        self.token
            .internal_withdraw(account_id.as_ref(), amount.into());
    }
}

pub trait Ownable {
    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.get_owner(),
            "Only owner can call mint."
        );
    }
    fn get_owner(&self) -> AccountId;
    fn set_owner(&mut self, owner: AccountId);
}

#[near_bindgen]
impl Ownable for MockWrappedAppchainToken {
    fn get_owner(&self) -> AccountId {
        self.owner_id.clone()
    }

    fn set_owner(&mut self, owner_id: AccountId) {
        self.assert_owner();
        self.owner_id = owner_id;
    }
}

near_contract_standards::impl_fungible_token_core!(MockWrappedAppchainToken, token);
near_contract_standards::impl_fungible_token_storage!(MockWrappedAppchainToken, token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for MockWrappedAppchainToken {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
