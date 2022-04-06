use std::collections::HashMap;

use appchain_anchor::AppchainAnchorContract;
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn set_validator_id_in_appchain(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    validator_id_in_appchain: &String,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.set_validator_id_in_appchain(validator_id_in_appchain.clone())
    );
    common::print_execution_result("set_validator_id_in_appchain", &result);
    result
}

pub fn set_validator_profile(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    profile: &HashMap<String, String>,
) -> ExecutionResult {
    let result = call!(signer, anchor.set_validator_profile(profile.clone()));
    common::print_execution_result("set_validator_profile", &result);
    result
}
