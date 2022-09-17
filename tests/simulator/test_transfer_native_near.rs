use crate::{
    common,
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
    ) = common::test_normal_actions(&worker, false, true, vec!["0x00".to_string()]).await?;
    //
    //
    //
    native_near_token::deploy_near_vault_contract(&worker, &users[0], &anchor)
        .await
        .expect_err("Should fail.");
    //
    root.call(&worker, anchor.id(), "store_wasm_of_near_vault_contract")
        .args(std::fs::read(format!("res/near_vault.wasm"))?)
        .gas(300_000_000_000_000)
        .deposit(parse_near!("2 N"))
        .transact()
        .await
        .expect("Failed in calling 'store_wasm_of_near_vault_contract'.");
    //
    native_near_token::deploy_near_vault_contract(&worker, &users[0], &anchor)
        .await
        .expect_err("Should fail.");
    //
    native_near_token::deploy_near_vault_contract(&worker, &root, &anchor)
        .await
        .expect("Failed to deploy native near token receiver contract.");
    //
    //
    //
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    let receiver_account =
        AccountId::from_str(format!("near-vault.{}", anchor.id()).as_str()).unwrap();
    let old_balance = users[0].view_account(&worker).await?.balance;
    println!("Balance of users[0]: {}", old_balance);
    users[0]
        .call(&worker, &receiver_account, "deposit_near_for_appchain_user")
        .args_json(json!({
            "receiver_id_in_appchain": user0_id_in_appchain,
            "near_amount": U128::from(parse_near!("1 N")),
        }))?
        .gas(200_000_000_000_000)
        .deposit(parse_near!("1 N"))
        .transact()
        .await
        .expect("Failed in calling 'deposit_near_for_appchain_user'");
    let new_balance = users[0].view_account(&worker).await?.balance;
    println!("Balance of users[0]: {}", new_balance);
    assert!(old_balance - new_balance > parse_near!("1 N"));
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    //
    //
    //
    native_near_token::open_bridging_of_native_near_token(&worker, &root, &anchor)
        .await
        .expect("Failed to open bridging of native NEAR token.");
    //
    let old_balance = users[0].view_account(&worker).await?.balance;
    println!("Balance of users[0]: {}", old_balance);
    users[0]
        .call(&worker, &receiver_account, "deposit_near_for_appchain_user")
        .args_json(json!({
            "receiver_id_in_appchain": user0_id_in_appchain,
            "near_amount": U128::from(parse_near!("1 N")),
        }))?
        .gas(200_000_000_000_000)
        .deposit(parse_near!("1 N"))
        .transact()
        .await
        .expect("Failed in calling 'deposit_near_for_appchain_user'");
    let new_balance = users[0].view_account(&worker).await?.balance;
    println!("Balance of users[0]: {}", new_balance);
    assert!(old_balance - new_balance > parse_near!("1 N"));
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    common::complex_viewer::print_native_near_token(&worker, &anchor).await?;
    //
    //
    //
    appchain_message_nonce += 1;
    let payload = BurnAssetPayload {
        token_id: "NEAR".to_string(),
        sender: user0_id_in_appchain.clone(),
        receiver_id: users[0].id().to_string().parse().unwrap(),
        amount: parse_near!("1 N"),
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::BurnAsset,
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
    //
    let old_balance = users[0].view_account(&worker).await?.balance;
    println!("Balance of users[0]: {}", old_balance);
    common::complex_actions::process_appchain_messages(&worker, &users[4], &anchor).await?;
    let new_balance = users[0].view_account(&worker).await?.balance;
    println!("Balance of users[0]: {}", new_balance);
    assert!(new_balance - old_balance == parse_near!("1 N"));
    common::complex_viewer::print_appchain_messages(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_messages_processing_results(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    common::complex_viewer::print_native_near_token(&worker, &anchor).await?;
    //
    Ok(())
}
