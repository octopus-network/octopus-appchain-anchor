use crate::{
    common,
    contract_interfaces::{permissionless_actions, wrapped_appchain_nft_manager},
};
use appchain_anchor::{
    appchain_messages::{LockNftPayload, PayloadType, RawMessage},
    types::NFTTransferMessage,
};
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, TokenMetadata, NFT_METADATA_SPEC,
};
use near_sdk::{
    borsh::BorshSerialize,
    serde_json::{self, json},
};
use near_units::parse_near;
use parity_scale_codec::Encode;
use std::str::FromStr;
use workspaces::AccountId;

#[tokio::test]
async fn test_transfer_nft_to_near() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (
        root,
        _oct_token,
        _wrapped_appchain_token,
        _registry,
        anchor,
        _wat_faucet,
        users,
        mut appchain_message_nonce,
    ) = common::test_normal_actions(&worker, false, true, vec!["0x00".to_string()]).await?;
    //
    //
    //
    wrapped_appchain_nft_manager::register_wrapped_appchain_nft(
        &worker,
        &users[0],
        &anchor,
        "nft_class1".to_string(),
        NFTContractMetadata {
            spec: NFT_METADATA_SPEC.to_string(),
            name: "nft class type 1".to_string(),
            symbol: "nft_class1".to_string(),
            icon: None,
            base_uri: None,
            reference: None,
            reference_hash: None,
        },
    )
    .await
    .expect_err("Should fail");
    //
    wrapped_appchain_nft_manager::register_wrapped_appchain_nft(
        &worker,
        &root,
        &anchor,
        "nft_class1".to_string(),
        NFTContractMetadata {
            spec: NFT_METADATA_SPEC.to_string(),
            name: "nft class type 1".to_string(),
            symbol: "nft_class1".to_string(),
            icon: None,
            base_uri: None,
            reference: None,
            reference_hash: None,
        },
    )
    .await
    .expect_err("Should fail");
    //
    root.call(
        &worker,
        anchor.id(),
        "store_wasm_of_wrapped_appchain_nft_contract",
    )
    .args(std::fs::read(format!("res/wrapped_appchain_nft.wasm"))?)
    .gas(300_000_000_000_000)
    .deposit(parse_near!("30 N"))
    .transact()
    .await
    .expect("Failed in calling 'store_wasm_of_wrapped_appchain_nft_contract'");
    //
    let class_id = "1".to_string();
    wrapped_appchain_nft_manager::register_wrapped_appchain_nft(
        &worker,
        &root,
        &anchor,
        class_id.clone(),
        NFTContractMetadata {
            spec: NFT_METADATA_SPEC.to_string(),
            name: "nft class type 1".to_string(),
            symbol: "nft_class1".to_string(),
            icon: None,
            base_uri: None,
            reference: None,
            reference_hash: None,
        },
    )
    .await
    .expect("Failed to register wrapped appchain nft");
    //
    //
    //
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    //
    //
    //
    appchain_message_nonce += 1;
    let payload = LockNftPayload {
        sender: user0_id_in_appchain.clone(),
        receiver_id: users[0].id().to_string().parse().unwrap(),
        class: 1 as u128,
        instance: 1 as u128,
        metadata: TokenMetadata {
            title: Some("token_id1 title".to_string()),
            description: Some("token_id1 description".to_string()),
            media: None,
            media_hash: None,
            copies: Some(1),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        },
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::LockNft,
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
    //
    //
    //
    let result = wrapped_appchain_nft_manager::open_bridging_of_wrapped_appchain_nft(
        &worker,
        &root,
        &anchor,
        class_id.clone(),
    )
    .await?;
    assert!(result.is_success());
    appchain_message_nonce += 1;
    let payload = LockNftPayload {
        sender: user0_id_in_appchain.clone(),
        receiver_id: users[0].id().to_string().parse().unwrap(),
        class: 1 as u128,
        instance: 1 as u128,
        metadata: TokenMetadata {
            title: Some("token_id1 title".to_string()),
            description: Some("token_id1 description".to_string()),
            media: None,
            media_hash: None,
            copies: Some(1),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        },
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::LockNft,
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
    //
    //
    //
    users[0]
        .call(
            &worker,
            &AccountId::from_str(
                format!("{}.{}", class_id.clone(), anchor.id().to_string()).as_str(),
            )
            .unwrap(),
            "nft_transfer_call",
        )
        .args_json(json!({
            "receiver_id": anchor.id(),
            "token_id": "1".to_string(),
            "approval_Id": Option::<String>::None,
            "memo": Option::<String>::None,
            "msg": serde_json::ser::to_string(&NFTTransferMessage::BridgeToAppchain {
                receiver_id_in_appchain: user0_id_in_appchain.clone(),
            }).unwrap(),
        }))?
        .gas(300_000_000_000_000)
        .deposit(1)
        .transact()
        .await
        .expect("Failed to transfer nft");
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    //
    //
    //
    appchain_message_nonce += 1;
    let payload = LockNftPayload {
        sender: user0_id_in_appchain.clone(),
        receiver_id: users[0].id().to_string().parse().unwrap(),
        class: 1 as u128,
        instance: 1 as u128,
        metadata: TokenMetadata {
            title: Some("token_id1 title".to_string()),
            description: Some("token_id1 description".to_string()),
            media: None,
            media_hash: None,
            copies: Some(1),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        },
    };
    let raw_message = RawMessage {
        nonce: appchain_message_nonce as u64,
        payload_type: PayloadType::LockNft,
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
    Ok(())
}
