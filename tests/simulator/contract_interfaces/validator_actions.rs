use near_sdk::serde_json::json;
use std::collections::HashMap;
use workspaces::{network::Sandbox, result::CallExecutionDetails, Account, Contract, Worker};

pub async fn set_validator_id_in_appchain(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    validator_id_in_appchain: &String,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "set_validator_id_in_appchain")
        .args_json(json!({
            "account_id_in_appchain": validator_id_in_appchain
        }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn set_validator_profile(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    profile: &HashMap<String, String>,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "set_validator_profile")
        .args_json(json!({ "profile": profile }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}
