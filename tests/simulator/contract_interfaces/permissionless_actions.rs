use appchain_anchor::types::{MultiTxsOperationProcessingResult, ValidatorMerkleProof};
use near_sdk::serde_json::json;
use workspaces::{error::Error, result::ExecutionFinalResult, Account, Contract};

pub async fn verify_and_stage_appchain_messages(
    signer: &Account,
    anchor: &Contract,
    encoded_messages: Vec<u8>,
    header: Vec<u8>,
    mmr_leaf: Vec<u8>,
    mmr_proof: Vec<u8>,
) -> Result<ExecutionFinalResult, Error> {
    let result = signer
        .call(anchor.id(), "verify_and_stage_appchain_messages")
        .gas(300_000_000_000_000)
        .args_json(json!({
            "encoded_messages": encoded_messages,
            "header": header,
            "mmr_leaf": mmr_leaf,
            "mmr_proof": mmr_proof
        }))
        .transact()
        .await;
    println!("{:?}", result);
    println!();
    result
}

pub async fn process_appchain_messages(
    signer: &Account,
    anchor: &Contract,
) -> Result<MultiTxsOperationProcessingResult, Error> {
    let result = signer
        .call(anchor.id(), "process_appchain_messages")
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("{:?}", result);
    println!();
    result.json::<MultiTxsOperationProcessingResult>()
}

pub async fn process_appchain_messages_with_all_proofs(
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
) -> Result<MultiTxsOperationProcessingResult, Error> {
    let result = signer
        .call(anchor.id(), "process_appchain_messages_with_all_proofs")
        .args_json(json!({
            "signed_commitment": signed_commitment,
            "validator_proofs": validator_proofs,
            "mmr_leaf_for_mmr_root": mmr_leaf_for_mmr_root,
            "mmr_proof_for_mmr_root": mmr_proof_for_mmr_root,
            "encoded_messages": encoded_messages,
            "header": header,
            "mmr_leaf_for_header": mmr_leaf_for_header,
            "mmr_proof_for_header": mmr_proof_for_header,
        }))
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("{:?}", result);
    println!();
    result.json::<MultiTxsOperationProcessingResult>()
}
