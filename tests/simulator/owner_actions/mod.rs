use appchain_anchor::AppchainAnchorContract;
use near_sdk::{
    json_types::{U128, U64},
    AccountId,
};
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn remove_staking_history_before(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: U64,
) -> ExecutionResult {
    let result = call!(signer, anchor.remove_staking_history_before(era_number));
    common::print_execution_result("remove_staking_history_before", &result);
    result
}

pub fn remove_anchor_event_history_before(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: U64,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.remove_anchor_event_history_before(era_number)
    );
    common::print_execution_result("remove_anchor_event_history_before", &result);
    result
}

pub fn register_near_fungible_token(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    symbol: String,
    name: String,
    decimals: u8,
    contract_account: AccountId,
    price: U128,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.register_near_fungible_token(symbol, name, decimals, contract_account, price)
    );
    common::print_execution_result("remove_anchor_event_history_before", &result);
    result
}
