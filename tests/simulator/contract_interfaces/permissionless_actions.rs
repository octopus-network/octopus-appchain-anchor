use appchain_anchor::types::{MultiTxsOperationProcessingResult, ValidatorMerkleProof};
use near_sdk::serde_json::json;
use workspaces::{network::Sandbox, Account, Contract, Worker};

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

pub async fn process_appchain_messages_with_all_proofs(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    signed_commitment: Vec<u8>,
    validator_proofs: Vec<ValidatorMerkleProof>,
    mmr_leaf_for_mmr_root: Vec<u8>,
    mmr_proof_for_mmr_root: Vec<u8>,
    encoded_messages: Vec<u8>,
    header: Vec<u8>,
    mmr_leaf_for_header: Vec<u8>,
    mmr_proof_for_header: Vec<u8>,
) -> anyhow::Result<MultiTxsOperationProcessingResult> {
    let result = signer
        .call(
            worker,
            anchor.id(),
            "process_appchain_messages_with_all_proofs",
        )
        .args_json(json!({
            "signed_commitment": signed_commitment,
            "validator_proofs": validator_proofs,
            "mmr_leaf_for_mmr_root": mmr_leaf_for_mmr_root,
            "mmr_proof_for_mmr_root": mmr_proof_for_mmr_root,
            "encoded_messages": encoded_messages,
            "header": header,
            "mmr_leaf_for_header": mmr_leaf_for_header,
            "mmr_proof_for_header": mmr_proof_for_header,
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("{:?}", result);
    println!();
    result.json::<MultiTxsOperationProcessingResult>()
}
