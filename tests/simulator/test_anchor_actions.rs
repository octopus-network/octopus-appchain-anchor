use std::collections::HashMap;

use near_sdk::{
    json_types::{U128, U64},
    serde_json::{self, json},
};
use near_sdk_sim::{call, ContractAccount, UserAccount};

use appchain_anchor::{appchain_challenge::AppchainChallenge, AppchainAnchorContract};
use mock_oct_token::MockOctTokenContract;
use wrapped_appchain_token::WrappedAppchainTokenContract;

use crate::{
    anchor_viewer, common, owner_actions, settings_manager, staking_actions, sudo_actions,
};

#[test]
fn test_anchor_actions() {
    let (
        root,
        oct_token,
        wrapped_appchain_token,
        _registry,
        anchor,
        users,
        mut appchain_message_nonce,
    ) = common::test_normal_actions(false, true);
    //
    // Try start and complete switching era1
    //
    appchain_message_nonce += 1;
    common::switch_era(&root, &anchor, 1, appchain_message_nonce, false);
    common::print_validator_set_info_of(&anchor, U64::from(1));
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
        false,
    );
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_validator_reward_histories(&anchor, &users[0], 0);
    common::print_validator_reward_histories(&anchor, &users[1], 0);
    common::print_delegator_reward_histories(&anchor, &users[2], &users[0], 0);
    common::print_delegator_reward_histories(&anchor, &users[3], &users[0], 0);
    common::print_validator_reward_histories(&anchor, &users[4], 0);
    //
    //
    //
    test_staking_actions(
        &root,
        &oct_token,
        &wrapped_appchain_token,
        &anchor,
        &users,
        appchain_message_nonce,
    );
    //
    // Reset contract status
    //
    let result = sudo_actions::clear_reward_distribution_records(&root, &anchor, U64::from(3));
    result.assert_success();
    let result = sudo_actions::clear_reward_distribution_records(&root, &anchor, U64::from(2));
    result.assert_success();
    let result = sudo_actions::clear_reward_distribution_records(&root, &anchor, U64::from(1));
    result.assert_success();
    let result = sudo_actions::clear_reward_distribution_records(&root, &anchor, U64::from(0));
    result.assert_success();
    let result = sudo_actions::clear_unbonded_stakes(&root, &anchor);
    result.assert_success();
    let result = sudo_actions::clear_unwithdrawn_rewards(&root, &anchor, U64::from(3));
    result.assert_success();
    let result = sudo_actions::clear_unwithdrawn_rewards(&root, &anchor, U64::from(2));
    result.assert_success();
    let result = sudo_actions::clear_unwithdrawn_rewards(&root, &anchor, U64::from(1));
    result.assert_success();
    let result = sudo_actions::clear_unwithdrawn_rewards(&root, &anchor, U64::from(0));
    result.assert_success();
    let result = sudo_actions::clear_anchor_event_histories(&root, &anchor);
    result.assert_success();
    let result = sudo_actions::clear_appchain_notification_histories(&root, &anchor);
    result.assert_success();
    let result = sudo_actions::reset_validator_set_histories_to(&root, &anchor, U64::from(2));
    result.assert_success();
    let result = sudo_actions::reset_validator_set_histories_to(&root, &anchor, U64::from(1));
    result.assert_success();
    let result = sudo_actions::reset_validator_set_histories_to(&root, &anchor, U64::from(0));
    result.assert_success();
    common::print_anchor_status(&anchor);
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_validator_list_of(&anchor, Some(0));
    common::print_validator_list_of(&anchor, Some(1));
    common::print_validator_list_of(&anchor, Some(2));
    common::print_validator_list_of(&anchor, Some(3));
    common::print_staking_histories(&anchor);
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
}

