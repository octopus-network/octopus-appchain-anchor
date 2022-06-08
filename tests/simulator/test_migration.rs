use crate::common;
use near_sdk::json_types::U64;

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
    common::basic_actions::deploy_new_anchor_contract(&worker, &anchor).await?;
    let result = root
        .call(&worker, anchor.id(), "migrate_state")
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    assert!(result.is_success());
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
