use crate::{
    common::{
        self,
        complex_viewer::{
            print_anchor_status, print_appchain_notifications, print_delegator_reward_histories,
            print_unbonded_stakes_of, print_validator_reward_histories,
            print_wat_balance_of_anchor,
        },
    },
    contract_interfaces::{anchor_viewer, permissionless_actions, staking_actions},
};
use appchain_anchor::{
    appchain_messages::{EraPayoutPayload, RawMessage},
    appchain_messages::{PayloadType, PlanNewEraPayload},
    types::{AnchorStatus, MultiTxsOperationProcessingResult, ValidatorSetInfo},
};
use near_primitives::borsh::BorshSerialize;
use near_sdk::{json_types::U64, serde_json};
use parity_scale_codec::Encode;
use workspaces::{Account, Contract};

pub async fn process_appchain_messages(signer: &Account, anchor: &Contract) {
    loop {
        let result = permissionless_actions::process_appchain_messages(signer, anchor)
            .await
            .unwrap();
        println!(
            "Process appchain messages: {}",
            serde_json::to_string::<MultiTxsOperationProcessingResult>(&result).unwrap()
        );
        println!();
        print_anchor_status(anchor).await;
        match result {
            MultiTxsOperationProcessingResult::Ok => break,
            MultiTxsOperationProcessingResult::NeedMoreGas => (),
            MultiTxsOperationProcessingResult::Error(message) => {
                panic!("Failed to process appchain messages: {}", &message);
            }
        }
    }
}

pub async fn switch_era(
    relayer: &Account,
    anchor: &Contract,
    era_number: u32,
    appchain_message_nonce: u32,
    to_confirm_view_result: bool,
) {
    if era_number > 0 {
        let payload = PlanNewEraPayload {
            new_era: era_number,
        };
        let raw_message = RawMessage {
            nonce: appchain_message_nonce as u64,
            payload_type: PayloadType::PlanNewEra,
            payload: payload.try_to_vec().unwrap(),
        };
        let mut raw_messages = Vec::new();
        raw_messages.push(raw_message);
        assert!(permissionless_actions::verify_and_stage_appchain_messages(
            relayer,
            anchor,
            raw_messages.encode(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
        .await
        .unwrap()
        .is_success());
    }
    process_appchain_messages(relayer, anchor).await;
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(anchor).await.unwrap();
        println!(
            "Anchor status: {}",
            serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
        );
        println!();
        let validator_set_info =
            anchor_viewer::get_validator_set_info_of(anchor, U64::from(u64::from(era_number)))
                .await
                .unwrap();
        println!(
            "Validator set info of era {}: {}",
            era_number,
            serde_json::to_string::<ValidatorSetInfo>(&validator_set_info).unwrap()
        );
        println!();
    }
}

pub async fn distribute_reward_of(
    relayer: &Account,
    anchor: &Contract,
    wrapped_appchain_token: &Contract,
    nonce: u32,
    era_number: u32,
    unprofitable_validator_ids: Vec<String>,
    to_confirm_view_result: bool,
) {
    let anchor_balance_of_wat =
        common::get_ft_balance_of(&anchor.as_account(), &wrapped_appchain_token)
            .await
            .unwrap();
    let payload = EraPayoutPayload {
        end_era: era_number,
        excluded_validators: unprofitable_validator_ids,
        offenders: Vec::new(),
    };
    let raw_message = RawMessage {
        nonce: nonce as u64,
        payload_type: PayloadType::EraPayout,
        payload: payload.try_to_vec().unwrap(),
    };
    let mut raw_messages = Vec::new();
    raw_messages.push(raw_message);
    assert!(permissionless_actions::verify_and_stage_appchain_messages(
        relayer,
        anchor,
        raw_messages.encode(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .await
    .unwrap()
    .is_success());
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(anchor).await.unwrap();
        println!(
            "Anchor status: {}",
            serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
        );
        println!();
    }
    process_appchain_messages(relayer, anchor).await;
    assert_eq!(
        common::get_ft_balance_of(&anchor.as_account(), &wrapped_appchain_token)
            .await
            .unwrap()
            .0,
        anchor_balance_of_wat.0 + common::to_actual_amount(10, 18)
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(anchor).await.unwrap();
        println!(
            "Anchor status: {}",
            serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
        );
        println!();
        let validator_set_info =
            anchor_viewer::get_validator_set_info_of(anchor, U64::from(u64::from(era_number)))
                .await
                .unwrap();
        println!(
            "Validator set info of era {}: {}",
            era_number,
            serde_json::to_string::<ValidatorSetInfo>(&validator_set_info).unwrap()
        );
        println!();
        print_appchain_notifications(&anchor).await;
    }
}

pub async fn withdraw_validator_rewards_of(
    anchor: &Contract,
    user: &Account,
    wrapped_appchain_token: &Contract,
    end_era: u64,
) {
    print_wat_balance_of_anchor(anchor, wrapped_appchain_token).await;
    let wat_balance_before_withdraw = common::get_ft_balance_of(user, wrapped_appchain_token)
        .await
        .unwrap();
    assert!(staking_actions::withdraw_validator_rewards(
        user,
        anchor,
        &user.id().to_string().parse().unwrap(),
    )
    .await
    .unwrap()
    .is_success());
    println!(
        "User '{}' withdrawed rewards: {}",
        &user.id(),
        common::get_ft_balance_of(user, wrapped_appchain_token)
            .await
            .unwrap()
            .0
            - wat_balance_before_withdraw.0
    );
    println!();
    print_validator_reward_histories(anchor, user, end_era).await;
}

pub async fn withdraw_delegator_rewards_of(
    anchor: &Contract,
    user: &Account,
    validator: &Account,
    wrapped_appchain_token: &Contract,
    end_era: u64,
) {
    print_wat_balance_of_anchor(anchor, wrapped_appchain_token).await;
    let wat_balance_before_withdraw = common::get_ft_balance_of(user, wrapped_appchain_token)
        .await
        .unwrap();
    assert!(staking_actions::withdraw_delegator_rewards(
        user,
        anchor,
        &user.id().to_string().parse().unwrap(),
        &validator.id().to_string().parse().unwrap(),
    )
    .await
    .unwrap()
    .is_success());
    println!(
        "User '{}' withdrawed delegator rewards: {}",
        &user.id(),
        common::get_ft_balance_of(user, wrapped_appchain_token)
            .await
            .unwrap()
            .0
            - wat_balance_before_withdraw.0
    );
    println!();
    print_delegator_reward_histories(anchor, user, validator, end_era).await;
}

pub async fn withdraw_stake_of(
    anchor: &Contract,
    user: &Account,
    oct_token: &Contract,
) -> anyhow::Result<()> {
    let oct_balance_before_withdraw = common::get_ft_balance_of(user, oct_token).await?;
    assert!(
        staking_actions::withdraw_stake(user, anchor, &user.id().to_string().parse().unwrap())
            .await
            .unwrap()
            .is_success()
    );
    println!(
        "User '{}' withdrawed stake: {}",
        &user.id(),
        common::get_ft_balance_of(user, oct_token).await?.0 - oct_balance_before_withdraw.0
    );
    println!();
    print_unbonded_stakes_of(anchor, user).await;
    Ok(())
}
