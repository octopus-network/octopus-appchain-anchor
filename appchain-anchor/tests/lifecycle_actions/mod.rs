use appchain_anchor::AppchainAnchorContract;
use near_sdk::json_types::{U128, U64};
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn go_booting(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let outcome = call!(signer, anchor.go_booting());
    common::print_outcome_result("go_booting", &outcome);
    outcome
}
