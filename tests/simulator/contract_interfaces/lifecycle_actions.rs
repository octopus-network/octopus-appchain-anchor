use near_sdk::serde_json::json;
use workspaces::{network::Sandbox, result::CallExecutionDetails, Account, Contract, Worker};

pub async fn go_booting(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "go_booting")
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn go_live(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "go_live")
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
