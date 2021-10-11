use std::convert::TryInto;

use appchain_anchor::types::{
    AnchorSettings, AnchorStatus, AppchainSettings, AppchainState, ProtocolSettings,
};
use near_sdk::serde_json;

mod anchor_viewer;
mod common;
mod lifecycle_actions;
mod oct_token_viewer;
mod permissionless_actions;
mod settings_actions;
mod staking_actions;

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
    let outcome = settings_actions::set_era_reward(&root, &anchor, common::to_oct_amount(100));
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
    //
    // Try complete switching era0
    //
}
