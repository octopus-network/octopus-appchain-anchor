use crate::{
    common::{self, to_actual_amount},
    contract_interfaces::{native_near_token, permissionless_actions},
};
use appchain_anchor::appchain_messages::{BurnAssetPayload, PayloadType, RawMessage};
use near_sdk::{borsh::BorshSerialize, json_types::U128, serde_json::json};
use near_units::parse_near;
use parity_scale_codec::Encode;
use std::str::FromStr;
use workspaces::AccountId;

#[tokio::test]
async fn test_transfer_native_near() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (
        root,
        _oct_token,
        _wrapped_appchain_token,
        _registry,
        _council,
        anchor,
        _wat_faucet,
        users,
        mut appchain_message_nonce,
    ) = common::test_normal_actions(&worker, false, true).await?;
    //
    //
    //
    assert!(
        native_near_token::deploy_near_vault_contract(&users[0], &anchor)
            .await
            .unwrap()
            .is_failure()
    );
    //
    assert!(root
        .call(anchor.id(), "store_wasm_of_near_vault_contract")
        .args(std::fs::read(format!("res/near_vault.wasm"))?)
        .gas(300_000_000_000_000)
        .deposit(parse_near!("2 N"))
        .transact()
        .await
        .unwrap()
        .is_success());
    //
    assert!(
        native_near_token::deploy_near_vault_contract(&users[0], &anchor)
            .await
            .unwrap()
            .is_failure()
    );
    //
    assert!(
        native_near_token::deploy_near_vault_contract(&root, &anchor)
            .await
            .unwrap()
            .is_success()
    );
    //
    //
    //
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    let receiver_account =
        AccountId::from_str(format!("near-vault.{}", anchor.id()).as_str()).unwrap();
    let old_balance = users[0].view_account().await?.balance;
    println!("Balance of users[0]: {}", old_balance);
    assert!(users[0]
        .call(&receiver_account, "deposit_near_for_appchain_user")
        .args_json(json!({
            "receiver_id_in_appchain": user0_id_in_appchain,
            "near_amount": U128::from(parse_near!("1 N")),
        }))
        .gas(200_000_000_000_000)
        .deposit(parse_near!("1 N"))
        .transact()
        .await
        .unwrap()
        .is_success());
    let new_balance = users[0].view_account().await?.balance;
    println!("Balance of users[0]: {}", new_balance);
    assert!(old_balance - new_balance > parse_near!("1 N"));
    common::complex_viewer::print_appchain_notifications(&anchor).await;
    //
    //
    //
    assert!(
        native_near_token::open_bridging_of_native_near_token(&root, &anchor)
            .await
            .unwrap()
            .is_success()
    );
    //
    let old_balance = users[0].view_account().await?.balance;
    println!("Balance of users[0]: {}", old_balance);
    assert!(users[0]
        .call(&receiver_account, "deposit_near_for_appchain_user")
        .args_json(json!({
            "receiver_id_in_appchain": user0_id_in_appchain,
            "near_amount": U128::from(parse_near!("1 N")),
        }))
        .gas(200_000_000_000_000)
        .deposit(parse_near!("1 N"))
        .transact()
        .await
        .unwrap()
        .is_success());
    let new_balance = users[0].view_account().await?.balance;
    println!("Balance of users[0]: {}", new_balance);
    assert!(old_balance - new_balance > parse_near!("1 N"));
    common::complex_viewer::print_appchain_notifications(&anchor).await;
    common::complex_viewer::print_native_near_token(&anchor).await;
    //
    //
    //
    let old_balance = users[0].view_account().await?.balance;
    println!("Balance of users[0]: {}", old_balance);
    appchain_message_nonce += 1;
    let payload = BurnAssetPayload {
        token_id: "NEAR".to_string(),
        sender: user0_id_in_appchain.clone(),
        receiver_id: users[0].id().to_string().parse().unwrap(),
        amount: parse_near!("1 N"),
        fee: to_actual_amount(1, 18),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::BurnAsset,
        payload: payload.try_to_vec().unwrap(),
    };
    let mut raw_messages = Vec::new();
    raw_messages.push(raw_message);
    assert!(permissionless_actions::stage_and_apply_appchain_messages(
        &users[5],
        &anchor,
        raw_messages.encode(),
        None,
    )
    .await
    .unwrap()
    .is_success());
    //
    common::complex_viewer::print_appchain_messages(&anchor).await;
    common::complex_viewer::print_appchain_messages_processing_results(&anchor).await;
    let new_balance = users[0].view_account().await?.balance;
    println!("Balance of users[0]: {}", new_balance);
    assert!(new_balance - old_balance == parse_near!("1 N"));
    common::complex_viewer::print_appchain_notifications(&anchor).await;
    common::complex_viewer::print_native_near_token(&anchor).await;
    //
    Ok(())
}
