use near_sdk::serde_json::json;
use workspaces::{error::Error, result::ExecutionFinalResult, Account, Contract};

pub async fn generate_initial_validator_set(
    signer: &Account,
    anchor: &Contract,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "generate_initial_validator_set")
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn go_live(signer: &Account, anchor: &Contract) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "go_live")
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn initialize_beefy_light_client(
    signer: &Account,
    anchor: &Contract,
    initial_public_keys: Vec<String>,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "initialize_beefy_light_client")
        .args_json(json!({ "initial_public_keys": initial_public_keys }))
        .gas(200_000_000_000_000)
        .transact()
        .await
}
