use crate::{
    common,
    contract_interfaces::{permissionless_actions, wrapped_appchain_token_manager},
};
use appchain_anchor::appchain_messages::{
    EraPayoutPayload, LockPayload, PayloadType, PlanNewEraPayload, RawMessage,
};
use near_sdk::{borsh::BorshSerialize, AccountId};
use parity_scale_codec::Encode;
use std::str::FromStr;

const TOTAL_SUPPLY: u128 = 100_000_000;

#[tokio::test]
async fn test_wrapped_appchain_token_bridging() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (
        _root,
        _oct_token,
        wrapped_appchain_token,
        _registry,
        anchor,
        _wat_faucet,
        users,
        mut appchain_message_nonce,
    ) = common::test_normal_actions(&worker, false, true, vec!["0x00".to_string()]).await?;
    //
    let total_supply = common::to_actual_amount(TOTAL_SUPPLY, 18);
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    let user4_id_in_appchain =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da273".to_string();
    //
    // Mint wrapped appchain token for user1 (error)
    //
    let user1_wat_balance =
        common::get_ft_balance_of(&worker, &users[1], &wrapped_appchain_token).await?;
    appchain_message_nonce += 1;
    let payload = LockPayload {
        sender: user4_id_in_appchain.clone(),
        receiver_id: AccountId::from_str("unknown.testnet").unwrap(),
        amount: total_supply / 10,
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::Lock,
        payload: payload.try_to_vec().unwrap(),
    };
    let mut raw_messages = Vec::new();
    raw_messages.push(raw_message);
    permissionless_actions::verify_and_stage_appchain_messages(
        &worker,
        &users[5],
        &anchor,
        raw_messages.encode(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .await
    .expect("Failed to call 'verify_and_stage_appchain_messages'");
    common::complex_actions::process_appchain_messages(&worker, &users[4], &anchor).await?;
    common::complex_viewer::print_appchain_messages(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_messages_processing_results(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    assert_eq!(
        common::get_ft_balance_of(&worker, &users[1], &wrapped_appchain_token)
            .await?
            .0,
        user1_wat_balance.0
    );
    //
    // Burn wrapped appchain token from user0
    //
    let result = wrapped_appchain_token_manager::burn_wrapped_appchain_token(
        &worker,
        &users[0],
        &anchor,
        user0_id_in_appchain,
        total_supply / 2 - common::to_actual_amount(50000, 18),
    )
    .await?;
    assert!(result.is_success());
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    //
    // Mint wrapped appchain token for user1
    //
    let user1_wat_balance =
        common::get_ft_balance_of(&worker, &users[1], &wrapped_appchain_token).await?;
    let mut raw_messages = Vec::new();
    //
    appchain_message_nonce += 1;
    let payload = LockPayload {
        sender: user4_id_in_appchain.clone(),
        receiver_id: users[1].id().to_string().parse().unwrap(),
        amount: common::to_actual_amount(60, 18),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::Lock,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = LockPayload {
        sender: user4_id_in_appchain.clone(),
        receiver_id: users[1].id().to_string().parse().unwrap(),
        amount: common::to_actual_amount(40, 18),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::Lock,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = PlanNewEraPayload { new_era: 1 };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::PlanNewEra,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = LockPayload {
        sender: user4_id_in_appchain.clone(),
        receiver_id: users[1].id().to_string().parse().unwrap(),
        amount: common::to_actual_amount(70, 18),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::Lock,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = LockPayload {
        sender: user4_id_in_appchain.clone(),
        receiver_id: users[1].id().to_string().parse().unwrap(),
        amount: common::to_actual_amount(30, 18),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::Lock,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = LockPayload {
        sender: user4_id_in_appchain.clone(),
        receiver_id: users[1].id().to_string().parse().unwrap(),
        amount: common::to_actual_amount(45, 18),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::Lock,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = LockPayload {
        sender: user4_id_in_appchain.clone(),
        receiver_id: users[1].id().to_string().parse().unwrap(),
        amount: common::to_actual_amount(45, 18),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::Lock,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = LockPayload {
        sender: user4_id_in_appchain.clone(),
        receiver_id: users[1].id().to_string().parse().unwrap(),
        amount: common::to_actual_amount(45, 18),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::Lock,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = LockPayload {
        sender: user4_id_in_appchain.clone(),
        receiver_id: users[1].id().to_string().parse().unwrap(),
        amount: common::to_actual_amount(45, 18),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::Lock,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = EraPayoutPayload {
        end_era: 0,
        excluded_validators: Vec::new(),
        offenders: Vec::new(),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::EraPayout,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = LockPayload {
        sender: user4_id_in_appchain.clone(),
        receiver_id: users[1].id().to_string().parse().unwrap(),
        amount: common::to_actual_amount(45, 18),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::Lock,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = LockPayload {
        sender: user4_id_in_appchain.clone(),
        receiver_id: users[1].id().to_string().parse().unwrap(),
        amount: common::to_actual_amount(45, 18),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::Lock,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = LockPayload {
        sender: user4_id_in_appchain.clone(),
        receiver_id: users[1].id().to_string().parse().unwrap(),
        amount: common::to_actual_amount(45, 18),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::Lock,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    permissionless_actions::verify_and_stage_appchain_messages(
        &worker,
        &users[5],
        &anchor,
        raw_messages.encode(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .await
    .expect("Failed to call 'verify_and_stage_appchain_messages'");
    common::complex_actions::process_appchain_messages(&worker, &users[3], &anchor).await?;
    common::complex_viewer::print_appchain_messages(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_messages_processing_results(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    assert_eq!(
        common::get_ft_balance_of(&worker, &users[1], &wrapped_appchain_token)
            .await?
            .0,
        user1_wat_balance.0 + common::to_actual_amount(515, 18)
    );
    //
    //
    //
    let mut raw_messages = Vec::new();
    //
    appchain_message_nonce += 1;
    let payload = PlanNewEraPayload { new_era: 2 };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::PlanNewEra,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = EraPayoutPayload {
        end_era: 1,
        excluded_validators: Vec::new(),
        offenders: Vec::new(),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::EraPayout,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = PlanNewEraPayload { new_era: 3 };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::PlanNewEra,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = EraPayoutPayload {
        end_era: 2,
        excluded_validators: Vec::new(),
        offenders: Vec::new(),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::EraPayout,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    appchain_message_nonce += 1;
    let payload = EraPayoutPayload {
        end_era: 1,
        excluded_validators: Vec::new(),
        offenders: Vec::new(),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::EraPayout,
        payload: payload.try_to_vec().unwrap(),
    };
    raw_messages.push(raw_message);
    //
    permissionless_actions::verify_and_stage_appchain_messages(
        &worker,
        &users[5],
        &anchor,
        raw_messages.encode(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .await
    .expect("Failed to call 'verify_and_stage_appchain_messages'");
    common::complex_actions::process_appchain_messages(&worker, &users[3], &anchor).await?;
    common::complex_viewer::print_appchain_messages(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_messages_processing_results(&worker, &anchor).await?;
    Ok(())
}
