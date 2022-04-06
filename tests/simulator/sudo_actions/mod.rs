use appchain_anchor::{AppchainAnchorContract, AppchainMessage};
use near_sdk::{json_types::U64, serde_json};
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn stage_appchain_messages(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    messages: Vec<AppchainMessage>,
) {
    messages.iter().for_each(|message| {
        println!(
            "Appchain message: {}",
            serde_json::to_string::<AppchainMessage>(&message).unwrap()
        );
    });
    let result = call!(signer, anchor.stage_appchain_messages(messages));
    common::print_execution_result("stage_appchain_messages", &result);
    assert!(result.is_ok());
    common::print_appchain_messages(anchor);
    common::print_anchor_status(anchor);
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

pub fn reset_validator_set_histories_to(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: U64,
) -> ExecutionResult {
    let result = call!(signer, anchor.reset_validator_set_histories_to(era_number));
    common::print_execution_result("reset_validator_set_histories_to", &result);
    result
}

pub fn clear_anchor_event_histories(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let result = call!(signer, anchor.clear_anchor_event_histories());
    common::print_execution_result("clear_anchor_event_histories", &result);
    result
}

pub fn clear_appchain_notification_histories(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let result = call!(signer, anchor.clear_appchain_notification_histories());
    common::print_execution_result("clear_appchain_notification_histories", &result);
    result
}

pub fn clear_reward_distribution_records(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: U64,
) -> ExecutionResult {
    let result = call!(signer, anchor.clear_reward_distribution_records(era_number));
    common::print_execution_result("clear_reward_distribution_records", &result);
    result
}

pub fn clear_unbonded_stakes(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let result = call!(signer, anchor.clear_unbonded_stakes());
    common::print_execution_result("clear_unbonded_stakes", &result);
    result
}

pub fn clear_unwithdrawn_rewards(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: U64,
) -> ExecutionResult {
    let result = call!(signer, anchor.clear_unwithdrawn_rewards(era_number));
    common::print_execution_result("clear_unwithdrawn_rewards", &result);
    result
}

pub fn pause_asset_transfer(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let outcome = call!(signer, anchor.pause_asset_transfer());
    common::print_execution_result("pause_asset_transfer", &outcome);
    outcome
}

pub fn resume_asset_transfer(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> ExecutionResult {
    let outcome = call!(signer, anchor.resume_asset_transfer());
    common::print_execution_result("resume_asset_transfer", &outcome);
    outcome
}
