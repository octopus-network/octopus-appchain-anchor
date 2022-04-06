use std::collections::HashMap;

use near_sdk::json_types::U64;
use near_sdk_sim::{ContractAccount, UserAccount};

use appchain_anchor::AppchainAnchorContract;
use mock_oct_token::MockOctTokenContract;
use wrapped_appchain_token::WrappedAppchainTokenContract;

use crate::{anchor_viewer, common, settings_manager, staking_actions, sudo_actions};

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
