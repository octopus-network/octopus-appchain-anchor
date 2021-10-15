use appchain_anchor::AppchainAnchorContract;
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn try_complete_switching_era(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let result = call!(signer, anchor.try_complete_switching_era());
    common::print_execution_result(anchor, "try_complete_switching_era", &result);
    result
}

pub fn try_complete_distributing_reward(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let result = call!(signer, anchor.try_complete_distributing_reward());
    common::print_execution_result(anchor, "try_complete_distributing_reward", &result);
    result
}
