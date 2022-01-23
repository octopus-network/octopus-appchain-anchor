use appchain_anchor::{
    types::{MultiTxsOperationProcessingResult, ValidatorMerkleProof},
    AppchainAnchorContract,
};
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn process_appchain_messages(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> MultiTxsOperationProcessingResult {
    let result = call!(signer, anchor.process_appchain_messages());
    common::print_execution_result("process_appchain_messages", &result);
    assert!(result.is_ok());
    result.unwrap_json::<MultiTxsOperationProcessingResult>()
}

pub fn start_updating_state_of_beefy_light_client(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    signed_commitment: Vec<u8>,
    validator_proofs: Vec<ValidatorMerkleProof>,
    mmr_leaf: Vec<u8>,
    mmr_proof: Vec<u8>,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.start_updating_state_of_beefy_light_client(
            signed_commitment,
            validator_proofs,
            mmr_leaf,
            mmr_proof
        )
    );
    common::print_execution_result("start_updating_state_of_beefy_light_client", &result);
    result
}

pub fn try_complete_updating_state_of_beefy_light_client(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) -> MultiTxsOperationProcessingResult {
    let result = call!(
        signer,
        anchor.try_complete_updating_state_of_beefy_light_client()
    );
    common::print_execution_result("try_complete_updating_state_of_beefy_light_client", &result);
    if !result.is_ok() {
        println!("{:#?}", result);
    }
    assert!(result.is_ok());
    result.unwrap_json::<MultiTxsOperationProcessingResult>()
}

pub fn verify_and_stage_appchain_messages(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    encoded_messages: Vec<u8>,
    header: Vec<u8>,
    mmr_leaf: Vec<u8>,
    mmr_proof: Vec<u8>,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.verify_and_stage_appchain_messages(encoded_messages, header, mmr_leaf, mmr_proof)
    );
    common::print_execution_result("verify_and_apply_appchain_messages", &result);
    result
}
