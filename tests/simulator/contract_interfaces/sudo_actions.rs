use appchain_anchor::AppchainMessage;
use near_sdk::{
    json_types::U64,
    serde_json::{self, json},
};
use workspaces::{network::Sandbox, result::CallExecutionDetails, Account, Contract, Worker};

pub async fn stage_appchain_message(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    message: AppchainMessage,
) -> anyhow::Result<CallExecutionDetails> {
    println!(
        "Appchain message: {}",
        serde_json::to_string::<AppchainMessage>(&message).unwrap()
    );
    println!();
    signer
        .call(worker, anchor.id(), "stage_appchain_message")
        .args_json(json!({ "appchain_message": message }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn reset_validator_set_histories_to(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    era_number: U64,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "reset_validator_set_histories_to")
        .args_json(json!({ "era_number": era_number }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn clear_appchain_notification_histories(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "clear_appchain_notification_histories")
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn clear_reward_distribution_records(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    era_number: U64,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "clear_reward_distribution_records")
        .args_json(json!({ "era_number": era_number }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn clear_unbonded_stakes(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "clear_unbonded_stakes")
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn clear_unwithdrawn_rewards(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    era_number: U64,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "clear_unwithdrawn_rewards")
        .args_json(json!({ "era_number": era_number }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}
