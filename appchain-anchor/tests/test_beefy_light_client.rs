use std::collections::HashMap;

use appchain_anchor::{
    types::{
        AnchorSettings, AppchainSettings, AppchainState, MultiTxsOperationProcessingResult,
        ProtocolSettings, ValidatorMerkleProof,
    },
    AppchainAnchorContract,
};
use codec::Decode;
use hex_literal::hex;
use near_sdk::{
    json_types::{U128, U64},
    serde_json,
};
use near_sdk_sim::{ContractAccount, UserAccount};

use beefy_light_client::mmr::{MmrLeaf, MmrLeafProof};
use beefy_light_client::{beefy_ecdsa_to_ethereum, commitment::SignedCommitment};

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
fn test_beefy_light_client() {
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
    let mut appchain_message_nonce: u32 = 0;
    //
    // Print validator set of era0
    //
    common::print_validator_list_of(&anchor, Some(0));
    common::print_delegator_list_of(&anchor, 0, &users[0]);
    //
    // Initialize beefy light client
    //
    let public_keys = vec![
        "0x020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1".to_string(), // Alice
        "0x0390084fdbf27d2b79d26a4f13f0ccd982cb755a661969143c37cbc49ef5b91f27".to_string(), // Bob
        "0x0389411795514af1627765eceffcbd002719f031604fadd7d188e2dc585b4e1afb".to_string(), // Charlie
        "0x03bc9d0ca094bd5b8b3225d7651eac5d18c1c04bf8ae8f8b263eebca4e1410ed0c".to_string(), // Dave
        "0x031d10105e323c4afce225208f71a6441ee327a65b9e646e772500c74d31f669aa".to_string(), // Eve
    ];
    let result = lifecycle_actions::initialize_beefy_light_client(&root, &anchor, public_keys);
    result.assert_success();
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
    //
    // user4 register validator
    //
    let user4_balance = token_viewer::get_oct_balance_of(&users[4], &oct_token);
    let amount4 = common::to_oct_amount(13_000);
    let result = staking_actions::register_validator(
        &users[4],
        &oct_token,
        &anchor,
        &Some(user4_id_in_appchain.clone()),
        amount4,
        true,
        user4_profile,
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[4], &oct_token).0,
        user4_balance.0 - amount4
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    assert_eq!(
        anchor_status.total_stake_in_next_era.0,
        amount0 + amount1 + amount2_0 + amount3_0 + amount0_p + amount2_0_p + amount4
    );
    assert_eq!(anchor_status.validator_count_in_next_era.0, 3);
    common::print_validator_profile(&anchor, &users[4].account_id(), &user4_id_in_appchain);
    //
    // Print staking histories
    //
    common::print_staking_histories(&anchor);
    //
    // Update state of beefy light client
    //
    update_state_of_beefy_light_client_1(&anchor, &users[4]);
    common::print_latest_appchain_commitment(&anchor);
    update_state_of_beefy_light_client_2(&anchor, &users[1]);
    common::print_latest_appchain_commitment(&anchor);
    //
    // Try start and complete switching era1
    //
    appchain_message_nonce += 1;
    common::switch_era(&root, &anchor, 1, appchain_message_nonce, true);
    common::print_validator_list_of(&anchor, Some(1));
    common::print_delegator_list_of(&anchor, 1, &users[0]);
    //
    // Distribut reward of era0
    //
    appchain_message_nonce += 1;
    common::distribute_reward_of(
        &root,
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        0,
        Vec::new(),
        true,
    );
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_validator_reward_histories(&anchor, &users[0], 0);
    common::print_validator_reward_histories(&anchor, &users[1], 0);
    common::print_delegator_reward_histories(&anchor, &users[2], &users[0], 0);
    common::print_delegator_reward_histories(&anchor, &users[3], &users[0], 0);
    common::print_validator_reward_histories(&anchor, &users[4], 0);
    //
    // user1 decrease stake
    //
    let result = staking_actions::decrease_stake(&users[1], &anchor, common::to_oct_amount(1000));
    result.assert_success();
    common::print_anchor_status(&anchor);
    let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(&anchor, &users[1]);
    assert!(unbonded_stakes.len() == 0);
    //
    // user2 decrease delegation
    //
    let result = staking_actions::decrease_delegation(
        &users[2],
        &anchor,
        &users[0].valid_account_id().to_string(),
        common::to_oct_amount(200),
    );
    result.assert_success();
    common::print_anchor_status(&anchor);
    let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(&anchor, &users[2]);
    assert!(unbonded_stakes.len() == 0);
    //
    // Print staking histories
    //
    common::print_staking_histories(&anchor);
    //
    // Try start and complete switching era2
    //
    appchain_message_nonce += 1;
    common::switch_era(&root, &anchor, 2, appchain_message_nonce, true);
    common::print_validator_list_of(&anchor, Some(2));
    common::print_delegator_list_of(&anchor, 2, &users[0]);
    //
    // Distribute reward of era1
    //
    appchain_message_nonce += 1;
    common::distribute_reward_of(
        &root,
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        1,
        Vec::new(),
        true,
    );
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_validator_reward_histories(&anchor, &users[0], 1);
    common::print_validator_reward_histories(&anchor, &users[1], 1);
    common::print_delegator_reward_histories(&anchor, &users[2], &users[0], 1);
    common::print_delegator_reward_histories(&anchor, &users[3], &users[0], 1);
    common::print_validator_reward_histories(&anchor, &users[4], 1);
    common::print_unbonded_stakes_of(&anchor, &users[0]);
    common::print_unbonded_stakes_of(&anchor, &users[1]);
    common::print_unbonded_stakes_of(&anchor, &users[2]);
    common::print_unbonded_stakes_of(&anchor, &users[3]);
    common::print_unbonded_stakes_of(&anchor, &users[4]);
    //
    // Change unlock period for testing
    //
    let result = settings_actions::change_unlock_period_of_validator_deposit(&root, &anchor, 3);
    result.assert_success();
    let result = settings_actions::change_unlock_period_of_delegator_deposit(&root, &anchor, 1);
    result.assert_success();
    //
    // user3 unbond delegation
    //
    let result = staking_actions::unbond_delegation(
        &users[2],
        &anchor,
        &users[0].valid_account_id().to_string(),
    );
    result.assert_success();
    common::print_anchor_status(&anchor);
    let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(&anchor, &users[2]);
    assert!(unbonded_stakes.len() == 1);
    //
    // Print staking histories
    //
    common::print_staking_histories(&anchor);
    //
    // Try start and complete switching era3
    //
    appchain_message_nonce += 1;
    common::switch_era(&root, &anchor, 3, appchain_message_nonce, true);
    common::print_validator_list_of(&anchor, Some(3));
    common::print_delegator_list_of(&anchor, 3, &users[0]);
    //
    // Distribute reward of era2
    //
    appchain_message_nonce += 1;
    common::distribute_reward_of(
        &root,
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        2,
        Vec::new(),
        true,
    );
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_validator_reward_histories(&anchor, &users[0], 2);
    common::print_validator_reward_histories(&anchor, &users[1], 2);
    common::print_delegator_reward_histories(&anchor, &users[2], &users[0], 2);
    common::print_delegator_reward_histories(&anchor, &users[3], &users[0], 2);
    common::print_validator_reward_histories(&anchor, &users[4], 2);
    common::print_unbonded_stakes_of(&anchor, &users[0]);
    common::print_unbonded_stakes_of(&anchor, &users[1]);
    common::print_unbonded_stakes_of(&anchor, &users[2]);
    common::print_unbonded_stakes_of(&anchor, &users[3]);
    common::print_unbonded_stakes_of(&anchor, &users[4]);
    //
    // user0 unbond stake
    //
    let result = staking_actions::unbond_stake(&users[0], &anchor);
    result.assert_success();
    common::print_anchor_status(&anchor);
    let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(&anchor, &users[0]);
    assert!(unbonded_stakes.len() == 0);
    //
    // Print staking histories
    //
    common::print_staking_histories(&anchor);
    //
    // Try start and complete switching era3
    //
    appchain_message_nonce += 1;
    common::switch_era(&root, &anchor, 4, appchain_message_nonce, true);
    common::print_validator_list_of(&anchor, Some(4));
    common::print_delegator_list_of(&anchor, 4, &users[0]);
    //
    // Distribute reward of era3
    //
    appchain_message_nonce += 1;
    common::distribute_reward_of(
        &root,
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        3,
        Vec::new(),
        true,
    );
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_validator_reward_histories(&anchor, &users[0], 3);
    common::print_validator_reward_histories(&anchor, &users[1], 3);
    common::print_delegator_reward_histories(&anchor, &users[2], &users[0], 3);
    common::print_delegator_reward_histories(&anchor, &users[3], &users[0], 3);
    common::print_validator_reward_histories(&anchor, &users[4], 3);
    common::print_unbonded_stakes_of(&anchor, &users[0]);
    common::print_unbonded_stakes_of(&anchor, &users[1]);
    common::print_unbonded_stakes_of(&anchor, &users[2]);
    common::print_unbonded_stakes_of(&anchor, &users[3]);
    common::print_unbonded_stakes_of(&anchor, &users[4]);
    //
    // Withdraw validator rewards
    //
    common::withdraw_validator_rewards_of(&anchor, &users[0], &wrapped_appchain_token, 3);
    common::withdraw_validator_rewards_of(&anchor, &users[1], &wrapped_appchain_token, 3);
    common::withdraw_validator_rewards_of(&anchor, &users[4], &wrapped_appchain_token, 3);
    //
    // Withdraw delegator rewards
    //
    common::withdraw_delegator_rewards_of(
        &anchor,
        &users[2],
        &users[0],
        &wrapped_appchain_token,
        3,
    );
    common::withdraw_delegator_rewards_of(
        &anchor,
        &users[3],
        &users[0],
        &wrapped_appchain_token,
        3,
    );
    //
    // Withdraw stake
    //
    common::withdraw_stake_of(&anchor, &users[0], &oct_token);
    common::withdraw_stake_of(&anchor, &users[1], &oct_token);
    common::withdraw_stake_of(&anchor, &users[2], &oct_token);
    common::withdraw_stake_of(&anchor, &users[3], &oct_token);
    common::withdraw_stake_of(&anchor, &users[4], &oct_token);
    //
    // Reset history data
    //
    let result = sudo_actions::reset_validator_set_histories_to(&root, &anchor, U64::from(0));
    result.assert_success();
    let result = sudo_actions::clear_anchor_event_histories(&root, &anchor);
    result.assert_success();
    common::print_validator_list_of(&anchor, Some(0));
    common::print_validator_list_of(&anchor, Some(1));
    common::print_validator_list_of(&anchor, Some(2));
    common::print_validator_list_of(&anchor, Some(3));
    common::print_staking_histories(&anchor);
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
}

