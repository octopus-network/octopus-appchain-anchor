use crate::{
    common::{
        self,
        complex_viewer::{
            print_anchor_status, print_appchain_notifications, print_delegator_reward_histories,
            print_unbonded_stakes_of, print_validator_reward_histories,
            print_wat_balance_of_anchor,
        },
    },
    contract_interfaces::{anchor_viewer, permissionless_actions, staking_actions, sudo_actions},
};
use appchain_anchor::{
    types::{AnchorStatus, MultiTxsOperationProcessingResult, ValidatorSetInfo},
    AppchainEvent, AppchainMessage,
};
use near_sdk::{json_types::U64, serde_json};
use workspaces::network::Sandbox;
use workspaces::{Account, Contract, Worker};

pub async fn process_appchain_messages(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
) -> anyhow::Result<()> {
    loop {
        let result =
            permissionless_actions::process_appchain_messages(worker, signer, anchor).await?;
        println!(
            "Process appchain messages: {}",
            serde_json::to_string::<MultiTxsOperationProcessingResult>(&result).unwrap()
        );
        println!();
        print_anchor_status(worker, anchor).await?;
        match result {
            MultiTxsOperationProcessingResult::Ok => break,
            MultiTxsOperationProcessingResult::NeedMoreGas => (),
            MultiTxsOperationProcessingResult::Error(message) => {
                panic!("Failed to process appchain messages: {}", &message);
            }
        }
    }
    Ok(())
}

pub async fn switch_era(
    worker: &Worker<Sandbox>,
    root: &Account,
    anchor: &Contract,
    era_number: u32,
    appchain_message_nonce: u32,
    to_confirm_view_result: bool,
) -> anyhow::Result<()> {
    if era_number > 0 {
        let appchain_message = AppchainMessage {
            appchain_event: AppchainEvent::EraSwitchPlaned { era_number },
            nonce: appchain_message_nonce,
        };
        sudo_actions::stage_appchain_message(worker, root, anchor, appchain_message)
            .await
            .expect("Failed to call 'stage_appchain_message'");
    }
    process_appchain_messages(worker, root, anchor).await?;
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(worker, anchor).await?;
        println!(
            "Anchor status: {}",
            serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
        );
        println!();
        let validator_set_info = anchor_viewer::get_validator_set_info_of(
            worker,
            anchor,
            U64::from(u64::from(era_number)),
        )
        .await?;
        println!(
            "Validator set info of era {}: {}",
            era_number,
            serde_json::to_string::<ValidatorSetInfo>(&validator_set_info).unwrap()
        );
        println!();
    }
    Ok(())
}

pub async fn distribute_reward_of(
    worker: &Worker<Sandbox>,
    root: &Account,
    anchor: &Contract,
    wrapped_appchain_token: &Contract,
    nonce: u32,
    era_number: u32,
    unprofitable_validator_ids: Vec<String>,
    to_confirm_view_result: bool,
) -> anyhow::Result<()> {
    let anchor_balance_of_wat =
        common::get_ft_balance_of(worker, &anchor.as_account(), &wrapped_appchain_token).await?;
    let appchain_message = AppchainMessage {
        appchain_event: AppchainEvent::EraRewardConcluded {
            era_number,
            unprofitable_validator_ids,
            offenders: Vec::new(),
        },
        nonce,
    };
    sudo_actions::stage_appchain_message(worker, root, anchor, appchain_message).await?;
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(worker, anchor).await?;
        println!(
            "Anchor status: {}",
            serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
        );
        println!();
    }
    process_appchain_messages(worker, root, anchor).await?;
    assert_eq!(
        common::get_ft_balance_of(worker, &anchor.as_account(), &wrapped_appchain_token)
            .await?
            .0,
        anchor_balance_of_wat.0 + common::to_actual_amount(10, 18)
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(worker, anchor).await?;
        println!(
            "Anchor status: {}",
            serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
        );
        println!();
        let validator_set_info = anchor_viewer::get_validator_set_info_of(
            worker,
            anchor,
            U64::from(u64::from(era_number)),
        )
        .await?;
        println!(
            "Validator set info of era {}: {}",
            era_number,
            serde_json::to_string::<ValidatorSetInfo>(&validator_set_info).unwrap()
        );
        println!();
        print_appchain_notifications(worker, &anchor).await?;
    }
    Ok(())
}

pub async fn withdraw_validator_rewards_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    user: &Account,
    wrapped_appchain_token: &Contract,
    end_era: u64,
) -> anyhow::Result<()> {
    print_wat_balance_of_anchor(worker, anchor, wrapped_appchain_token).await?;
    let wat_balance_before_withdraw =
        common::get_ft_balance_of(worker, user, wrapped_appchain_token).await?;
    staking_actions::withdraw_validator_rewards(
        worker,
        user,
        anchor,
        &user.id().to_string().parse().unwrap(),
    )
    .await?;
    println!(
        "User '{}' withdrawed rewards: {}",
        &user.id(),
        common::get_ft_balance_of(worker, user, wrapped_appchain_token)
            .await?
            .0
            - wat_balance_before_withdraw.0
    );
    println!();
    print_validator_reward_histories(worker, anchor, user, end_era).await?;
    Ok(())
}

pub async fn withdraw_delegator_rewards_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    user: &Account,
    validator: &Account,
    wrapped_appchain_token: &Contract,
    end_era: u64,
) -> anyhow::Result<()> {
    print_wat_balance_of_anchor(worker, anchor, wrapped_appchain_token).await?;
    let wat_balance_before_withdraw =
        common::get_ft_balance_of(worker, user, wrapped_appchain_token).await?;
    staking_actions::withdraw_delegator_rewards(
        worker,
        user,
        anchor,
        &user.id().to_string().parse().unwrap(),
        &validator.id().to_string().parse().unwrap(),
    )
    .await?;
    println!(
        "User '{}' withdrawed delegator rewards: {}",
        &user.id(),
        common::get_ft_balance_of(worker, user, wrapped_appchain_token)
            .await?
            .0
            - wat_balance_before_withdraw.0
    );
    println!();
    print_delegator_reward_histories(worker, anchor, user, validator, end_era).await?;
    Ok(())
}

pub async fn withdraw_stake_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    user: &Account,
    oct_token: &Contract,
) -> anyhow::Result<()> {
    let oct_balance_before_withdraw = common::get_ft_balance_of(worker, user, oct_token).await?;
    staking_actions::withdraw_stake(
        worker,
        user,
        anchor,
        &user.id().to_string().parse().unwrap(),
    )
    .await?;
    println!(
        "User '{}' withdrawed stake: {}",
        &user.id(),
        common::get_ft_balance_of(worker, user, oct_token).await?.0 - oct_balance_before_withdraw.0
    );
    println!();
    print_unbonded_stakes_of(worker, anchor, user).await?;
    Ok(())
}
