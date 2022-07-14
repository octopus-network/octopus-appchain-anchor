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
