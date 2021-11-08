use appchain_anchor::AppchainAnchorContract;
use near_sdk::json_types::{U128, U64};
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn change_minimum_validator_count(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    value: u64,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.change_minimum_validator_count(U64::from(value))
    );
    common::print_execution_result("change_minimum_validator_count", &result);
    result
}

pub fn set_rpc_endpoint(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    value: String,
) -> ExecutionResult {
    let result = call!(signer, anchor.set_rpc_endpoint(value));
    common::print_execution_result("set_rpc_endpoint", &result);
    result
}

pub fn set_subql_endpoint(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    value: String,
) -> ExecutionResult {
    let result = call!(signer, anchor.set_subql_endpoint(value));
    common::print_execution_result("set_subql_endpoint", &result);
    result
}

pub fn set_era_reward(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    value: u128,
) -> ExecutionResult {
    let result = call!(signer, anchor.set_era_reward(U128::from(value)));
    common::print_execution_result("set_era_reward", &result);
    result
}

pub fn set_token_price_maintainer_account(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    operator: &UserAccount,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.set_token_price_maintainer_account(operator.valid_account_id().to_string())
    );
    common::print_execution_result("set_token_price_maintainer_account", &result);
    result
}

pub fn set_price_of_oct_token(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    value: u128,
) -> ExecutionResult {
    let result = call!(signer, anchor.set_price_of_oct_token(U128::from(value)));
    common::print_execution_result("set_price_of_oct_token", &result);
    result
}

pub fn change_minimum_total_stake_price_for_booting(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    value: u128,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.change_minimum_total_stake_price_for_booting(U128::from(value))
    );
    common::print_execution_result("change_minimum_total_stake_price_for_booting", &result);
    result
}

pub fn change_unlock_period_of_validator_deposit(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    value: u64,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.change_unlock_period_of_validator_deposit(U64::from(value))
    );
    common::print_execution_result("change_unlock_period_of_validator_deposit", &result);
    result
}

pub fn change_unlock_period_of_delegator_deposit(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    value: u64,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.change_unlock_period_of_delegator_deposit(U64::from(value))
    );
    common::print_execution_result("change_unlock_period_of_delegator_deposit", &result);
    result
}
