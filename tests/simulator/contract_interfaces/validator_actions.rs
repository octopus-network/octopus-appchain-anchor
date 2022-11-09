use near_sdk::serde_json::json;
use std::collections::HashMap;
use workspaces::{error::Error, result::ExecutionFinalResult, Account, Contract};

pub async fn set_validator_id_in_appchain(
    signer: &Account,
    anchor: &Contract,
    validator_id_in_appchain: &String,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "set_validator_id_in_appchain")
        .args_json(json!({
            "account_id_in_appchain": validator_id_in_appchain
        }))
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn set_validator_profile(
    signer: &Account,
    anchor: &Contract,
    profile: &HashMap<String, String>,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "set_validator_profile")
        .args_json(json!({ "profile": profile }))
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn enable_delegation(
    signer: &Account,
    anchor: &Contract,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "enable_delegation")
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn disable_delegation(
    signer: &Account,
    anchor: &Contract,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "disable_delegation")
        .gas(200_000_000_000_000)
        .transact()
        .await
}
