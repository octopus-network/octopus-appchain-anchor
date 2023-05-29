use crate::common::{self, complex_actions};
use near_sdk::{json_types::U64, serde_json::json};
use near_units::parse_near;

#[tokio::test]
async fn test_migration() -> anyhow::Result<()> {
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    let user1_id_in_appchain =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da270".to_string();
    //
    let worker = workspaces::sandbox().await?;
    let (
        root,
        _,
        wrapped_appchain_token,
        _registry,
        _council,
        anchor,
        _wat_faucet,
        users,
        mut appchain_message_nonce,
    ) = common::test_normal_actions(&worker, true, false, vec!["0x00".to_string()]).await?;
    //
    // Try start and complete switching era1
    //
    appchain_message_nonce += 1;
    complex_actions::switch_era(&users[5], &anchor, 1, appchain_message_nonce, false).await;
    //
    // Distribut reward of era0
    //
    appchain_message_nonce += 1;
    complex_actions::distribute_reward_of(
        &users[5],
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        0,
        Vec::new(),
        false,
    )
    .await;
    //
    assert!(root
        .call(anchor.id(), "store_wasm_of_self")
        .args(std::fs::read(format!("res/appchain_anchor.wasm"))?)
        .gas(300_000_000_000_000)
        .deposit(parse_near!("30 N"))
        .transact()
        .await
        .unwrap()
        .is_success());
    //
    let result = root
        .call(anchor.id(), "update_self")
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("Result of calling 'update_self': {:?}", result);
    println!();
    assert!(result.is_success());
    //
    let result = anchor
        .call("migrate_appchain_messages")
        .args_json(json!({
            "start_nonce": 0,
        }))
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!(
        "Result of calling 'migrate_appchain_messages': {:?}",
        result
    );
    println!();
    assert!(result.is_success());
    //
    //
    //
    common::complex_viewer::print_anchor_status(&anchor).await;
    common::complex_viewer::print_wrapped_appchain_token_info(&anchor).await;
    common::complex_viewer::print_appchain_settings(&anchor).await;
    common::complex_viewer::print_anchor_settings(&anchor).await;
    common::complex_viewer::print_validator_set_info_of(&anchor, U64::from(0)).await;
    common::complex_viewer::print_validator_list_of(&anchor, Some(0)).await;
    common::complex_viewer::print_validator_list_of(&anchor, Some(1)).await;
    common::complex_viewer::print_validator_list_of(&anchor, Some(2)).await;
    common::complex_viewer::print_validator_list_of(&anchor, Some(3)).await;
    common::complex_viewer::print_user_staking_histories_of(&anchor, &users[0]).await;
    common::complex_viewer::print_user_staking_histories_of(&anchor, &users[1]).await;
    common::complex_viewer::print_user_staking_histories_of(&anchor, &users[2]).await;
    common::complex_viewer::print_user_staking_histories_of(&anchor, &users[3]).await;
    common::complex_viewer::print_user_staking_histories_of(&anchor, &users[4]).await;
    common::complex_viewer::print_validator_profile(
        &anchor,
        &users[0].id().to_string().parse().unwrap(),
        &user0_id_in_appchain,
    )
    .await;
    common::complex_viewer::print_validator_profile(
        &anchor,
        &users[1].id().to_string().parse().unwrap(),
        &user1_id_in_appchain,
    )
    .await;
    common::complex_viewer::print_appchain_messages(&anchor).await;
    Ok(())
}
