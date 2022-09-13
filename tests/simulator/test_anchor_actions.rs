use crate::{
    common,
    contract_interfaces::{anchor_viewer, settings_manager, staking_actions},
};
use near_sdk::json_types::U64;
use std::collections::HashMap;
use workspaces::{network::Sandbox, Account, Contract, Worker};

#[tokio::test]
async fn test_anchor_actions() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (
        root,
        oct_token,
        wrapped_appchain_token,
        _registry,
        anchor,
        _wat_faucet,
        users,
        mut appchain_message_nonce,
    ) = common::test_normal_actions(&worker, false, true, vec!["0x00".to_string()]).await?;
    //
    // Try start and complete switching era1
    //
    appchain_message_nonce += 1;
    common::complex_actions::switch_era(
        &worker,
        &users[5],
        &anchor,
        1,
        appchain_message_nonce,
        false,
    )
    .await
    .expect("Failed to switch era");
    common::complex_viewer::print_validator_set_info_of(&worker, &anchor, U64::from(1)).await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(1)).await?;
    common::complex_viewer::print_delegator_list_of(&worker, &anchor, 1, &users[0]).await?;
    //
    // Distribut reward of era0
    //
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &users[5],
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        0,
        Vec::new(),
        false,
    )
    .await
    .expect("Failed to distribute rewards");
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[0], 0)
        .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[1], 0)
        .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[2], &users[0], 0,
    )
    .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[3], &users[0], 0,
    )
    .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[4], 0)
        .await?;
    //
    //
    //
    test_staking_actions(
        &worker,
        &root,
        &oct_token,
        &wrapped_appchain_token,
        &anchor,
        &users,
        appchain_message_nonce,
    )
    .await?;
    //
    Ok(())
}

