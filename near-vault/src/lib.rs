use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, AccountId, Gas, PanicOnDefault, Promise};
use std::ops::Mul;
use std::str::FromStr;

/// Constants for gas.
const T_GAS_FOR_GENERATE_APPCHAIN_NOTIFICATION: u64 = 50;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct NativeNearTokenReceiver {
    /// The account id of corresponding appchain anchor contract
    appchain_anchor_account: AccountId,
}

#[near_bindgen]
impl NativeNearTokenReceiver {
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
    #[payable]
    pub fn deposit_near_for_appchain_user(
        &mut self,
        receiver_id_in_appchain: String,
        near_amount: U128,
    ) {
        assert!(
            env::attached_deposit() == near_amount.0,
            "Attached deposit is not equal to the requested amount."
        );
        //
        #[derive(near_sdk::serde::Serialize)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            sender_id_in_near: AccountId,
            receiver_id_in_appchain: String,
            amount: U128,
        }
        let args = Input {
            sender_id_in_near: env::predecessor_account_id(),
            receiver_id_in_appchain,
            amount: near_amount,
        };
        let args = near_sdk::serde_json::to_vec(&args)
            .expect("Failed to serialize the cross contract args using JSON.");
        Promise::new(self.appchain_anchor_account.clone()).function_call(
            "generate_appchain_notification_for_near_deposit".to_string(),
            args,
            0,
            Gas::ONE_TERA.mul(T_GAS_FOR_GENERATE_APPCHAIN_NOTIFICATION),
        );
    }
    ///
    pub fn unlock_near(&mut self, receiver_id: AccountId, amount: U128) {
        self.assert_anchor();
        assert!(
            env::account_balance() - env::account_locked_balance() > amount.0,
            "Available balance is not enough."
        );
        Promise::new(receiver_id).transfer(amount.0);
    }
}
