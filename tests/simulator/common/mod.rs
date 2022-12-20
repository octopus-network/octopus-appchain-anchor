pub mod basic_actions;
pub mod complex_actions;
pub mod complex_viewer;

use crate::contract_interfaces::{
    anchor_viewer, lifecycle_actions, settings_manager, staking_actions, validator_actions,
    wrapped_appchain_token_manager,
};
use appchain_anchor::types::AppchainState;
use near_sdk::json_types::U128;
use near_sdk::json_types::U64;
use near_sdk::serde_json::json;
use near_sdk::AccountId;
use std::collections::HashMap;
use std::str::FromStr;
use workspaces::network::Sandbox;
use workspaces::Worker;
use workspaces::{error::Error, result::ExecutionFinalResult, Account, Contract};

const TOTAL_SUPPLY: u128 = 100_000_000;

pub async fn call_ft_transfer(
    sender: &Account,
    receiver: &Account,
    amount: u128,
    ft_token_contract: &Contract,
) -> Result<ExecutionFinalResult, Error> {
    sender
        .call(ft_token_contract.id(), "ft_transfer")
        .args_json(json!({
            "receiver_id": receiver.id(),
            "amount": U128::from(amount),
            "memo": Option::<String>::None,
        }))
        .gas(20_000_000_000_000)
        .deposit(1)
        .transact()
        .await
}

pub async fn call_ft_transfer_call(
    sender: &Account,
    receiver: &Account,
    amount: u128,
    msg: String,
    ft_token_contract: &Contract,
) -> Result<ExecutionFinalResult, Error> {
    sender
        .call(ft_token_contract.id(), "ft_transfer_call")
        .args_json(json!({
            "receiver_id": receiver.id(),
            "amount": U128::from(amount),
            "memo": Option::<String>::None,
            "msg": msg.clone(),
        }))
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await
}

pub async fn get_ft_balance_of(user: &Account, ft_contract: &Contract) -> Result<U128, Error> {
    ft_contract
        .call("ft_balance_of")
        .args_json(json!({
            "account_id": user.id()
        }))
        .view()
        .await?
        .json::<U128>()
}

pub fn to_actual_amount(amount: u128, decimals: u32) -> u128 {
    let bt_decimals_base = (10 as u128).pow(decimals);
    amount * bt_decimals_base
}

