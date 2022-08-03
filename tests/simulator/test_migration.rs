use crate::common;
use near_sdk::json_types::U64;
use near_units::parse_near;

#[tokio::test]
async fn test_migration() -> anyhow::Result<()> {
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    let user1_id_in_appchain =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da270".to_string();
    //
    let worker = workspaces::sandbox().await?;
    let (root, _, _, _, anchor, users, _) =
        common::test_normal_actions(&worker, true, false, vec!["0x00".to_string()]).await?;
    //
    root.call(&worker, anchor.id(), "store_wasm_of_self")
        .args(std::fs::read(format!("res/appchain_anchor.wasm"))?)
        .gas(300_000_000_000_000)
        .deposit(parse_near!("30 N"))
        .transact()
        .await
        .expect("Failed in calling 'store_wasm_of_self'");
    //
    let result = root
        .call(&worker, anchor.id(), "update_self")
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("Result of calling 'update_self': {:?}", result);
    println!();
    assert!(result.is_success());
    //
    anchor
        .call(&worker, "migrate_staking_histories")
        .gas(200_000_000_000_000)
        .transact()
        .await
        .expect("Failed to call 'migrate_staking_histories'");
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    //
    anchor
        .call(&worker, "migrate_appchain_notification_histories")
        .gas(200_000_000_000_000)
        .transact()
        .await
        .expect("Failed to call 'migrate_appchain_notification_histories'");
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    //
    //
    //
    common::complex_viewer::print_anchor_status(&worker, &anchor).await?;
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_settings(&worker, &anchor).await?;
    common::complex_viewer::print_anchor_settings(&worker, &anchor).await?;
    common::complex_viewer::print_validator_set_info_of(&worker, &anchor, U64::from(0)).await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(0)).await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(1)).await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(2)).await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(3)).await?;
    common::complex_viewer::print_user_staking_histories_of(&worker, &anchor, &users[0]).await?;
    common::complex_viewer::print_user_staking_histories_of(&worker, &anchor, &users[1]).await?;
    common::complex_viewer::print_user_staking_histories_of(&worker, &anchor, &users[2]).await?;
    common::complex_viewer::print_user_staking_histories_of(&worker, &anchor, &users[3]).await?;
    common::complex_viewer::print_user_staking_histories_of(&worker, &anchor, &users[4]).await?;
    common::complex_viewer::print_validator_profile(
        &worker,
        &anchor,
        &users[0].id().to_string().parse().unwrap(),
        &user0_id_in_appchain,
    )
    .await?;
    common::complex_viewer::print_validator_profile(
        &worker,
        &anchor,
        &users[1].id().to_string().parse().unwrap(),
        &user1_id_in_appchain,
    )
    .await?;
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    Ok(())
}
