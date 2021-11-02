use std::collections::HashMap;

use appchain_anchor::AppchainAnchorContract;
use mock_oct_token::MockOctTokenContract;
use near_sdk::serde_json::json;
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn register_validator(
    signer: &UserAccount,
    oct_token: &ContractAccount<MockOctTokenContract>,
    anchor: &ContractAccount<AppchainAnchorContract>,
    account_id_in_appchain: &Option<String>,
    amount: u128,
    can_be_delegated_to: bool,
    profile: HashMap<String, String>,
) -> ExecutionResult {
    common::ft_transfer_call_oct_token(
        signer,
        &anchor.user_account,
        amount,
        json!({
            "RegisterValidator": {
                "validator_id_in_appchain": account_id_in_appchain,
                "can_be_delegated_to": can_be_delegated_to,
                "profile": profile
            }
        })
        .to_string(),
        oct_token,
    )
}

pub fn register_delegator(
    signer: &UserAccount,
    oct_token: &ContractAccount<MockOctTokenContract>,
    anchor: &ContractAccount<AppchainAnchorContract>,
    validator_id: &String,
    amount: u128,
) -> ExecutionResult {
    common::ft_transfer_call_oct_token(
        signer,
        &anchor.user_account,
        amount,
        json!({
            "RegisterDelegator": {
                "validator_id": validator_id
            }
        })
        .to_string(),
        oct_token,
    )
}

pub fn increase_stake(
    signer: &UserAccount,
    oct_token: &ContractAccount<MockOctTokenContract>,
    anchor: &ContractAccount<AppchainAnchorContract>,
    amount: u128,
) -> ExecutionResult {
    common::ft_transfer_call_oct_token(
        signer,
        &anchor.user_account,
        amount,
        "\"IncreaseStake\"".to_string(),
        oct_token,
    )
}

pub fn increase_delegation(
    signer: &UserAccount,
    oct_token: &ContractAccount<MockOctTokenContract>,
    anchor: &ContractAccount<AppchainAnchorContract>,
    validator_id: &String,
    amount: u128,
) -> ExecutionResult {
    common::ft_transfer_call_oct_token(
        signer,
        &anchor.user_account,
        amount,
        json!({
            "IncreaseDelegation": {
                "validator_id": validator_id
            }
        })
        .to_string(),
        oct_token,
    )
}

pub fn decrease_stake(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    amount: u128,
) -> ExecutionResult {
    let result = call!(signer, anchor.decrease_stake(amount.into()));
    common::print_execution_result("decrease_stake", &result);
    result
}

pub fn unbond_stake(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let result = call!(signer, anchor.unbond_stake());
    common::print_execution_result("unbond_stake", &result);
    result
}

pub fn enable_delegation(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let result = call!(signer, anchor.enable_delegation());
    common::print_execution_result("enable_delegation", &result);
    result
}

pub fn disable_delegation(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let result = call!(signer, anchor.disable_delegation());
    common::print_execution_result("disable_delegation", &result);
    result
}

pub fn decrease_delegation(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    validator_id: &String,
    amount: u128,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.decrease_delegation(validator_id.clone(), amount.into())
    );
    common::print_execution_result("decrease_delegation", &result);
    result
}

pub fn unbond_delegation(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    validator_id: &String,
) -> ExecutionResult {
    let result = call!(signer, anchor.unbond_delegation(validator_id.clone()));
    common::print_execution_result("unbond_delegation", &result);
    result
}

pub fn withdraw_stake(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    account_id: &String,
) -> ExecutionResult {
    let result = call!(signer, anchor.withdraw_stake(account_id.clone()));
    common::print_execution_result("withdraw_stake", &result);
    result
}

pub fn withdraw_validator_rewards(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    validator_id: &String,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.withdraw_validator_rewards(validator_id.clone())
    );
    common::print_execution_result("withdraw_validator_rewards", &result);
    result
}

pub fn withdraw_delegator_rewards(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    delegator_id: &String,
    validator_id: &String,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.withdraw_delegator_rewards(delegator_id.clone(), validator_id.clone())
    );
    common::print_execution_result("withdraw_delegator_rewards", &result);
    result
}
