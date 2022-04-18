use std::collections::HashMap;

use appchain_anchor::{
    types::{
        AnchorSettings, AppchainSettings, AppchainState, ProtocolSettings, WrappedAppchainToken,
    },
    AppchainEvent, AppchainMessage,
};
use near_sdk::{json_types::U128, serde_json};

mod anchor_viewer;
mod common;
mod lifecycle_actions;
mod permissionless_actions;
mod settings_actions;
mod staking_actions;
mod sudo_actions;
mod token_viewer;
mod wrapped_appchain_token_manager;

const TOTAL_SUPPLY: u128 = 100_000_000;

#[test]
fn test_wrapped_appchain_token_bridging() {
    let total_supply = common::to_oct_amount(TOTAL_SUPPLY);
    let (root, oct_token, _registry, anchor, users) = common::init(total_supply, false);
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    let user1_id_in_appchain =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da270".to_string();
    let user4_id_in_appchain =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da273".to_string();
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
    let result =
        wrapped_appchain_token_manager::set_price_of_wrapped_appchain_token(&users[4], &anchor, 0);
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
    let wrapped_appchain_token_info = anchor_viewer::get_wrapped_appchain_token(&anchor);
    println!(
        "Wrapped appchain token: {}",
        serde_json::to_string::<WrappedAppchainToken>(&wrapped_appchain_token_info).unwrap()
    );
    //
    // user0 register validator
    //
    let user0_balance = token_viewer::get_oct_balance_of(&users[0], &oct_token);
    let amount0 = common::to_oct_amount(10_000);
    let result = staking_actions::register_validator(
        &users[0],
        &oct_token,
        &anchor,
        &Some(user0_id_in_appchain.clone()),
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
    //
    // user1 register validator
    //
    let user1_balance = token_viewer::get_oct_balance_of(&users[1], &oct_token);
    let amount1 = common::to_oct_amount(15_000);
    let result = staking_actions::register_validator(
        &users[1],
        &oct_token,
        &anchor,
        &Some(user1_id_in_appchain.clone()),
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
    common::print_staking_histories(&anchor);
    common::print_validator_list_of(&anchor, None);
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
    // Mint wrapped appchain token for user1 (error)
    //
    let user1_wat_balance =
        token_viewer::get_wat_balance_of(&users[1].valid_account_id(), &wrapped_appchain_token);
    let mut appchain_messages = Vec::<AppchainMessage>::new();
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: "unknown.testnet".to_string(),
            amount: U128::from(total_supply / 10),
        },
        nonce: appchain_message_nonce,
    });
    sudo_actions::stage_appchain_messages(&root, &anchor, appchain_messages);
    common::process_appchain_messages(&users[4], &anchor);
    common::print_appchain_messages_processing_results(&anchor);
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
    assert_eq!(
        token_viewer::get_wat_balance_of(&users[1].valid_account_id(), &wrapped_appchain_token).0,
        user1_wat_balance.0
    );
    //
    // Burn wrapped appchain token from user0
    //
    let result = wrapped_appchain_token_manager::burn_wrapped_appchain_token(
        &users[0],
        &anchor,
        user0_id_in_appchain,
        total_supply / 2 - common::to_oct_amount(50000),
    );
    result.assert_success();
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
    common::print_wrapped_appchain_token_info(&anchor);
    //
    // Mint wrapped appchain token for user1
    //
    let user1_wat_balance =
        token_viewer::get_wat_balance_of(&users[1].valid_account_id(), &wrapped_appchain_token);
    let mut appchain_messages = Vec::<AppchainMessage>::new();
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].account_id(),
            amount: U128::from(common::to_oct_amount(60)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].account_id(),
            amount: U128::from(common::to_oct_amount(40)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraSwitchPlaned { era_number: 1 },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].account_id(),
            amount: U128::from(common::to_oct_amount(70)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].account_id(),
            amount: U128::from(common::to_oct_amount(30)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].account_id(),
            amount: U128::from(common::to_oct_amount(45)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].account_id(),
            amount: U128::from(common::to_oct_amount(45)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].account_id(),
            amount: U128::from(common::to_oct_amount(45)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].account_id(),
            amount: U128::from(common::to_oct_amount(45)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraRewardConcluded {
            era_number: 0,
            unprofitable_validator_ids: Vec::new(),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].account_id(),
            amount: U128::from(common::to_oct_amount(45)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].account_id(),
            amount: U128::from(common::to_oct_amount(45)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].account_id(),
            amount: U128::from(common::to_oct_amount(45)),
        },
        nonce: appchain_message_nonce,
    });
    sudo_actions::stage_appchain_messages(&root, &anchor, appchain_messages);
    common::process_appchain_messages(&users[3], &anchor);
    common::print_appchain_messages_processing_results(&anchor);
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
    common::print_wrapped_appchain_token_info(&anchor);
    assert_eq!(
        token_viewer::get_wat_balance_of(&users[1].valid_account_id(), &wrapped_appchain_token).0,
        user1_wat_balance.0 + common::to_oct_amount(515)
    );
    //
    //
    //
    let mut appchain_messages = Vec::<AppchainMessage>::new();
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraSwitchPlaned { era_number: 2 },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraRewardConcluded {
            era_number: 1,
            unprofitable_validator_ids: Vec::new(),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraSwitchPlaned { era_number: 3 },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraRewardConcluded {
            era_number: 2,
            unprofitable_validator_ids: Vec::new(),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraRewardConcluded {
            era_number: 1,
            unprofitable_validator_ids: Vec::new(),
        },
        nonce: appchain_message_nonce,
    });
    sudo_actions::stage_appchain_messages(&root, &anchor, appchain_messages);
    common::process_appchain_messages(&users[3], &anchor);
    common::print_appchain_messages_processing_results(&anchor);
}
