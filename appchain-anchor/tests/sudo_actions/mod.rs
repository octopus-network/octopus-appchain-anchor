use appchain_anchor::{AppchainAnchorContract, AppchainMessage};
use near_sdk::{json_types::U64, serde_json};
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn apply_appchain_messages(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    messages: Vec<AppchainMessage>,
) -> ExecutionResult {
    messages.iter().for_each(|message| {
        println!(
            "Appchain message: {}",
            serde_json::to_string::<AppchainMessage>(&message).unwrap()
        );
    });
    let result = call!(signer, anchor.apply_appchain_messages(messages));
    common::print_execution_result("apply_appchain_messages", &result);
    result
}

pub fn remove_validator_set_before(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: U64,
) -> ExecutionResult {
    let result = call!(signer, anchor.remove_validator_set_before(era_number));
    common::print_execution_result("remove_validator_set_before", &result);
    result
}

pub fn reset_validator_set_histories(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let result = call!(signer, anchor.reset_validator_set_histories());
    common::print_execution_result("reset_validator_set_histories", &result);
    result
}

pub fn reset_staking_histories(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let result = call!(signer, anchor.reset_staking_histories());
    common::print_execution_result("reset_staking_histories", &result);
    result
}

pub fn reset_anchor_event_histories(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let result = call!(signer, anchor.reset_anchor_event_histories());
    common::print_execution_result("reset_anchor_event_histories", &result);
    result
}
