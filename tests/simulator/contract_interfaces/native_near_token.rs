use workspaces::{error::Error, result::ExecutionFinalResult, Account, Contract};

pub async fn deploy_near_vault_contract(
    signer: &Account,
    anchor: &Contract,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "deploy_near_vault_contract")
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn open_bridging_of_native_near_token(
    signer: &Account,
    anchor: &Contract,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "open_bridging_of_native_near_token")
        .gas(200_000_000_000_000)
        .transact()
        .await
}
