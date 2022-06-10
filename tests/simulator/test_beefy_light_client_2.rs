use crate::{common, contract_interfaces::permissionless_actions};
use appchain_anchor::types::{MultiTxsOperationProcessingResult, ValidatorMerkleProof};
use beefy_light_client::commitment::{Commitment, Signature};
use beefy_light_client::{beefy_ecdsa_to_ethereum, commitment::SignedCommitment};
use beefy_merkle_tree::{merkle_proof, Keccak256};
use codec::Encode;
use hex_literal::hex;
use near_sdk::serde_json;
use secp256k1_test::{rand::thread_rng, Message as SecpMessage, Secp256k1};
use std::convert::TryInto;

#[tokio::test]
async fn test_beefy_light_client_2() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    const MAX_VALIDATORS: i32 = 85;

    let secp = Secp256k1::new();

    let mut initial_public_keys = Vec::new();
    let mut origin_initial_public_keys = Vec::new();
    let commitment = Commitment {
        payload: hex!("f45927644a0b5bc6f1ce667330071fbaea498403c084eb0d4cb747114887345d"),
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
            root: proof.root,
            proof: proof.proof,
            number_of_leaves: proof.number_of_leaves.try_into().unwrap(),
            leaf_index: proof.leaf_index.try_into().unwrap(),
            leaf: proof.leaf,
        });
    }

    let encoded_mmr_leaf = hex!("c501000800000079f0451c096266bee167393545bafc7b27b7d14810084a843955624588ba29c1010000000000000005000000304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a20000000000000000000000000000000000000000000000000000000000000000");
    let encoded_mmr_proof = hex!("0800000000000000090000000000000004c2d6348aef1ef52e779c59bcc1d87fa0175b59b4fa2ea8fc322e4ceb2bdd1ea2");
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
