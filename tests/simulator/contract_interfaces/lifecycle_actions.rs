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
