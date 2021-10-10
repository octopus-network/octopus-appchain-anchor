use appchain_anchor::AppchainAnchorContract;
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn try_complete_switching_era(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let outcome = call!(signer, anchor.try_complete_switching_era());
    common::print_outcome_result("try_complete_switching_era", &outcome);
    outcome
}

pub fn try_complete_distributing_reward(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let outcome = call!(signer, anchor.try_complete_distributing_reward());
    common::print_outcome_result("try_complete_distributing_reward", &outcome);
    outcome
}
