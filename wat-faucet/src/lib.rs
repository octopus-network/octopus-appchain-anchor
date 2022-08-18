use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::U128,
    near_bindgen, AccountId, Gas, PanicOnDefault, Promise,
};
use std::{ops::Mul, str::FromStr};

/// Constants for gas.
const T_GAS_FOR_BURN_WRAPPED_APPCHAIN_TOKEN: u64 = 35;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct WrappedAppchainTokenFaucet {
    appchain_anchor_account: AccountId,
}

#[near_bindgen]
impl WrappedAppchainTokenFaucet {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "The contract is already initialized.");
        let account_id = String::from(env::current_account_id().as_str());
        let (_first, second) = account_id
            .split_once(".")
            .expect("This contract must be deployed as a sub-account.");
        Self {
            appchain_anchor_account: AccountId::from_str(second).unwrap(),
        }
    }
    // Assert that the contract called by the owner.
    fn assert_anchor(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.appchain_anchor_account,
            "Function can only be called by corresponding appchain anchor."
        );
    }
    ///
    pub fn burn_wrapped_appchain_token(&mut self, receiver_id: String, amount: U128) {
        self.assert_anchor();
        //
        #[derive(near_sdk::serde::Serialize)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            receiver_id: String,
            amount: U128,
        }
        let args = Input {
            receiver_id,
            amount,
        };
        let args = near_sdk::serde_json::to_vec(&args)
            .expect("Failed to serialize the cross contract args using JSON.");
        Promise::new(self.appchain_anchor_account.clone()).function_call(
            "burn_wrapped_appchain_token".to_string(),
            args,
            0,
            Gas::ONE_TERA.mul(T_GAS_FOR_BURN_WRAPPED_APPCHAIN_TOKEN),
        );
    }
}