async fn test_staking_actions(
    worker: &Worker<Sandbox>,
    root: &Account,
    oct_token: &Contract,
    wrapped_appchain_token: &Contract,
    anchor: &Contract,
    users: &Vec<Account>,
    mut appchain_message_nonce: u32,
) -> anyhow::Result<()> {
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
    staking_actions::decrease_stake(
        &worker,
        &users[1],
        &anchor,
        common::to_actual_amount(1000, 18),
    )
    .await
    .expect("Failed to decrease stake");
    common::complex_viewer::print_anchor_status(&worker, &anchor).await?;
    let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(&worker, &anchor, &users[1])
        .await
        .expect("Failed to unbond stakes");
    assert!(unbonded_stakes.len() == 0);
    //
    // user2 decrease delegation
    //
    staking_actions::decrease_delegation(
        &worker,
        &users[2],
        &anchor,
        &users[0].id().to_string().parse().unwrap(),
        common::to_actual_amount(200, 18),
    )
    .await
    .expect("Failed to decrease delegation");
    common::complex_viewer::print_anchor_status(&worker, &anchor).await?;
    let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(&worker, &anchor, &users[2])
        .await
        .expect("Failed to unbond stakes");
    assert!(unbonded_stakes.len() == 0);
    //
    // Print staking histories
    //
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    //
    // Try start and complete switching era2
    //
    appchain_message_nonce += 1;
    common::complex_actions::switch_era(
        &worker,
        &users[5],
        &anchor,
        2,
        appchain_message_nonce,
        true,
    )
    .await
    .expect("Failed to switch era");
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(2)).await?;
    common::complex_viewer::print_delegator_list_of(&worker, &anchor, 2, &users[0]).await?;
    //
    // Distribute reward of era1
    //
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &users[5],
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        1,
        [user0_id_in_appchain.clone()].to_vec(),
        true,
    )
    .await
    .expect("Failed to distribute rewards");
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[0], 1)
        .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[1], 1)
        .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[2], &users[0], 1,
    )
    .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[3], &users[0], 1,
    )
    .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[4], 1)
        .await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[0]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[1]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[2]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[3]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[4]).await?;
    //
    // Change unlock period for testing
    //
    settings_manager::change_unlock_period_of_validator_deposit(&worker, &root, &anchor, 3)
        .await
        .expect("Failed in calling 'change_unlock_period_of_validator_deposit'");
    settings_manager::change_unlock_period_of_delegator_deposit(&worker, &root, &anchor, 1)
        .await
        .expect("Failed in calling 'change_unlock_period_of_delegator_deposit'");
    //
    // user3 unbond delegation
    //
    staking_actions::unbond_delegation(
        &worker,
        &users[2],
        &anchor,
        &users[0].id().to_string().parse().unwrap(),
    )
    .await
    .expect("Failed to unbond delegation");
    common::complex_viewer::print_anchor_status(&worker, &anchor).await?;
    let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(&worker, &anchor, &users[2])
        .await
        .expect("Failed to unbond stakes");
    assert!(unbonded_stakes.len() == 1);
    //
    // Print staking histories
    //
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    //
    // Try start and complete switching era3
    //
    appchain_message_nonce += 1;
    common::complex_actions::switch_era(
        &worker,
        &users[5],
        &anchor,
        3,
        appchain_message_nonce,
        true,
    )
    .await
    .expect("Failed to switch era");
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(3)).await?;
    common::complex_viewer::print_delegator_list_of(&worker, &anchor, 3, &users[0]).await?;
    //
    // Distribute reward of era2
    //
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &users[5],
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        2,
        [user0_id_in_appchain.clone(), user4_id_in_appchain.clone()].to_vec(),
        true,
    )
    .await
    .expect("Failed to distribute rewards");
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[0], 2)
        .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[1], 2)
        .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[2], &users[0], 2,
    )
    .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[3], &users[0], 2,
    )
    .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[4], 2)
        .await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[0]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[1]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[2]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[3]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[4]).await?;
    //
    // user0 unbond stake
    //
    // let result = staking_actions::unbond_stake(&users[0], &anchor).await?;
    // assert!(result.is_success());
    // common::complex_viewer::print_anchor_status(&worker, &anchor).await?;
    // let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(&worker, &anchor, &users[0]).await?;
    // assert!(unbonded_stakes.len() == 0);
    //
    // user1 unbond stake
    //
    // let result = staking_actions::unbond_stake(&users[1], &anchor).await?;
    // assert!(result.is_success());
    // common::complex_viewer::print_anchor_status(&worker, &anchor).await?;
    // let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(&worker, &anchor, &users[1]).await?;
    // assert!(unbonded_stakes.len() == 1);
    //
    // Print staking histories
    //
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    //
    // Try start and complete switching era4
    //
    appchain_message_nonce += 1;
    common::complex_actions::switch_era(
        &worker,
        &users[5],
        &anchor,
        4,
        appchain_message_nonce,
        true,
    )
    .await
    .expect("Failed to switch era");
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(4)).await?;
    common::complex_viewer::print_delegator_list_of(&worker, &anchor, 4, &users[0]).await?;
    //
    // Distribute reward of era3
    //
    common::complex_viewer::print_validator_set_info_of(&worker, &anchor, U64::from(3)).await?;
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &users[5],
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        3,
        [user0_id_in_appchain.clone(), user4_id_in_appchain.clone()].to_vec(),
        true,
    )
    .await
    .expect("Failed to distribute rewards");
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[0], 3)
        .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[1], 3)
        .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[2], &users[0], 3,
    )
    .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[3], &users[0], 3,
    )
    .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[4], 3)
        .await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[0]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[1]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[2]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[3]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[4]).await?;
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &users[5],
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        3,
        Vec::new(),
        true,
    )
    .await
    .expect("Failed to distribute rewards");
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[0], 3)
        .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[1], 3)
        .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[2], &users[0], 3,
    )
    .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[3], &users[0], 3,
    )
    .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[4], 3)
        .await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[0]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[1]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[2]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[3]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[4]).await?;
    //
    // Try start and complete switching era5
    //
    appchain_message_nonce += 1;
    common::complex_actions::switch_era(
        &worker,
        &users[5],
        &anchor,
        5,
        appchain_message_nonce,
        true,
    )
    .await
    .expect("Failed to switch era");
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(5)).await?;
    common::complex_viewer::print_delegator_list_of(&worker, &anchor, 5, &users[0]).await?;
    //
    // Distribute reward of era4
    //
    common::complex_viewer::print_validator_set_info_of(&worker, &anchor, U64::from(4)).await?;
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &users[5],
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        4,
        Vec::new(),
        true,
    )
    .await
    .expect("Failed to distribute rewards");
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[0], 4)
        .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[1], 4)
        .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[2], &users[0], 4,
    )
    .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[3], &users[0], 4,
    )
    .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[4], 4)
        .await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[0]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[1]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[2]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[3]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[4]).await?;
    //
    // Withdraw validator rewards
    //
    common::complex_actions::withdraw_validator_rewards_of(
        &worker,
        &anchor,
        &users[0],
        &wrapped_appchain_token,
        3,
    )
    .await
    .expect("Failed in calling 'withdraw_validator_rewards_of'");
    common::complex_actions::withdraw_validator_rewards_of(
        &worker,
        &anchor,
        &users[1],
        &wrapped_appchain_token,
        3,
    )
    .await
    .expect("Failed in calling 'withdraw_validator_rewards_of'");
    common::complex_actions::withdraw_validator_rewards_of(
        &worker,
        &anchor,
        &users[4],
        &wrapped_appchain_token,
        3,
    )
    .await
    .expect("Failed in calling 'withdraw_validator_rewards_of'");
    //
    // Withdraw delegator rewards
    //
    common::complex_actions::withdraw_delegator_rewards_of(
        &worker,
        &anchor,
        &users[2],
        &users[0],
        &wrapped_appchain_token,
        3,
    )
    .await
    .expect("Failed in calling 'withdraw_delegator_rewards_of'");
    common::complex_actions::withdraw_delegator_rewards_of(
        &worker,
        &anchor,
        &users[3],
        &users[0],
        &wrapped_appchain_token,
        3,
    )
    .await
    .expect("Failed in calling 'withdraw_delegator_rewards_of'");
    //
    // Withdraw stake
    //
    common::complex_actions::withdraw_stake_of(&worker, &anchor, &users[0], &oct_token)
        .await
        .expect("Failed in calling 'withdraw_stake_of'");
    common::complex_actions::withdraw_stake_of(&worker, &anchor, &users[1], &oct_token)
        .await
        .expect("Failed in calling 'withdraw_stake_of'");
    common::complex_actions::withdraw_stake_of(&worker, &anchor, &users[2], &oct_token)
        .await
        .expect("Failed in calling 'withdraw_stake_of'");
    common::complex_actions::withdraw_stake_of(&worker, &anchor, &users[3], &oct_token)
        .await
        .expect("Failed in calling 'withdraw_stake_of'");
    common::complex_actions::withdraw_stake_of(&worker, &anchor, &users[4], &oct_token)
        .await
        .expect("Failed in calling 'withdraw_stake_of'");
    //
    // Print whole status
    //
    common::complex_viewer::print_anchor_status(&worker, &anchor).await?;
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(0)).await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(1)).await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(2)).await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(3)).await?;
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    Ok(())
}
