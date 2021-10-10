use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct MockAppchainRegistry {
    owner: AccountId,
    oct_token: AccountId,
}

#[near_bindgen]
impl MockAppchainRegistry {
    #[init]
    pub fn new(oct_token: AccountId) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        Self {
            owner: env::signer_account_id(),
            oct_token,
        }
    }
}