fn test_staking_actions(
    root: &UserAccount,
    oct_token: &ContractAccount<MockOctTokenContract>,
    wrapped_appchain_token: &ContractAccount<WrappedAppchainTokenContract>,
    anchor: &ContractAccount<AppchainAnchorContract>,
    users: &Vec<UserAccount>,
    mut appchain_message_nonce: u32,
) {
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    let user4_id_in_appchain =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da273".to_string();
    let mut user0_profile = HashMap::<String, String>::new();
    user0_profile.insert("key0".to_string(), "value0".to_string());
    let mut user1_profile = HashMap::<String, String>::new();
    user1_profile.insert("key1".to_string(), "value1".to_string());
    let mut user4_profile = HashMap::<String, String>::new();
    user4_profile.insert("key4".to_string(), "value4".to_string());
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
        &users[0].account_id(),
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
        [user0_id_in_appchain.clone()].to_vec(),
        true,
    );
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_staking_histories(&anchor);
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
    let result = settings_manager::change_unlock_period_of_validator_deposit(&root, &anchor, 3);
    result.assert_success();
    let result = settings_manager::change_unlock_period_of_delegator_deposit(&root, &anchor, 1);
    result.assert_success();
    //
    // user3 unbond delegation
    //
    let result = staking_actions::unbond_delegation(&users[2], &anchor, &users[0].account_id());
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
        [user0_id_in_appchain.clone(), user4_id_in_appchain.clone()].to_vec(),
        true,
    );
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_staking_histories(&anchor);
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
    // let result = staking_actions::unbond_stake(&users[0], &anchor);
    // result.assert_success();
    // common::print_anchor_status(&anchor);
    // let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(&anchor, &users[0]);
    // assert!(unbonded_stakes.len() == 0);
    //
    // user1 unbond stake
    //
    // let result = staking_actions::unbond_stake(&users[1], &anchor);
    // result.assert_success();
    // common::print_anchor_status(&anchor);
    // let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(&anchor, &users[1]);
    // assert!(unbonded_stakes.len() == 1);
    //
    // Print staking histories
    //
    common::print_staking_histories(&anchor);
    //
    // Try start and complete switching era4
    //
    appchain_message_nonce += 1;
    common::switch_era(&root, &anchor, 4, appchain_message_nonce, true);
    common::print_validator_list_of(&anchor, Some(4));
    common::print_delegator_list_of(&anchor, 4, &users[0]);
    //
    // Distribute reward of era3
    //
    common::print_validator_set_info_of(&anchor, U64::from(3));
    appchain_message_nonce += 1;
    common::distribute_reward_of(
        &root,
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        3,
        [user0_id_in_appchain.clone(), user4_id_in_appchain.clone()].to_vec(),
        true,
    );
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_staking_histories(&anchor);
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
    // Try start and complete switching era5
    //
    appchain_message_nonce += 1;
    common::switch_era(&root, &anchor, 5, appchain_message_nonce, true);
    common::print_validator_list_of(&anchor, Some(5));
    common::print_delegator_list_of(&anchor, 5, &users[0]);
    //
    // Distribute reward of era4
    //
    common::print_validator_set_info_of(&anchor, U64::from(4));
    appchain_message_nonce += 1;
    common::distribute_reward_of(
        &root,
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        4,
        Vec::new(),
        true,
    );
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_staking_histories(&anchor);
    common::print_validator_reward_histories(&anchor, &users[0], 4);
    common::print_validator_reward_histories(&anchor, &users[1], 4);
    common::print_delegator_reward_histories(&anchor, &users[2], &users[0], 4);
    common::print_delegator_reward_histories(&anchor, &users[3], &users[0], 4);
    common::print_validator_reward_histories(&anchor, &users[4], 4);
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
    // Print whole status
    //
    common::print_anchor_status(&anchor);
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_validator_list_of(&anchor, Some(0));
    common::print_validator_list_of(&anchor, Some(1));
    common::print_validator_list_of(&anchor, Some(2));
    common::print_validator_list_of(&anchor, Some(3));
    common::print_staking_histories(&anchor);
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
}

