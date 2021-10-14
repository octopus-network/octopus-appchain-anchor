use appchain_anchor::AppchainAnchorContract;
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn go_booting(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let result = call!(signer, anchor.go_booting());
    common::print_execution_result("go_booting", &result);
    result
}

pub fn go_live(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let result = call!(signer, anchor.go_live());
    common::print_execution_result("go_live", &result);
    result
}