fn update_state_of_beefy_light_client_1(
    anchor: &ContractAccount<AppchainAnchorContract>,
    user: &UserAccount,
) {
    let alice_pk = beefy_ecdsa_to_ethereum(
        &hex!("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1")[..],
    );
    let bob_pk = beefy_ecdsa_to_ethereum(
        &hex!("0390084fdbf27d2b79d26a4f13f0ccd982cb755a661969143c37cbc49ef5b91f27")[..],
    );
    let charlie_pk = beefy_ecdsa_to_ethereum(
        &hex!("0389411795514af1627765eceffcbd002719f031604fadd7d188e2dc585b4e1afb")[..],
    );
    let dave_pk = beefy_ecdsa_to_ethereum(
        &hex!("03bc9d0ca094bd5b8b3225d7651eac5d18c1c04bf8ae8f8b263eebca4e1410ed0c")[..],
    );
    let eve_pk = beefy_ecdsa_to_ethereum(
        &hex!("031d10105e323c4afce225208f71a6441ee327a65b9e646e772500c74d31f669aa")[..],
    );

    let encoded_signed_commitment_1 = hex!("f45927644a0b5bc6f1ce667330071fbaea498403c084eb0d4cb747114887345d0900000000000000000000001401b9b5b39fb15d7e22710ad06075cf0e20c4b0c1e3d0a6482946e1d0daf86ca2e37b40209316f00a549cdd2a7fd191694fee4f76f698d0525642563e665db85d6300010ee39cb2cb008f7dce753541b5442e98a260250286b335d6048f2dd4695237655ccc93ebcd3d7c04461e0b9d12b81b21a826c5ee3eebcd6ab9e85c8717f6b1ae010001b094279e0bb4442ba07165da47ab9c0d7d0f479e31d42c879564915714e8ea3d42393dc430addc4a5f416316c02e0676e525c56a3d0c0033224ebda4c83052670001f965d806a16c5dfb9d119f78cdbed379bccb071528679306208880ad29a9cf9e00e75f1b284fa3457b7b37223a2272cf2bf90ce4fd7e84e321eddec3cdeb66f801");
    let signed_commitment_1 = SignedCommitment::decode(&mut &encoded_signed_commitment_1[..]);
    println!("signed_commitment_1: {:?}", signed_commitment_1);

    let validator_proofs_1 = vec![
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("f68aec7304bf37f340dae2ea20fb5271ee28a3128812b84a615da4789e458bde").into(),
                hex!("93c6c7e160154c8467b700c291a1d4da94ae9aaf1c5010003a6aa3e9b18657ab").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 0,
            leaf: alice_pk.clone(),
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("aeb47a269393297f4b0a3c9c9cfd00c7a4195255274cf39d83dabc2fcc9ff3d7").into(),
                hex!("93c6c7e160154c8467b700c291a1d4da94ae9aaf1c5010003a6aa3e9b18657ab").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 1,
            leaf: bob_pk.clone(),
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("50bdd3ac4f54a04702a055c33303025b2038446c7334ed3b3341f310f052116f").into(),
                hex!("697ea2a8fe5b03468548a7a413424a6292ab44a82a6f5cc594c3fa7dda7ce402").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 2,
            leaf: charlie_pk.clone(),
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("3eb799651607280e854bd2e42c1df1c8e4a6167772dfb3c64a813e40f6e87136").into(),
                hex!("697ea2a8fe5b03468548a7a413424a6292ab44a82a6f5cc594c3fa7dda7ce402").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 3,
            leaf: dave_pk.clone(),
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("2145814fb41496b2881ca364a06e320fd1bf2fa7b94e1e37325cefbe29056519").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 4,
            leaf: eve_pk.clone(),
        },
    ];

    let  encoded_mmr_leaf_1 = hex!("c501000800000079f0451c096266bee167393545bafc7b27b7d14810084a843955624588ba29c1010000000000000005000000304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a20000000000000000000000000000000000000000000000000000000000000000");

    let leaf: Vec<u8> = Decode::decode(&mut &encoded_mmr_leaf_1[..]).unwrap();
    let mmr_leaf_1: MmrLeaf = Decode::decode(&mut &*leaf).unwrap();
    println!("mmr_leaf_1: {:?}", mmr_leaf_1);

    let encoded_mmr_proof_1 =  hex!("0800000000000000090000000000000004c2d6348aef1ef52e779c59bcc1d87fa0175b59b4fa2ea8fc322e4ceb2bdd1ea2");
    let mmr_proof_1 = MmrLeafProof::decode(&mut &encoded_mmr_proof_1[..]);
    println!("mmr_proof_1: {:?}", mmr_proof_1);
    //
    let result = permissionless_actions::start_updating_state_of_beefy_light_client(
        &user,
        &anchor,
        encoded_signed_commitment_1.to_vec(),
        validator_proofs_1,
        encoded_mmr_leaf_1.to_vec(),
        encoded_mmr_proof_1.to_vec(),
    );
    result.assert_success();
    let result =
        permissionless_actions::try_complete_updating_state_of_beefy_light_client(&user, &anchor);
    println!(
        "Result of 'try_complete_updating_state_of_beefy_light_client': {}",
        serde_json::to_string::<MultiTxsOperationProcessingResult>(&result).unwrap()
    )
}

