use appchain_anchor::types::{MultiTxsOperationProcessingResult, ValidatorMerkleProof};
use near_sdk::serde_json::json;
use workspaces::{network::Sandbox, result::CallExecutionDetails, Account, Contract, Worker};

pub async fn process_appchain_messages(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
) -> anyhow::Result<MultiTxsOperationProcessingResult> {
    let result = signer
        .call(worker, anchor.id(), "process_appchain_messages")
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("{:?}", result);
    println!();
    result.json::<MultiTxsOperationProcessingResult>()
}

pub async fn start_updating_state_of_beefy_light_client(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    signed_commitment: Vec<u8>,
    validator_proofs: Vec<ValidatorMerkleProof>,
    mmr_leaf: Vec<u8>,
    mmr_proof: Vec<u8>,
) -> anyhow::Result<CallExecutionDetails> {
    let result = signer
        .call(
            worker,
            anchor.id(),
            "start_updating_state_of_beefy_light_client",
        )
        .args_json(json!({
            "signed_commitment": signed_commitment,
            "validator_proofs": validator_proofs,
            "mmr_leaf": mmr_leaf,
            "mmr_proof": mmr_proof
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await;
    println!("{:?}", result.as_ref().unwrap());
    result
}

pub async fn try_complete_updating_state_of_beefy_light_client(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
) -> anyhow::Result<MultiTxsOperationProcessingResult> {
    let result = signer
        .call(
            worker,
            anchor.id(),
            "try_complete_updating_state_of_beefy_light_client",
        )
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("{:?}", result);
    result.json::<MultiTxsOperationProcessingResult>()
}