#[test]
fn test_migration() {
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    let user1_id_in_appchain =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da270".to_string();
    //
    let (root, _, _, _, anchor, users, _) = common::test_normal_actions(true, false);
    common::deploy_new_anchor_contract(&anchor);
    let result = call!(root, anchor.migrate_state());
    common::print_execution_result("migrate_state", &result);
    //
    //
    //
    common::print_anchor_status(&anchor);
    common::print_anchor_status(&anchor);
    common::print_wrapped_appchain_token_info(&anchor);
    common::print_appchain_settings(&anchor);
    common::print_anchor_settings(&anchor);
    common::print_validator_set_info_of(&anchor, U64::from(0));
    common::print_validator_list_of(&anchor, Some(0));
    common::print_validator_list_of(&anchor, Some(1));
    common::print_validator_list_of(&anchor, Some(2));
    common::print_validator_list_of(&anchor, Some(3));
    common::print_user_staking_histories_of(&anchor, &users[0]);
    common::print_user_staking_histories_of(&anchor, &users[1]);
    common::print_user_staking_histories_of(&anchor, &users[2]);
    common::print_user_staking_histories_of(&anchor, &users[3]);
    common::print_user_staking_histories_of(&anchor, &users[4]);
    common::print_validator_profile(&anchor, &users[0].account_id(), &user0_id_in_appchain);
    common::print_validator_profile(&anchor, &users[1].account_id(), &user1_id_in_appchain);
    common::print_staking_histories(&anchor);
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
}

#[test]
fn test_equivocation_challenge() {
    //
    let (_root, _, _, _, anchor, users, _) = common::test_normal_actions(false, false);
    //
    if let Ok(challenge) = serde_json::from_str::<AppchainChallenge>("{\"EquivocationChallenge\":{\"submitter_account\":\"tt.testnet\",\"proof\":{\"set_id\":0,\"equivocation\":{\"Prevote\":{\"round_number\":2,\"identity\":[209,124,45,120,35,235,242,96,253,19,143,45,126,39,209,20,192,20,93,150,139,95,245,0,97,37,242,65,79,173,174,105],\"first\":[{\"target_hash\":[96,43,40,162,148,136,214,36,237,150,130,159,164,176,134,217,188,7,156,28,26,245,153,173,235,220,148,113,142,54,86,172],\"target_number\":2},[160,8,125,180,57,254,58,164,29,247,251,21,218,43,228,81,110,42,54,245,139,100,113,120,8,169,186,72,79,1,10,44,124,214,240,57,158,28,5,246,112,141,249,88,85,136,172,109,27,246,217,212,175,90,35,66,230,60,7,116,132,238,222,10]],\"second\":[{\"target_hash\":[55,65,109,173,2,87,56,21,245,65,225,251,11,255,55,219,64,83,133,115,5,161,227,232,204,172,40,117,127,126,63,225],\"target_number\":1},[182,116,59,76,20,131,229,152,169,61,221,96,84,126,111,231,69,122,21,132,2,242,18,172,118,22,204,130,230,203,228,28,91,196,141,105,180,223,209,205,3,210,217,106,135,148,174,214,169,196,82,106,255,89,109,197,73,142,237,71,179,42,184,7]]}}}}}") {
        let result = call!(
            users[3],
            anchor.commit_appchain_challenge(challenge)
        );
        common::print_execution_result("commit_appchain_challenge", &result);
        //
        let appchain_challenge = anchor_viewer::get_appchain_challenge(&anchor, 0);
        println!(
            "Appchain challenge 0: {}",
            serde_json::to_string(&appchain_challenge).unwrap()
        );
    //
        let appchain_challenges = anchor_viewer::get_appchain_challenges(&anchor, 0, None);
        let mut index = 0;
        for appchain_challenge in appchain_challenges {
            println!(
                "Appchain challenge {}: {}",
                index,
                serde_json::to_string(&appchain_challenge).unwrap()
            );
            index += 1;
        }
    } else {
        panic!("Wrong testing data for equivocation challenge.");
    }
}

#[test]
fn test_transfer_oct_to_appchain() {
    //
    let (root, oct_token, _, _, anchor, users, _) = common::test_normal_actions(false, false);
    //
    let execution_result = owner_actions::register_near_fungible_token(
        &root,
        &anchor,
        "OCT".to_string(),
        "Oct token".to_string(),
        18,
        oct_token.account_id(),
        U128::from(1000000),
    );
    assert!(execution_result.is_ok());
    common::print_near_fungible_tokens(&anchor);
    //
    let execution_result = common::ft_transfer_call_oct_token(
        &users[0],
        &anchor.user_account,
        common::to_oct_amount(200),
        json!({
            "BridgeToAppchain": {
                "receiver_id_in_appchain": "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string(),
            }
        })
        .to_string(),
        &oct_token,
    );
    assert!(execution_result.is_ok());
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
}
