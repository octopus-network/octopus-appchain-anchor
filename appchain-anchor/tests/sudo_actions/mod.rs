use appchain_anchor::{AppchainAnchorContract, AppchainMessage};
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn apply_appchain_message(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    message: AppchainMessage,
) -> ExecutionResult {
    let result = call!(signer, anchor.apply_appchain_message(message));
    common::print_execution_result("apply_appchain_message", &result);
    result
}
