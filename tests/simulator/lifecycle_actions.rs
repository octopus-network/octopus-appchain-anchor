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

pub fn initialize_beefy_light_client(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    initial_public_keys: Vec<String>,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.initialize_beefy_light_client(initial_public_keys)
    );
    common::print_execution_result("initialize_beefy_light_client", &result);
    result
}
