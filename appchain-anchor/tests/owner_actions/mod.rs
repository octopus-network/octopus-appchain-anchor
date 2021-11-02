use appchain_anchor::{AppchainAnchorContract, AppchainMessage};
use near_sdk::json_types::U64;
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
