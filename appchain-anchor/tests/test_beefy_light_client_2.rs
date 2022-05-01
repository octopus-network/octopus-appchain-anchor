use std::{collections::HashMap, convert::TryInto};

use appchain_anchor::{
    types::{
        AnchorSettings, AppchainSettings, AppchainState, MultiTxsOperationProcessingResult,
        ProtocolSettings, ValidatorMerkleProof,
    },
    AppchainAnchorContract,
};
use codec::Encode;
use hex_literal::hex;
use near_sdk::{json_types::U128, serde_json};
use near_sdk_sim::{ContractAccount, UserAccount};
use secp256k1_test::{rand::thread_rng, Message as SecpMessage, Secp256k1};

use beefy_light_client::commitment::{Commitment, Signature};
use beefy_light_client::{beefy_ecdsa_to_ethereum, commitment::SignedCommitment};
use beefy_merkle_tree::{merkle_proof, Keccak256};

mod anchor_viewer;
mod common;
mod lifecycle_actions;
mod owner_actions;
mod permissionless_actions;
mod settings_actions;
mod staking_actions;
mod sudo_actions;
mod token_viewer;
mod validator_actions;
mod wrapped_appchain_token_manager;

const TOTAL_SUPPLY: u128 = 100_000_000;

#[test]
fn test_beefy_light_client_2() {
    let total_supply = common::to_oct_amount(TOTAL_SUPPLY);
    let (root, oct_token, _registry, anchor, users) = common::init(total_supply, false);
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    let user1_id_in_appchain =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da270".to_string();
    let user4_id_in_appchain =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da273".to_string();
    let mut user0_profile = HashMap::<String, String>::new();
    user0_profile.insert("key0".to_string(), "value0".to_string());
    let mut user1_profile = HashMap::<String, String>::new();
    user1_profile.insert("key1".to_string(), "value1".to_string());
    let mut user4_profile = HashMap::<String, String>::new();
    user4_profile.insert("key4".to_string(), "value4".to_string());
    //
    // Check initial status
    //
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor),
        AppchainState::Staging
    );
    let anchor_settings = anchor_viewer::get_anchor_settings(&anchor);
    println!(
        "Anchor settings: {}",
        serde_json::to_string::<AnchorSettings>(&anchor_settings).unwrap()
    );
    let appchain_settings = anchor_viewer::get_appchain_settings(&anchor);
    println!(
        "Appchain settings: {}",
        serde_json::to_string::<AppchainSettings>(&appchain_settings).unwrap()
    );
    let protocol_settings = anchor_viewer::get_protocol_settings(&anchor);
    println!(
        "Protocol settings: {}",
        serde_json::to_string::<ProtocolSettings>(&protocol_settings).unwrap()
    );
    assert_eq!(
        protocol_settings.minimum_validator_deposit.0,
        common::to_oct_amount(10_000)
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    assert_eq!(anchor_status.total_stake_in_next_era.0, 0);
    assert_eq!(anchor_status.validator_count_in_next_era.0, 0);
    //
    //
    //
    let result = settings_actions::set_price_of_oct_token(&users[4], &anchor, 2_130_000);
    assert!(!result.is_ok());
    let result = wrapped_appchain_token_manager::set_price_of_wrapped_appchain_token(
        &users[4], &anchor, 110_000,
    );
    assert!(!result.is_ok());
    let result = settings_actions::set_token_price_maintainer_account(&root, &anchor, &users[4]);
    result.assert_success();
    //
    // Initialize wrapped appchain token contract.
    //
    let result = wrapped_appchain_token_manager::set_price_of_wrapped_appchain_token(
        &users[4], &anchor, 110,
    );
    result.assert_success();
    let result = wrapped_appchain_token_manager::set_account_of_wrapped_appchain_token(
        &root,
        &anchor,
        "wrapped_appchain_token".to_string(),
    );
    result.assert_success();
    let wrapped_appchain_token = common::deploy_wrapped_appchain_token_contract(
        &root,
        &anchor,
        U128::from(total_supply / 2),
        &users,
    );
    common::print_wrapped_appchain_token_info(&anchor);
    //
    // user0 register validator
    //
    let user0_balance = token_viewer::get_oct_balance_of(&users[0], &oct_token);
    let amount0 = common::to_oct_amount(10_000);
    let result = staking_actions::register_validator(
        &users[0],
        &oct_token,
        &anchor,
        &None,
        amount0,
        true,
        HashMap::new(),
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[0], &oct_token).0,
        user0_balance.0 - amount0
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    assert_eq!(anchor_status.total_stake_in_next_era.0, amount0);
    assert_eq!(anchor_status.validator_count_in_next_era.0, 1);
    common::print_validator_profile(&anchor, &users[0].account_id(), &user0_id_in_appchain);
    //
    // user1 register validator
    //
    let user1_balance = token_viewer::get_oct_balance_of(&users[1], &oct_token);
    let amount1 = common::to_oct_amount(15_000);
    let result = staking_actions::register_validator(
        &users[1],
        &oct_token,
        &anchor,
        &None,
        amount1,
        false,
        HashMap::new(),
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[1], &oct_token).0,
        user1_balance.0 - amount1
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    assert_eq!(anchor_status.total_stake_in_next_era.0, amount0 + amount1);
    assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    common::print_validator_profile(&anchor, &users[1].account_id(), &user1_id_in_appchain);
    //
    // user2 register delegator to user0
    //
    let user2_balance = token_viewer::get_oct_balance_of(&users[2], &oct_token);
    let amount2_0 = common::to_oct_amount(1000);
    let result = staking_actions::register_delegator(
        &users[2],
        &oct_token,
        &anchor,
        &users[0].account_id(),
        amount2_0,
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[2], &oct_token).0,
        user2_balance.0 - amount2_0
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    assert_eq!(
        anchor_status.total_stake_in_next_era.0,
        amount0 + amount1 + amount2_0
    );
    assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    //
    // user3 register delegator to user0
    //
    let user3_balance = token_viewer::get_oct_balance_of(&users[3], &oct_token);
    let amount3_0 = common::to_oct_amount(2000);
    let result = staking_actions::register_delegator(
        &users[3],
        &oct_token,
        &anchor,
        &users[0].account_id(),
        amount3_0,
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[3], &oct_token).0,
        user3_balance.0 - amount3_0
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    assert_eq!(
        anchor_status.total_stake_in_next_era.0,
        amount0 + amount1 + amount2_0 + amount3_0
    );
    assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    //
    // user0 increase stake
    //
    let user0_balance = token_viewer::get_oct_balance_of(&users[0], &oct_token);
    let amount0_p = common::to_oct_amount(1_200);
    let result = staking_actions::increase_stake(&users[0], &oct_token, &anchor, amount0_p);
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[0], &oct_token).0,
        user0_balance.0 - amount0_p
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    assert_eq!(
        anchor_status.total_stake_in_next_era.0,
        amount0 + amount1 + amount2_0 + amount3_0 + amount0_p
    );
    assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    //
    // user2 increase delegation to user0
    //
    let user2_balance = token_viewer::get_oct_balance_of(&users[2], &oct_token);
    let amount2_0_p = common::to_oct_amount(500);
    let result = staking_actions::increase_delegation(
        &users[2],
        &oct_token,
        &anchor,
        &users[0].account_id(),
        amount2_0_p,
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[2], &oct_token).0,
        user2_balance.0 - amount2_0_p
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    assert_eq!(
        anchor_status.total_stake_in_next_era.0,
        amount0 + amount1 + amount2_0 + amount3_0 + amount0_p + amount2_0_p
    );
    assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    //
    // Print anchor status and staking histories
    //
    common::print_anchor_status(&anchor);
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_staking_histories(&anchor);
    common::print_validator_list_of(&anchor, None);
    //
    // Try go_booting
    //
    let result = lifecycle_actions::go_booting(&root, &anchor);
    assert!(!result.is_ok());
    //
    // Change protocol settings and try go_booting
    //
    let result = settings_actions::change_minimum_validator_count(&root, &anchor, 2);
    result.assert_success();
    let result = lifecycle_actions::go_booting(&root, &anchor);
    assert!(!result.is_ok());
    //
    // Change price of OCT token and try go_booting
    //
    let result = settings_actions::set_price_of_oct_token(&users[4], &anchor, 2_130_000);
    result.assert_success();
    let result = lifecycle_actions::go_booting(&root, &anchor);
    assert!(!result.is_ok());
    //
    // Change total stake price and try go_booting
    //
    let result = settings_actions::change_minimum_total_stake_price_for_booting(
        &root,
        &anchor,
        63_000_000_000,
    );
    result.assert_success();
    let result = lifecycle_actions::go_booting(&root, &anchor);
    result.assert_success();
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor),
        AppchainState::Booting
    );
    //
    // Print validator set of era0
    //
    common::print_validator_list_of(&anchor, Some(0));
    common::print_delegator_list_of(&anchor, 0, &users[0]);
    //
    // Initialize beefy light client
    //
    update_state_of_beefy_light_client_max(&anchor, &root);
    common::print_latest_appchain_commitment(&anchor);
    //
    // Go live
    //
    let result = settings_actions::set_rpc_endpoint(&root, &anchor, "rpc_endpoint".to_string());
    result.assert_success();
    let result = settings_actions::set_era_reward(&root, &anchor, common::to_oct_amount(10));
    result.assert_success();
    let result = lifecycle_actions::go_live(&root, &anchor);
    result.assert_success();
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor),
        AppchainState::Active
    );
}

fn update_state_of_beefy_light_client_max(
    anchor: &ContractAccount<AppchainAnchorContract>,
    user: &UserAccount,
) {
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
    let result = lifecycle_actions::initialize_beefy_light_client(
        &user,
        &anchor,
        origin_initial_public_keys
            .iter()
            .map(|pk_bytes| format!("0x{}", hex::encode(pk_bytes)))
            .collect(),
    );
    result.assert_success();
    //
    let result = permissionless_actions::start_updating_state_of_beefy_light_client(
        &user,
        &anchor,
        encoded_signed_commitment.to_vec(),
        validator_proofs,
        encoded_mmr_leaf.to_vec(),
        encoded_mmr_proof.to_vec(),
    );
    result.assert_success();
    loop {
        let result = permissionless_actions::try_complete_updating_state_of_beefy_light_client(
            &user, &anchor,
        );
        println!(
            "Result of 'try_complete_updating_state_of_beefy_light_client': {}",
            serde_json::to_string::<MultiTxsOperationProcessingResult>(&result).unwrap()
        );
        if !result.eq(&MultiTxsOperationProcessingResult::NeedMoreGas) {
            break;
        }
    }
}
