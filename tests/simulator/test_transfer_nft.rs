use std::str::FromStr;

use appchain_anchor::{AppchainEvent, AppchainMessage};
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, TokenMetadata, NFT_METADATA_SPEC,
};
use near_sdk::{serde_json::json, AccountId};
use near_sdk_sim::{lazy_static_include, to_yocto};

use crate::{common, settings_manager, sudo_actions, wrapped_appchain_nft_manager};

lazy_static_include::lazy_static_include_bytes! {
    WRAPPED_APPCHAIN_NFT_WASM_BYTES => "./res/wrapped_appchain_nft.wasm",
}

#[test]
fn test_transfer_nft_to_near() {
    let (
        root,
        _oct_token,
        _wrapped_appchain_token,
        _registry,
        anchor,
        users,
        mut appchain_message_nonce,
    ) = common::test_normal_actions(false, true);
    //
    //
    //
    let result = wrapped_appchain_nft_manager::register_wrapped_appchain_nft(
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
    );
    assert!(!result.is_ok());
    let result = wrapped_appchain_nft_manager::register_wrapped_appchain_nft(
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
    );
    assert!(!result.is_ok());
    let result = root.call(
        anchor.account_id(),
        "store_wasm_of_wrapped_appchain_nft_contract",
        &WRAPPED_APPCHAIN_NFT_WASM_BYTES,
        near_sdk_sim::DEFAULT_GAS,
        to_yocto("200"),
    );
    common::print_execution_result("store_wasm_of_wrapped_appchain_nft_contract", &result);
    result.assert_success();
    //
    //
    //
    let class_id = "nft_class1".to_string();
    let result = wrapped_appchain_nft_manager::register_wrapped_appchain_nft(
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
    );
    result.assert_success();
    //
    //
    //
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    let result = settings_manager::turn_on_beefy_light_client_witness_mode(&root, &anchor);
    result.assert_success();
    //
    //
    //
    let mut appchain_messages = Vec::<AppchainMessage>::new();
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NonFungibleTokenLocked {
            owner_id_in_appchain: user0_id_in_appchain.clone(),
            receiver_id_in_near: users[0].account_id(),
            class_id: class_id.clone(),
            instance_id: "token_id1".to_string(),
            token_metadata: TokenMetadata {
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
        },
        nonce: appchain_message_nonce,
    });
    sudo_actions::stage_appchain_messages(&root, &anchor, appchain_messages);
    common::process_appchain_messages(&users[4], &anchor);
    common::print_appchain_messages_processing_results(&anchor);
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
    //
    //
    //
    let result = wrapped_appchain_nft_manager::open_bridging_of_wrapped_appchain_nft(
        &root,
        &anchor,
        class_id.clone(),
    );
    result.assert_success();
    let token_id = "token_id1".to_string();
    let mut appchain_messages = Vec::<AppchainMessage>::new();
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NonFungibleTokenLocked {
            owner_id_in_appchain: user0_id_in_appchain.clone(),
            receiver_id_in_near: users[0].account_id(),
            class_id: class_id.clone(),
            instance_id: token_id.clone(),
            token_metadata: TokenMetadata {
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
        },
        nonce: appchain_message_nonce,
    });
    sudo_actions::stage_appchain_messages(&root, &anchor, appchain_messages);
    common::process_appchain_messages(&users[4], &anchor);
    common::print_appchain_messages_processing_results(&anchor);
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
    //
    //
    //
    let result = users[0].call(
        AccountId::from_str(
            format!("{}.{}", class_id.clone(), anchor.account_id().to_string()).as_str(),
        )
        .unwrap(),
        "nft_transfer_call",
        json!({
            "receiver_id": anchor.account_id(),
            "token_id": token_id.clone(),
            "approval_id": null,
            "memo": null,
            "msg": json!({
                "BridgeToAppchain": {
                    "receiver_id_in_appchain": user0_id_in_appchain.clone()
                }
            }).to_string()
        })
        .to_string()
        .as_bytes(),
        near_sdk_sim::DEFAULT_GAS,
        1,
    );
    common::print_execution_result("nft_transfer_call", &result);
    result.assert_success();
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
    //
    //
    //
    let mut appchain_messages = Vec::<AppchainMessage>::new();
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NonFungibleTokenLocked {
            owner_id_in_appchain: user0_id_in_appchain.clone(),
            receiver_id_in_near: users[0].account_id(),
            class_id: class_id.clone(),
            instance_id: "token_id1".to_string(),
            token_metadata: TokenMetadata {
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
        },
        nonce: appchain_message_nonce,
    });
    sudo_actions::stage_appchain_messages(&root, &anchor, appchain_messages);
    common::process_appchain_messages(&users[4], &anchor);
    common::print_appchain_messages_processing_results(&anchor);
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
}
