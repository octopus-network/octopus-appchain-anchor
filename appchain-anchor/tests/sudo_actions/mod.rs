use appchain_anchor::{AppchainAnchorContract, AppchainMessage};
use near_sdk::json_types::{U128, U64};
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn apply_appchain_message(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    message: AppchainMessage,
) -> ExecutionResult {
    let outcome = call!(signer, anchor.apply_appchain_message(message));
    common::print_outcome_result("apply_appchain_message", &outcome);
    outcome
}
