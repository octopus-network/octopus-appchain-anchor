use crate::{common, contract_interfaces::permissionless_actions};
use appchain_anchor::types::{MultiTxsOperationProcessingResult, ValidatorMerkleProof};
use beefy_light_client::commitment::{Commitment, Payload, Signature};
use beefy_light_client::{beefy_ecdsa_to_ethereum, commitment::SignedCommitment};
use beefy_merkle_tree::{merkle_proof, Keccak256};
use codec::Encode;
use hex_literal::hex;
use near_sdk::serde_json;
use secp256k1_test::{rand::thread_rng, Message as SecpMessage, Secp256k1};
use std::convert::TryInto;

type BeefyPayloadId = [u8; 2];
const MMR_ROOT_ID: BeefyPayloadId = *b"mh";

#[tokio::test]
async fn test_beefy_light_client_2() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    const MAX_VALIDATORS: i32 = 85;

    let secp = Secp256k1::new();

    let mut initial_public_keys = Vec::new();
    let mut origin_initial_public_keys = Vec::new();
    let payload = Payload(vec![(
        MMR_ROOT_ID,
        hex!("67678b4a811dc055ff865fdfdda11c7464a9c77a988af4fcdea92e38ae6c6320").to_vec(),
    )]);
    let commitment = Commitment {
        payload,
        block_number: 9,
        validator_set_id: 0,
    };
    let commitment_hash = commitment.hash();
    let msg = SecpMessage::from_slice(&commitment_hash[..]).unwrap();
    let mut signed_commitment = SignedCommitment {
        commitment,
        signatures: vec![],
    };

    for _ in 0..MAX_VALIDATORS {
        let (privkey, pubkey) = secp.generate_keypair(&mut thread_rng());
        origin_initial_public_keys.push(pubkey.serialize().to_vec());
        // println!("pubkey: {:?}", pubkey);
        // println!("prikey: {:?}", privkey);
        let validator_address = beefy_ecdsa_to_ethereum(&pubkey.serialize());
        // println!("validator_address: {:?}", validator_address);
        initial_public_keys.push(validator_address);
        let (recover_id, signature) = secp.sign_recoverable(&msg, &privkey).serialize_compact();

        let mut buf = [0_u8; 65];
        buf[0..64].copy_from_slice(&signature[..]);
        buf[64] = recover_id.to_i32() as u8;

        signed_commitment.signatures.push(Some(Signature(buf)));
    }
    let encoded_signed_commitment = signed_commitment.encode();

    let mut validator_proofs = Vec::new();
    for i in 0..initial_public_keys.len() {
        let proof = merkle_proof::<Keccak256, _, _>(initial_public_keys.clone(), i);
        validator_proofs.push(ValidatorMerkleProof {
            proof: proof.proof,
            number_of_leaves: proof.number_of_leaves.try_into().unwrap(),
            leaf_index: proof.leaf_index.try_into().unwrap(),
            leaf: proof.leaf,
        });
    }

    let encoded_mmr_leaf = hex!("c5010016000000e961cf2536958785869a8f1892c478ff5f91c5a01ece8a50d7f52cc5d31f96d3010000000000000005000000304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a20000000000000000000000000000000000000000000000000000000000000000");
    let encoded_mmr_proof = hex!("16000000000000001900000000000000143b96661a7161a6a760af588ebdefc79401e1c046d889d59f76d824406f713188c58385673dc5fffca2611dec971872597fa18462ec82f781d44c7f51f888460a927066f988d8d2b5c193a0fca08920bc21c56dfd2ea44fdcd9ceb97acd22e1a5dc8d1b12b23542b45f9e025bc4e611129aae70a08a7180839c8b698becf48e2326479d9be91711c950d8584e9f9dd49b6424e13d590afc8b00a41d5be40c4fb5");
    //
    let initial_public_keys = origin_initial_public_keys
        .iter()
        .map(|pk_bytes| format!("0x{}", hex::encode(pk_bytes)))
        .collect();
    let (
        root,
        _oct_token,
        _wrapped_appchain_token,
        _registry,
        anchor,
        _wat_faucet,
        _users,
        _appchain_message_nonce,
    ) = common::test_normal_actions(&worker, false, true, initial_public_keys).await?;
    //
    permissionless_actions::start_updating_state_of_beefy_light_client(
        &worker,
        &root,
        &anchor,
        encoded_signed_commitment.to_vec(),
        validator_proofs,
        encoded_mmr_leaf.to_vec(),
        encoded_mmr_proof.to_vec(),
    )
    .await
    .expect("Failed in calling 'start_updating_state_of_beefy_light_client'");
    loop {
        let result = permissionless_actions::try_complete_updating_state_of_beefy_light_client(
            &worker, &root, &anchor,
        )
        .await
        .expect("Failed in calling 'try_complete_updating_state_of_beefy_light_client'");
        println!(
            "Result of 'try_complete_updating_state_of_beefy_light_client': {}",
            serde_json::to_string::<MultiTxsOperationProcessingResult>(&result).unwrap()
        );
        if !result.eq(&MultiTxsOperationProcessingResult::NeedMoreGas) {
            break;
        }
    }
    Ok(())
}
