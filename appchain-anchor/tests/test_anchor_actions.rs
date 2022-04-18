use std::collections::HashMap;

use appchain_anchor::{types::AppchainState, AppchainAnchorContract};
use mock_appchain_registry::MockAppchainRegistryContract;
use mock_oct_token::MockOctTokenContract;
use near_sdk::json_types::{U128, U64};
use near_sdk_sim::{call, ContractAccount, UserAccount};
use wrapped_appchain_token::WrappedAppchainTokenContract;

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
fn test_anchor_actions() {
    let (
        root,
        oct_token,
        wrapped_appchain_token,
        registry,
        anchor,
        users,
        mut appchain_message_nonce,
    ) = test_normal_actions(false, true);
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

fn test_normal_actions(
    with_old_anchor: bool,
    to_confirm_view_result: bool,
) -> (
    UserAccount,
    ContractAccount<MockOctTokenContract>,
    ContractAccount<WrappedAppchainTokenContract>,
    ContractAccount<MockAppchainRegistryContract>,
    ContractAccount<AppchainAnchorContract>,
    Vec<UserAccount>,
    u32,
) {
    let total_supply = common::to_oct_amount(TOTAL_SUPPLY);
    let (root, oct_token, registry, anchor, users) = common::init(total_supply, with_old_anchor);
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
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(anchor_status.total_stake_in_next_era.0, 0);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 0);
    }
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
    if to_confirm_view_result {
        common::print_wrapped_appchain_token_info(&anchor);
    }
    //
    // user0 register validator (error)
    //
    let user0_balance = token_viewer::get_oct_balance_of(&users[0], &oct_token);
    let amount0 = common::to_oct_amount(9999);
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
        user0_balance.0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(anchor_status.total_stake_in_next_era.0, 0);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 0);
    }
    //
    // user0 register validator
    //
    let user0_balance = token_viewer::get_oct_balance_of(&users[0], &oct_token);
    let amount0 = common::to_oct_amount(23_000);
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
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(anchor_status.total_stake_in_next_era.0, amount0);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 1);
        common::print_validator_profile(&anchor, &users[0].account_id(), &user0_id_in_appchain);
    }
    //
    // user1 register validator
    //
    let user1_balance = token_viewer::get_oct_balance_of(&users[1], &oct_token);
    let amount1 = common::to_oct_amount(25_000);
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
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(anchor_status.total_stake_in_next_era.0, amount0 + amount1);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
        common::print_validator_profile(&anchor, &users[1].account_id(), &user1_id_in_appchain);
    }
    //
    // user2 register delegator to user0 (error)
    //
    let user2_balance = token_viewer::get_oct_balance_of(&users[2], &oct_token);
    let amount2 = common::to_oct_amount(499);
    let result = staking_actions::register_delegator(
        &users[2],
        &oct_token,
        &anchor,
        &users[0].account_id(),
        amount2,
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[2], &oct_token).0,
        user2_balance.0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(anchor_status.total_stake_in_next_era.0, amount0 + amount1);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
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
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
    //
    // user2 register delegator to user1 (error)
    //
    let user2_balance = token_viewer::get_oct_balance_of(&users[2], &oct_token);
    let amount2_1 = common::to_oct_amount(1000);
    let result = staking_actions::register_delegator(
        &users[2],
        &oct_token,
        &anchor,
        &users[1].account_id(),
        amount2_1,
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[2], &oct_token).0,
        user2_balance.0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
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
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0 + amount3_0
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
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
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0 + amount3_0 + amount0_p
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
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
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0 + amount3_0 + amount0_p + amount2_0_p
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
    //
    // Print anchor status and staking histories
    //
    if to_confirm_view_result {
        common::print_anchor_status(&anchor);
        common::print_wrapped_appchain_token_info(&anchor);
        common::print_staking_histories(&anchor);
        common::print_validator_list_of(&anchor, None);
    }
    //
    // Try go_booting
    //
    let result = lifecycle_actions::go_booting(&root, &anchor);
    assert!(!result.is_ok());
    //
    // Set appchain settings and try go_booting
    //
    let result = settings_actions::set_rpc_endpoint(&root, &anchor, "rpc_endpoint".to_string());
    result.assert_success();
    let result = settings_actions::set_subql_endpoint(&root, &anchor, "subql_endpoint".to_string());
    result.assert_success();
    let result = settings_actions::set_era_reward(&root, &anchor, common::to_oct_amount(10));
    result.assert_success();
    let result = lifecycle_actions::go_booting(&root, &anchor);
    assert!(!result.is_ok());
    //
    // Change protocol settings and try go_booting
    //
    let result = settings_actions::change_minimum_validator_count(&root, &anchor, 1);
    result.assert_success();
    let result = lifecycle_actions::go_booting(&root, &anchor);
    assert!(!result.is_ok());
    //
    // Change price of OCT token and try go_booting
    //
    let result = settings_actions::set_price_of_oct_token(&users[4], &anchor, 2_130_000);
    result.assert_success();
    let result = lifecycle_actions::go_booting(&root, &anchor);
    result.assert_success();
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor),
        AppchainState::Booting
    );
    //
    // Check validator set of era0
    //
    let mut appchain_message_nonce: u32 = 0;
    if to_confirm_view_result {
        common::print_anchor_status(&anchor);
        common::print_staking_histories(&anchor);
        common::print_validator_set_info_of(&anchor, U64::from(0));
        common::print_validator_list_of(&anchor, Some(0));
        common::print_delegator_list_of(&anchor, 0, &users[0]);
    }
    //
    // Initialize beefy light client
    //
    let result =
        lifecycle_actions::initialize_beefy_light_client(&root, &anchor, vec!["0x00".to_string()]);
    result.assert_success();
    //
    // Go live
    //
    let result = lifecycle_actions::go_live(&root, &anchor);
    result.assert_success();
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor),
        AppchainState::Active
    );
    //
    // Change id in appchain and profile of user0, user1
    //
    let result =
        validator_actions::set_validator_id_in_appchain(&users[0], &anchor, &user0_id_in_appchain);
    result.assert_success();
    let result = validator_actions::set_validator_profile(&users[0], &anchor, &user0_profile);
    result.assert_success();
    if to_confirm_view_result {
        common::print_validator_profile(&anchor, &users[0].account_id(), &user0_id_in_appchain);
    }
    let result =
        validator_actions::set_validator_id_in_appchain(&users[1], &anchor, &user1_id_in_appchain);
    result.assert_success();
    let result = validator_actions::set_validator_profile(&users[1], &anchor, &user1_profile);
    result.assert_success();
    if to_confirm_view_result {
        common::print_validator_profile(&anchor, &users[1].account_id(), &user1_id_in_appchain);
    }
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
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0 + amount3_0 + amount0_p + amount2_0_p + amount4
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 3);
        common::print_validator_profile(&anchor, &users[4].account_id(), &user4_id_in_appchain);
    }
    //
    // Print staking histories
    //
    if to_confirm_view_result {
        common::print_staking_histories(&anchor);
    }
    //
    //
    //
    (
        root,
        oct_token,
        wrapped_appchain_token,
        registry,
        anchor,
        users,
        appchain_message_nonce,
    )
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
    let (root, _, _, _, anchor, users, _) = test_normal_actions(true, false);
    common::deploy_new_anchor_contract(&anchor);
    let result = call!(root, anchor.migrate_state());
    common::print_execution_result("migrate_state", &result);
    //
    //
    //
    common::print_anchor_status(&anchor);
    common::print_wrapped_appchain_token_info(&anchor);
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
