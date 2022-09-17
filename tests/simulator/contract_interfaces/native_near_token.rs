use workspaces::{network::Sandbox, result::CallExecutionDetails, Account, Contract, Worker};

pub async fn deploy_near_vault_contract(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "deploy_near_vault_contract")
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn open_bridging_of_native_near_token(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "open_bridging_of_native_near_token")
        .gas(200_000_000_000_000)
        .transact()
        .await
}