fn update_state_of_beefy_light_client_2(
    anchor: &ContractAccount<AppchainAnchorContract>,
    user: &UserAccount,
) {
    let alice_pk = beefy_ecdsa_to_ethereum(
        &hex!("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1")[..],
    );
    let bob_pk = beefy_ecdsa_to_ethereum(
        &hex!("0390084fdbf27d2b79d26a4f13f0ccd982cb755a661969143c37cbc49ef5b91f27")[..],
    );
    let charlie_pk = beefy_ecdsa_to_ethereum(
        &hex!("0389411795514af1627765eceffcbd002719f031604fadd7d188e2dc585b4e1afb")[..],
    );
    let dave_pk = beefy_ecdsa_to_ethereum(
        &hex!("03bc9d0ca094bd5b8b3225d7651eac5d18c1c04bf8ae8f8b263eebca4e1410ed0c")[..],
    );
    let eve_pk = beefy_ecdsa_to_ethereum(
        &hex!("031d10105e323c4afce225208f71a6441ee327a65b9e646e772500c74d31f669aa")[..],
    );

    let encoded_signed_commitment_2 = hex!("8d3cb96dca5110aff60423046bbf4a76db0e71158aa5586ffa3423fbaf9ef1da1100000000000000000000001401864ce4553324cc92db4ac622b9dbb031a6a4bd26ee1ab66e0272f567928865ec46847b55f98fa7e1dbafb0256f0a23e2f0a375e4547f5d1819d9b8694f17f6a80101c9ae8aad1b81e2249736324716c09c122889317e4f3e47066c501a839c15312e5c823dd37436d8e3bac8041329c5d0ed5dd94c45b5c1eed13d9111924f0a13c1000159fe06519c672d183de7776b6902a13c098d917721b5600a2296dca3a74a81bc01031a671fdb5e5050ff1f432d72e7a2c144ab38f8401ffd368e693257162a4600014290c6aa5028ceb3a3a773c80beee2821f3a7f5b43f592f7a82b0cbbbfab5ba41363daae5a7006fea2f89a30b4900f85fa82283587df789fd7b5b773ad7e8c410100");
    let signed_commitment_2 = SignedCommitment::decode(&mut &encoded_signed_commitment_2[..]);
    println!("signed_commitment_2: {:?}", signed_commitment_2);

    let validator_proofs_2 = vec![
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("f68aec7304bf37f340dae2ea20fb5271ee28a3128812b84a615da4789e458bde").into(),
                hex!("93c6c7e160154c8467b700c291a1d4da94ae9aaf1c5010003a6aa3e9b18657ab").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 0,
            leaf: alice_pk,
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("aeb47a269393297f4b0a3c9c9cfd00c7a4195255274cf39d83dabc2fcc9ff3d7").into(),
                hex!("93c6c7e160154c8467b700c291a1d4da94ae9aaf1c5010003a6aa3e9b18657ab").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 1,
            leaf: bob_pk,
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("50bdd3ac4f54a04702a055c33303025b2038446c7334ed3b3341f310f052116f").into(),
                hex!("697ea2a8fe5b03468548a7a413424a6292ab44a82a6f5cc594c3fa7dda7ce402").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 2,
            leaf: charlie_pk,
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("3eb799651607280e854bd2e42c1df1c8e4a6167772dfb3c64a813e40f6e87136").into(),
                hex!("697ea2a8fe5b03468548a7a413424a6292ab44a82a6f5cc594c3fa7dda7ce402").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 3,
            leaf: dave_pk,
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("2145814fb41496b2881ca364a06e320fd1bf2fa7b94e1e37325cefbe29056519").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 4,
            leaf: eve_pk,
        },
    ];

    let encoded_mmr_leaf_2 = hex!("c5010010000000d0a3a930e5f3b0f997c3794023c86f8ba28c6ba2cacf230d08d46be0fdf29435010000000000000005000000304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a20000000000000000000000000000000000000000000000000000000000000000");

    let leaf: Vec<u8> = Decode::decode(&mut &encoded_mmr_leaf_2[..]).unwrap();
    let mmr_leaf_2: MmrLeaf = Decode::decode(&mut &*leaf).unwrap();
    println!("mmr_leaf_2: {:?}", mmr_leaf_2);

    let encoded_mmr_proof_2 =  hex!("10000000000000001100000000000000048a766e1ab001e2ff796517dcfbff957a751c994aff4c3ba9447a46d88ec2ef15");
    let mmr_proof_2 = MmrLeafProof::decode(&mut &encoded_mmr_proof_2[..]);
    println!("mmr_proof_2: {:?}", mmr_proof_2);
    //
    let result = permissionless_actions::start_updating_state_of_beefy_light_client(
        &user,
        &anchor,
        encoded_signed_commitment_2.to_vec(),
        validator_proofs_2,
        encoded_mmr_leaf_2.to_vec(),
        encoded_mmr_proof_2.to_vec(),
    );
    result.assert_success();
    let result =
        permissionless_actions::try_complete_updating_state_of_beefy_light_client(&user, &anchor);
    println!(
        "Result of 'try_complete_updating_state_of_beefy_light_client': {}",
        serde_json::to_string::<MultiTxsOperationProcessingResult>(&result).unwrap()
    )
}