pub async fn test_normal_actions(
    worker: &Worker<Sandbox>,
    with_old_anchor: bool,
    to_confirm_view_result: bool,
) -> anyhow::Result<(
    Account,
    Contract,
    Contract,
    Contract,
    Contract,
    Contract,
    Contract,
    Vec<Account>,
    u32,
)> {
    let total_supply = to_actual_amount(TOTAL_SUPPLY, 18);
    let (root, oct_token, registry, council, anchor, wat_faucet, users) =
        basic_actions::initialize_contracts_and_users(worker, total_supply, with_old_anchor)
            .await?;
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
        anchor_viewer::get_appchain_state(&anchor).await?,
        AppchainState::Booting
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor).await?;
        assert_eq!(anchor_status.total_stake_in_next_era.0, 0);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 0);
    }
    //
    //
    //
    assert!(
        settings_manager::set_price_of_oct_token(&users[4], &anchor, 2_130_000)
            .await
            .unwrap()
            .is_failure()
    );
    assert!(
        wrapped_appchain_token_manager::set_price_of_wrapped_appchain_token(
            &users[4], &anchor, 110_000,
        )
        .await
        .unwrap()
        .is_failure()
    );
    assert!(
        settings_manager::set_token_price_maintainer_account(&root, &anchor, &users[4])
            .await
            .unwrap()
            .is_success()
    );
    //
    // Initialize wrapped appchain token contract.
    //
    assert!(
        wrapped_appchain_token_manager::set_price_of_wrapped_appchain_token(
            &users[4], &anchor, 110
        )
        .await
        .unwrap()
        .is_success()
    );
    assert!(
        wrapped_appchain_token_manager::set_account_of_wrapped_appchain_token(
            &root,
            &anchor,
            AccountId::from_str(format!("wrapped_appchain_token.{}", root.id()).as_str()).unwrap(),
        )
        .await
        .unwrap()
        .is_success()
    );
    let wrapped_appchain_token = basic_actions::deploy_wrapped_appchain_token_contract(
        &root,
        &anchor,
        &wat_faucet.as_account(),
        &U128::from(total_supply / 2 + to_actual_amount(10, 18)),
        &users,
    )
    .await
    .expect("Failed to deploy wrapped appchain token contract");
    if to_confirm_view_result {
        complex_viewer::print_wrapped_appchain_token_info(&anchor).await;
    }
    //
    assert!(call_ft_transfer(
        &wat_faucet.as_account(),
        &users[0],
        to_actual_amount(TOTAL_SUPPLY / 2, 18),
        &wrapped_appchain_token,
    )
    .await
    .unwrap()
    .is_success());
    //
    if !with_old_anchor {
        assert!(settings_manager::set_bonus_for_new_validator(
            &root,
            &anchor,
            to_actual_amount(1, 18)
        )
        .await
        .unwrap()
        .is_success());
    }
    //
    // user0 register validator (error)
    //
    let user0_balance = get_ft_balance_of(&users[0], &oct_token).await?;
    let amount0 = to_actual_amount(4999, 18);
    assert!(staking_actions::register_validator(
        &users[0],
        &oct_token,
        &anchor,
        &String::new(),
        amount0,
        true,
        HashMap::new(),
    )
    .await
    .unwrap()
    .is_success());
    assert_eq!(
        get_ft_balance_of(&users[0], &oct_token).await?.0,
        user0_balance.0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor).await?;
        assert_eq!(anchor_status.total_stake_in_next_era.0, 0);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 0);
    }
    //
    // user0 register validator
    //
    let user0_balance = get_ft_balance_of(&users[0], &oct_token).await?;
    let wat_faucet_balance =
        get_ft_balance_of(&wat_faucet.as_account(), &wrapped_appchain_token).await?;
    let amount0 = to_actual_amount(23_000, 18);
    assert!(staking_actions::register_validator(
        &users[0],
        &oct_token,
        &anchor,
        &user0_id_in_appchain,
        amount0,
        false,
        HashMap::new(),
    )
    .await
    .unwrap()
    .is_success());
    assert_eq!(
        get_ft_balance_of(&users[0], &oct_token).await?.0,
        user0_balance.0 - amount0
    );
    if !with_old_anchor {
        assert_eq!(
            get_ft_balance_of(&wat_faucet.as_account(), &wrapped_appchain_token)
                .await?
                .0,
            wat_faucet_balance.0
        );
    }
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor).await?;
        assert_eq!(anchor_status.total_stake_in_next_era.0, amount0);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 1);
        complex_viewer::print_validator_profile(
            &anchor,
            &users[0].id().to_string().parse().unwrap(),
            &user0_id_in_appchain,
        )
        .await;
    }
    //
    // user1 register validator
    //
    let user1_balance = get_ft_balance_of(&users[1], &oct_token).await?;
    let wat_faucet_balance =
        get_ft_balance_of(&wat_faucet.as_account(), &wrapped_appchain_token).await?;
    let amount1 = to_actual_amount(25_000, 18);
    assert!(staking_actions::register_validator(
        &users[1],
        &oct_token,
        &anchor,
        &user1_id_in_appchain,
        amount1,
        true,
        HashMap::new(),
    )
    .await
    .unwrap()
    .is_success());
    assert_eq!(
        get_ft_balance_of(&users[1], &oct_token).await?.0,
        user1_balance.0 - amount1
    );
    if !with_old_anchor {
        assert_eq!(
            get_ft_balance_of(&wat_faucet.as_account(), &wrapped_appchain_token)
                .await?
                .0,
            wat_faucet_balance.0
        );
    }
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor).await?;
        assert_eq!(anchor_status.total_stake_in_next_era.0, amount0 + amount1);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
        complex_viewer::print_validator_profile(
            &anchor,
            &users[1].id().to_string().parse().unwrap(),
            &user1_id_in_appchain,
        )
        .await;
    }
    //
    // user2 register delegator to user0 (error)
    //
    let user2_balance = get_ft_balance_of(&users[2], &oct_token).await?;
    let amount2 = to_actual_amount(199, 18);
    assert!(staking_actions::register_delegator(
        &users[2],
        &oct_token,
        &anchor,
        &users[0].id().to_string().parse().unwrap(),
        amount2,
    )
    .await
    .unwrap()
    .is_success());
    assert_eq!(
        get_ft_balance_of(&users[2], &oct_token).await?.0,
        user2_balance.0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor).await?;
        assert_eq!(anchor_status.total_stake_in_next_era.0, amount0 + amount1);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
    //
    // user1 enable delegation
    //
    assert!(validator_actions::enable_delegation(&users[0], &anchor)
        .await
        .unwrap()
        .is_success());
    //
    // user2 register delegator to user0
    //
    let user2_balance = get_ft_balance_of(&users[2], &oct_token).await?;
    let amount2_0 = to_actual_amount(1000, 18);
    assert!(staking_actions::register_delegator(
        &users[2],
        &oct_token,
        &anchor,
        &users[0].id().to_string().parse().unwrap(),
        amount2_0,
    )
    .await
    .unwrap()
    .is_success());
    assert_eq!(
        get_ft_balance_of(&users[2], &oct_token).await?.0,
        user2_balance.0 - amount2_0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor).await?;
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
    //
    // user1 disable delegation
    //
    assert!(validator_actions::disable_delegation(&users[1], &anchor)
        .await
        .unwrap()
        .is_success());
    //
    // user2 register delegator to user1 (error)
    //
    let user2_balance = get_ft_balance_of(&users[2], &oct_token).await?;
    let amount2_1 = to_actual_amount(1000, 18);
    assert!(staking_actions::register_delegator(
        &users[2],
        &oct_token,
        &anchor,
        &users[1].id().to_string().parse().unwrap(),
        amount2_1,
    )
    .await
    .unwrap()
    .is_success());
    assert_eq!(
        get_ft_balance_of(&users[2], &oct_token).await?.0,
        user2_balance.0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor).await?;
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
    //
    // user3 register delegator to user0
    //
    let user3_balance = get_ft_balance_of(&users[3], &oct_token).await?;
    let amount3_0 = to_actual_amount(2000, 18);
    assert!(staking_actions::register_delegator(
        &users[3],
        &oct_token,
        &anchor,
        &users[0].id().to_string().parse().unwrap(),
        amount3_0,
    )
    .await
    .unwrap()
    .is_success());
    assert_eq!(
        get_ft_balance_of(&users[3], &oct_token).await?.0,
        user3_balance.0 - amount3_0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor).await?;
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0 + amount3_0
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
    //
    // user0 increase stake
    //
    let user0_balance = get_ft_balance_of(&users[0], &oct_token).await?;
    let amount0_p = to_actual_amount(1_200, 18);
    assert!(
        staking_actions::increase_stake(&users[0], &oct_token, &anchor, amount0_p)
            .await
            .unwrap()
            .is_success()
    );
    assert_eq!(
        get_ft_balance_of(&users[0], &oct_token).await?.0,
        user0_balance.0 - amount0_p
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor).await?;
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0 + amount3_0 + amount0_p
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
    //
    // user2 increase delegation to user0
    //
    let user2_balance = get_ft_balance_of(&users[2], &oct_token).await?;
    let amount2_0_p = to_actual_amount(500, 18);
    assert!(staking_actions::increase_delegation(
        &users[2],
        &oct_token,
        &anchor,
        &users[0].id().to_string().parse().unwrap(),
        amount2_0_p,
    )
    .await
    .unwrap()
    .is_success());
    assert_eq!(
        get_ft_balance_of(&users[2], &oct_token).await?.0,
        user2_balance.0 - amount2_0_p
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor).await?;
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
        complex_viewer::print_anchor_status(&anchor).await;
        complex_viewer::print_wrapped_appchain_token_info(&anchor).await;
        complex_viewer::print_staking_histories(&anchor).await;
        complex_viewer::print_validator_list_of(&anchor, None).await;
    }
    //
    // Try generate_initial_validator_set
    //
    lifecycle_actions::generate_initial_validator_set(&root, &anchor)
        .await
        .unwrap()
        .is_failure();
    //
    // Set appchain settings and try generate_initial_validator_set
    //
    assert!(
        settings_manager::set_rpc_endpoint(&root, &anchor, "rpc_endpoint".to_string())
            .await
            .unwrap()
            .is_success()
    );
    assert!(
        settings_manager::set_subql_endpoint(&root, &anchor, "subql_endpoint".to_string())
            .await
            .unwrap()
            .is_success()
    );
    assert!(
        settings_manager::set_era_reward(&root, &anchor, to_actual_amount(10, 18))
            .await
            .unwrap()
            .is_success()
    );
    lifecycle_actions::generate_initial_validator_set(&root, &anchor)
        .await
        .unwrap()
        .is_failure();
    //
    // Change protocol settings and try generate_initial_validator_set
    //
    assert!(
        settings_manager::change_minimum_validator_count(&root, &anchor, 1)
            .await
            .unwrap()
            .is_success()
    );
    lifecycle_actions::generate_initial_validator_set(&root, &anchor)
        .await
        .unwrap()
        .is_failure();
    //
    // Change price of OCT token and try generate_initial_validator_set
    //
    assert!(
        settings_manager::set_price_of_oct_token(&users[4], &anchor, 2_130_000)
            .await
            .unwrap()
            .is_success()
    );
    lifecycle_actions::generate_initial_validator_set(&root, &anchor)
        .await
        .unwrap()
        .is_failure();
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor).await?,
        AppchainState::Booting
    );
    //
    // Check validator set of era0
    //
    let appchain_message_nonce: u32 = 0;
    if to_confirm_view_result {
        complex_viewer::print_anchor_status(&anchor).await;
        complex_viewer::print_staking_histories(&anchor).await;
        complex_viewer::print_validator_set_info_of(&anchor, U64::from(0)).await;
        complex_viewer::print_validator_list_of(&anchor, Some(0)).await;
        complex_viewer::print_delegator_list_of(&anchor, 0, &users[0]).await;
    }
    //
    // Initialization
    //
    assert!(settings_manager::turn_on_witness_mode(&root, &anchor)
        .await
        .unwrap()
        .is_success());
    assert!(
        settings_manager::set_relayer_account(&root, &anchor, &users[5])
            .await
            .unwrap()
            .is_success()
    );
    //
    // Go live
    //
    assert!(lifecycle_actions::go_live(&root, &anchor)
        .await
        .unwrap()
        .is_success());
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor).await?,
        AppchainState::Active
    );
    //
    // Change id in appchain and profile of user0, user1
    //
    assert!(validator_actions::set_validator_id_in_appchain(
        &users[0],
        &anchor,
        &user0_id_in_appchain
    )
    .await
    .unwrap()
    .is_success());
    assert!(
        validator_actions::set_validator_profile(&users[0], &anchor, &user0_profile)
            .await
            .unwrap()
            .is_success()
    );
    if to_confirm_view_result {
        complex_viewer::print_validator_profile(
            &anchor,
            &users[0].id().to_string().parse().unwrap(),
            &user0_id_in_appchain,
        )
        .await;
    }
    assert!(validator_actions::set_validator_id_in_appchain(
        &users[1],
        &anchor,
        &user1_id_in_appchain
    )
    .await
    .unwrap()
    .is_success());
    assert!(
        validator_actions::set_validator_profile(&users[1], &anchor, &user1_profile)
            .await
            .unwrap()
            .is_success()
    );
    if to_confirm_view_result {
        complex_viewer::print_validator_profile(
            &anchor,
            &users[1].id().to_string().parse().unwrap(),
            &user1_id_in_appchain,
        )
        .await;
    }
    //
    // user4 register validator
    //
    let user4_balance = get_ft_balance_of(&users[4], &oct_token).await?;
    let wat_faucet_balance =
        get_ft_balance_of(&wat_faucet.as_account(), &wrapped_appchain_token).await?;
    let amount4 = to_actual_amount(26_000, 18);
    assert!(staking_actions::register_validator(
        &users[4],
        &oct_token,
        &anchor,
        &user4_id_in_appchain,
        amount4,
        true,
        user4_profile,
    )
    .await
    .unwrap()
    .is_success());
    assert_eq!(
        get_ft_balance_of(&users[4], &oct_token).await?.0,
        user4_balance.0 - amount4
    );
    if !with_old_anchor {
        assert_eq!(
            get_ft_balance_of(&wat_faucet.as_account(), &wrapped_appchain_token)
                .await?
                .0,
            wat_faucet_balance.0 - to_actual_amount(1, 18)
        );
    }
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor).await?;
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0 + amount3_0 + amount0_p + amount2_0_p + amount4
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 3);
        complex_viewer::print_validator_profile(
            &anchor,
            &users[4].id().to_string().parse().unwrap(),
            &user4_id_in_appchain,
        )
        .await;
    }
    //
    // Print staking histories
    //
    if to_confirm_view_result {
        complex_viewer::print_staking_histories(&anchor).await;
    }
    //
    //
    //
    Ok((
        root,
        oct_token,
        wrapped_appchain_token,
        registry,
        council,
        anchor,
        wat_faucet,
        users,
        appchain_message_nonce,
    ))
}
