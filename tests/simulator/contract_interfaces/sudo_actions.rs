use appchain_anchor::{types::AppchainMessageProcessingResult, AppchainMessage};
use near_sdk::{
    json_types::U64,
    serde_json::{self, json},
};
use workspaces::{network::Sandbox, result::CallExecutionDetails, Account, Contract, Worker};

pub async fn stage_appchain_messages(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    messages: Vec<AppchainMessage>,
) -> anyhow::Result<CallExecutionDetails> {
    messages.iter().for_each(|message| {
        println!(
            "Appchain message: {}",
            serde_json::to_string::<AppchainMessage>(&message).unwrap()
        );
        println!();
    });
    signer
        .call(worker, anchor.id(), "stage_appchain_messages")
        .args_json(json!({ "messages": messages }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn apply_appchain_message(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    nonce: u32,
) -> anyhow::Result<Option<AppchainMessageProcessingResult>> {
    let result = signer
        .call(worker, anchor.id(), "apply_appchain_message")
        .args_json(json!({ "nonce": nonce }))?
        .gas(200_000_000_000_000)
        .transact()
        .await?;
    println!("{:?}", result);
    println!();
    result.json::<Option<AppchainMessageProcessingResult>>()
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

pub async fn clear_anchor_event_histories(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "clear_anchor_event_histories")
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

pub async fn initialize_beefy_light_client(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    initial_public_keys: Vec<String>,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "initialize_beefy_light_client")
        .args_json(json!({ "initial_public_keys": initial_public_keys }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}
