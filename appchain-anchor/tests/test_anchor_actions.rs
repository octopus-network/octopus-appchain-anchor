use std::convert::TryInto;

use appchain_anchor::{
    types::{
        AnchorSettings, AnchorStatus, AppchainSettings, AppchainState, ProtocolSettings,
        ValidatorSetInfo, ValidatorSetProcessingStatus,
    },
    AppchainAnchorContract, AppchainEvent, AppchainMessage,
};
use near_sdk::{json_types::U64, serde_json};
use near_sdk_sim::{outcome, ContractAccount, UserAccount};

mod anchor_viewer;
mod common;
mod lifecycle_actions;
mod oct_token_viewer;
mod permissionless_actions;
mod settings_actions;
mod staking_actions;
mod sudo_actions;

const TOTAL_SUPPLY: u128 = 100_000_000;

#[test]
fn test_staging_actions() {
    let total_supply = common::to_oct_amount(TOTAL_SUPPLY);
    let (root, oct_token, registry, anchor, users) = common::init(total_supply);
    //
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor),
        AppchainState::Staging
    );
    //
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
    //
    assert_eq!(
        protocol_settings.minimum_validator_deposit.0,
        common::to_oct_amount(10_000)
    );
    //
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    assert_eq!(anchor_status.total_stake_in_next_era.0, 0);
    assert_eq!(anchor_status.validator_count_in_next_era.0, 0);
    //
    // user0 register validator (error)
    //
    let user0_balance = oct_token_viewer::get_ft_balance_of(&users[0], &oct_token);
    let amount0 = common::to_oct_amount(9999);
    let user0_id_in_appchain = "user0_id_in_appchain".to_string();
    let outcome = staking_actions::register_validator(
        &users[0],
        &oct_token,
        &anchor,
        &user0_id_in_appchain,
        amount0,
        true,
    );
    outcome.assert_success();
    assert_eq!(
        oct_token_viewer::get_ft_balance_of(&users[0], &oct_token).0,
        user0_balance.0
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    assert_eq!(anchor_status.total_stake_in_next_era.0, 0);
    assert_eq!(anchor_status.validator_count_in_next_era.0, 0);
    //
    // user0 register validator
    //
    let user0_balance = oct_token_viewer::get_ft_balance_of(&users[0], &oct_token);
    let amount0 = common::to_oct_amount(10_000);
    let user0_id_in_appchain = "user0_id_in_appchain".to_string();
    let outcome = staking_actions::register_validator(
        &users[0],
        &oct_token,
        &anchor,
        &user0_id_in_appchain,
        amount0,
        true,
    );
    outcome.assert_success();
    assert_eq!(
        oct_token_viewer::get_ft_balance_of(&users[0], &oct_token).0,
        user0_balance.0 - amount0
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    assert_eq!(anchor_status.total_stake_in_next_era.0, amount0);
    assert_eq!(anchor_status.validator_count_in_next_era.0, 1);
    //
    // user1 register validator
    //
    let user1_balance = oct_token_viewer::get_ft_balance_of(&users[1], &oct_token);
    let amount1 = common::to_oct_amount(15_000);
    let user1_id_in_appchain = "user1_id_in_appchain".to_string();
    let outcome = staking_actions::register_validator(
        &users[1],
        &oct_token,
        &anchor,
        &user1_id_in_appchain,
        amount1,
        false,
    );
    outcome.assert_success();
    assert_eq!(
        oct_token_viewer::get_ft_balance_of(&users[1], &oct_token).0,
        user1_balance.0 - amount1
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    assert_eq!(anchor_status.total_stake_in_next_era.0, amount0 + amount1);
    assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    //
    // user2 register delegator to user0 (error)
    //
    let user2_balance = oct_token_viewer::get_ft_balance_of(&users[2], &oct_token);
    let amount2 = common::to_oct_amount(999);
    let outcome = staking_actions::register_delegator(
        &users[2],
        &oct_token,
        &anchor,
        &users[0].account_id(),
        amount2,
    );
    outcome.assert_success();
    assert_eq!(
        oct_token_viewer::get_ft_balance_of(&users[2], &oct_token).0,
        user2_balance.0
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    assert_eq!(anchor_status.total_stake_in_next_era.0, amount0 + amount1);
    assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    //
    // user2 register delegator to user0
    //
    let user2_balance = oct_token_viewer::get_ft_balance_of(&users[2], &oct_token);
    let amount2_0 = common::to_oct_amount(1000);
    let outcome = staking_actions::register_delegator(
        &users[2],
        &oct_token,
        &anchor,
        &users[0].account_id(),
        amount2_0,
    );
    outcome.assert_success();
    assert_eq!(
        oct_token_viewer::get_ft_balance_of(&users[2], &oct_token).0,
        user2_balance.0 - amount2_0
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    assert_eq!(
        anchor_status.total_stake_in_next_era.0,
        amount0 + amount1 + amount2_0
    );
    assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    //
    // user2 register delegator to user1 (error)
    //
    let user2_balance = oct_token_viewer::get_ft_balance_of(&users[2], &oct_token);
    let amount2_1 = common::to_oct_amount(1000);
    let outcome = staking_actions::register_delegator(
        &users[2],
        &oct_token,
        &anchor,
        &users[1].account_id(),
        amount2_1,
    );
    outcome.assert_success();
    assert_eq!(
        oct_token_viewer::get_ft_balance_of(&users[2], &oct_token).0,
        user2_balance.0
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    assert_eq!(
        anchor_status.total_stake_in_next_era.0,
        amount0 + amount1 + amount2_0
    );
    assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    //
    // user3 register delegator to user0
    //
    let user3_balance = oct_token_viewer::get_ft_balance_of(&users[3], &oct_token);
    let amount3_0 = common::to_oct_amount(2000);
    let outcome = staking_actions::register_delegator(
        &users[3],
        &oct_token,
        &anchor,
        &users[0].account_id(),
        amount3_0,
    );
    outcome.assert_success();
    assert_eq!(
        oct_token_viewer::get_ft_balance_of(&users[3], &oct_token).0,
        user3_balance.0 - amount3_0
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    assert_eq!(
        anchor_status.total_stake_in_next_era.0,
        amount0 + amount1 + amount2_0 + amount3_0
    );
    assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    //
    // user0 increase stake
    //
    let user0_balance = oct_token_viewer::get_ft_balance_of(&users[0], &oct_token);
    let amount0_p = common::to_oct_amount(1_200);
    let outcome = staking_actions::increase_stake(&users[0], &oct_token, &anchor, amount0_p);
    outcome.assert_success();
    assert_eq!(
        oct_token_viewer::get_ft_balance_of(&users[0], &oct_token).0,
        user0_balance.0 - amount0_p
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    assert_eq!(
        anchor_status.total_stake_in_next_era.0,
        amount0 + amount1 + amount2_0 + amount3_0 + amount0_p
    );
    assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    //
    // user2 increase delegation to user0
    //
    let user2_balance = oct_token_viewer::get_ft_balance_of(&users[2], &oct_token);
    let amount2_0_p = common::to_oct_amount(500);
    let outcome = staking_actions::increase_delegation(
        &users[2],
        &oct_token,
        &anchor,
        &users[0].account_id(),
        amount2_0_p,
    );
    outcome.assert_success();
    assert_eq!(
        oct_token_viewer::get_ft_balance_of(&users[2], &oct_token).0,
        user2_balance.0 - amount2_0_p
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    assert_eq!(
        anchor_status.total_stake_in_next_era.0,
        amount0 + amount1 + amount2_0 + amount3_0 + amount0_p + amount2_0_p
    );
    assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    //
    // Get staking histories
    //
    for i in 0..6 {
        let staking_history = anchor_viewer::get_staking_history(&anchor, i.try_into().unwrap());
        println!(
            "Staking history {}: {}",
            i,
            serde_json::to_string(&staking_history).unwrap()
        );
    }
    //
    // Try go_booting
    //
    let outcome = lifecycle_actions::go_booting(&root, &anchor);
    assert!(!outcome.is_ok());
    //
    // Set appchain settings and try go_booting
    //
    let outcome = settings_actions::set_chain_spec(&root, &anchor, "chain_spec".to_string());
    outcome.assert_success();
    let outcome =
        settings_actions::set_raw_chain_spec(&root, &anchor, "raw_chain_spec".to_string());
    outcome.assert_success();
    let outcome = settings_actions::set_boot_nodes(&root, &anchor, "boot_nodes".to_string());
    outcome.assert_success();
    let outcome = settings_actions::set_rpc_endpoint(&root, &anchor, "rpc_endpoint".to_string());
    outcome.assert_success();
    let outcome = settings_actions::set_era_reward(&root, &anchor, common::to_oct_amount(10));
    outcome.assert_success();
    let outcome = lifecycle_actions::go_booting(&root, &anchor);
    assert!(!outcome.is_ok());
    //
    // Change protocol settings and try go_booting
    //
    let outcome = settings_actions::change_minimum_validator_count(&root, &anchor, 2);
    outcome.assert_success();
    let outcome = lifecycle_actions::go_booting(&root, &anchor);
    assert!(!outcome.is_ok());
    //
    // Change price of OCT token and try go_booting
    //
    let outcome = settings_actions::set_price_of_oct_token(&users[4], &anchor, 2_130_000);
    assert!(!outcome.is_ok());
    let outcome = settings_actions::set_token_price_maintainer_account(&root, &anchor, &users[4]);
    outcome.assert_success();
    let outcome = settings_actions::set_price_of_oct_token(&users[4], &anchor, 2_130_000);
    outcome.assert_success();
    let outcome = lifecycle_actions::go_booting(&root, &anchor);
    assert!(!outcome.is_ok());
    //
    // Change total stake price and try go_booting
    //
    let outcome = settings_actions::change_minimum_total_stake_price_for_booting(
        &root,
        &anchor,
        common::to_oct_amount(6_000_000_000),
    );
    outcome.assert_success();
    let outcome = lifecycle_actions::go_booting(&root, &anchor);
    outcome.assert_success();
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor),
        AppchainState::Booting
    );
    let processing_status = anchor_viewer::get_processing_status_of(&anchor, 0);
    println!(
        "Processing status of era {}: {}",
        0,
        serde_json::to_string::<ValidatorSetProcessingStatus>(&processing_status).unwrap()
    );
    //
    // Try complete switching era0
    //
    switch_era(&root, &anchor, 0);
    print_validator_list_of(&anchor, 0);
    print_delegator_list_of(&anchor, 0, &users[0]);
    //
    // Go live
    //
    let outcome = lifecycle_actions::go_live(&root, &anchor);
    outcome.assert_success();
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor),
        AppchainState::Active
    );
    //
    // Distribut reward of era0
    //
    distribute_reward_of(&root, &anchor, 0);
    print_validator_reward_histories(&anchor, &users[0], 0);
    print_validator_reward_histories(&anchor, &users[1], 0);
    print_delegator_reward_histories(&anchor, &users[2], &users[0], 0);
    print_delegator_reward_histories(&anchor, &users[3], &users[0], 0);
    //
    // user4 register validator
    //
    let user4_balance = oct_token_viewer::get_ft_balance_of(&users[4], &oct_token);
    let amount4 = common::to_oct_amount(13_000);
    let user4_id_in_appchain = "user4_id_in_appchain".to_string();
    let outcome = staking_actions::register_validator(
        &users[4],
        &oct_token,
        &anchor,
        &user4_id_in_appchain,
        amount4,
        true,
    );
    outcome.assert_success();
    assert_eq!(
        oct_token_viewer::get_ft_balance_of(&users[4], &oct_token).0,
        user4_balance.0 - amount4
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    assert_eq!(
        anchor_status.total_stake_in_next_era.0,
        amount0 + amount1 + amount2_0 + amount3_0 + amount0_p + amount2_0_p + amount4
    );
    assert_eq!(anchor_status.validator_count_in_next_era.0, 3);
    //
    // Try start and complete switching era1
    //
    switch_era(&root, &anchor, 1);
    print_validator_list_of(&anchor, 1);
    print_delegator_list_of(&anchor, 1, &users[0]);
    //
    // Distribut reward of era0
    //
    distribute_reward_of(&root, &anchor, 1);
    print_validator_reward_histories(&anchor, &users[0], 1);
    print_validator_reward_histories(&anchor, &users[1], 1);
    print_delegator_reward_histories(&anchor, &users[2], &users[0], 1);
    print_delegator_reward_histories(&anchor, &users[3], &users[0], 1);
    print_validator_reward_histories(&anchor, &users[4], 1);
    //
    // user1 decrease stake
    //
    let outcome = staking_actions::decrease_stake(&users[1], &anchor, common::to_oct_amount(1000));
    outcome.assert_success();
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(&anchor, &users[1]);
    assert!(unbonded_stakes.len() == 0);
    //
    // user2 decrease delegation
    //
    let outcome = staking_actions::decrease_delegation(
        &users[2],
        &anchor,
        &users[0].valid_account_id().to_string(),
        common::to_oct_amount(200),
    );
    outcome.assert_success();
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(&anchor, &users[2]);
    assert!(unbonded_stakes.len() == 0);
    //
    // Try start and complete switching era2
    //
    switch_era(&root, &anchor, 2);
    print_validator_list_of(&anchor, 2);
    print_delegator_list_of(&anchor, 2, &users[0]);
    //
    // Distribute reward of era2
    //
    distribute_reward_of(&root, &anchor, 2);
    print_validator_reward_histories(&anchor, &users[1], 2);
    print_delegator_reward_histories(&anchor, &users[2], &users[0], 2);
    print_delegator_reward_histories(&anchor, &users[3], &users[0], 2);
    print_validator_reward_histories(&anchor, &users[4], 2);
    print_unbonded_stakes_of(&anchor, &users[1]);
    print_unbonded_stakes_of(&anchor, &users[2]);
}

fn switch_era(
    root: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: u64,
) {
    if era_number > 0 {
        let outcome = sudo_actions::apply_appchain_message(
            root,
            anchor,
            AppchainMessage {
                appchain_event: AppchainEvent::EraSwitchPlaned {
                    era_number: U64::from(era_number),
                },
                block_height: U64::from(era_number + 1),
                timestamp: U64::from(era_number + 1),
                nonce: (era_number + 1).try_into().unwrap(),
            },
        );
        outcome.assert_success();
        let processing_status = anchor_viewer::get_processing_status_of(anchor, era_number);
        println!(
            "Processing status of era {}: {}",
            era_number,
            serde_json::to_string::<ValidatorSetProcessingStatus>(&processing_status).unwrap()
        );
    }
    loop {
        let outcome = permissionless_actions::try_complete_switching_era(root, &anchor);
        println!(
            "Try complete switching era: {}",
            outcome.unwrap_json_value().as_bool().unwrap()
        );
        let processing_status = anchor_viewer::get_processing_status_of(anchor, era_number);
        println!(
            "Processing status of era {}: {}",
            era_number,
            serde_json::to_string::<ValidatorSetProcessingStatus>(&processing_status).unwrap()
        );
        if outcome.unwrap_json_value().as_bool().unwrap() {
            break;
        }
    }
    let anchor_status = anchor_viewer::get_anchor_status(anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    let validator_set_info = anchor_viewer::get_validator_set_info_of(anchor, era_number);
    println!(
        "Validator set info of era {}: {}",
        era_number,
        serde_json::to_string::<ValidatorSetInfo>(&validator_set_info).unwrap()
    );
}

fn print_validator_list_of(anchor: &ContractAccount<AppchainAnchorContract>, era_number: u64) {
    let validator_list = anchor_viewer::get_validator_list_of_era(anchor, era_number);
    let mut index = 0;
    for validator in validator_list {
        println!(
            "Validator {} in era {}: {}",
            index,
            era_number,
            serde_json::to_string(&validator).unwrap()
        );
        index += 1;
    }
}

fn print_delegator_list_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: u64,
    validator: &UserAccount,
) {
    let delegator_list =
        anchor_viewer::get_delegators_of_validator_in_era(&anchor, era_number, validator);
    let mut index = 0;
    for delegator in delegator_list {
        println!(
            "Delegator {} of {} in era {}: {}",
            index,
            validator.valid_account_id().to_string(),
            era_number,
            serde_json::to_string(&delegator).unwrap()
        );
        index += 1;
    }
}

fn distribute_reward_of(
    root: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: u64,
) {
    let outcome = sudo_actions::apply_appchain_message(
        root,
        anchor,
        AppchainMessage {
            appchain_event: AppchainEvent::EraRewardConcluded {
                era_number: U64::from(era_number),
                unprofitable_validator_ids: Vec::new(),
            },
            block_height: U64::from(era_number + 1),
            timestamp: U64::from(era_number + 1),
            nonce: (era_number + 1).try_into().unwrap(),
        },
    );
    outcome.assert_success();
    let anchor_status = anchor_viewer::get_anchor_status(anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    loop {
        let outcome = permissionless_actions::try_complete_distributing_reward(root, anchor);
        println!(
            "Try complete switching era: {}",
            outcome.unwrap_json_value().as_bool().unwrap()
        );
        let processing_status = anchor_viewer::get_processing_status_of(anchor, era_number);
        println!(
            "Processing status of era {}: {}",
            era_number,
            serde_json::to_string::<ValidatorSetProcessingStatus>(&processing_status).unwrap()
        );
        if outcome.unwrap_json_value().as_bool().unwrap() {
            break;
        }
    }
    let anchor_status = anchor_viewer::get_anchor_status(anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    let validator_set_info = anchor_viewer::get_validator_set_info_of(anchor, era_number);
    println!(
        "Validator set info of era {}: {}",
        era_number,
        serde_json::to_string::<ValidatorSetInfo>(&validator_set_info).unwrap()
    );
}

fn print_validator_reward_histories(
    anchor: &ContractAccount<AppchainAnchorContract>,
    validator: &UserAccount,
    end_era: u64,
) {
    let reward_histories = anchor_viewer::get_validator_rewards_of(anchor, 0, end_era, validator);
    let mut index = 0;
    for reward_history in reward_histories {
        println!(
            "Reward history {} of {}: {}",
            index,
            validator.account_id().to_string(),
            serde_json::to_string(&reward_history).unwrap()
        );
        index += 1;
    }
}

fn print_delegator_reward_histories(
    anchor: &ContractAccount<AppchainAnchorContract>,
    delegator: &UserAccount,
    validator: &UserAccount,
    end_era: u64,
) {
    let reward_histories =
        anchor_viewer::get_delegator_rewards_of(anchor, 0, end_era, delegator, validator);
    let mut index = 0;
    for reward_history in reward_histories {
        println!(
            "Reward history {} of {} to {}: {}",
            index,
            delegator.account_id().to_string(),
            validator.account_id().to_string(),
            serde_json::to_string(&reward_history).unwrap()
        );
        index += 1;
    }
}

fn print_unbonded_stakes_of(anchor: &ContractAccount<AppchainAnchorContract>, user: &UserAccount) {
    let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(anchor, user);
    let mut index = 0;
    for unbonded_stake in unbonded_stakes {
        println!(
            "Unbonded stake {} of {}: {}",
            index,
            user.valid_account_id().to_string(),
            serde_json::to_string(&unbonded_stake).unwrap()
        );
        index += 1;
    }
}
