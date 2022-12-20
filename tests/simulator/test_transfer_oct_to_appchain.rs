use crate::{common, contract_interfaces::near_fungible_token_manager};
use near_sdk::{json_types::U128, serde_json::json};

#[tokio::test]
async fn test_transfer_oct_to_appchain() -> anyhow::Result<()> {
    //
    let worker = workspaces::sandbox().await?;
    let (root, oct_token, _, _registry, _council, anchor, _wat_faucet, users, _) =
        common::test_normal_actions(&worker, false, false).await?;
    //
    assert!(near_fungible_token_manager::register_near_fungible_token(
        &root,
        &anchor,
        "OCT".to_string(),
        "Oct token".to_string(),
        18,
        oct_token.id().to_string().parse().unwrap(),
        U128::from(1000000),
    )
    .await
    .unwrap()
    .is_success());
    common::complex_viewer::print_near_fungible_tokens(&anchor).await;
    //
    assert!(common::call_ft_transfer_call(
        &users[0],
        &anchor.as_account(),
        common::to_actual_amount(200, 18),
        json!({
            "BridgeToAppchain": {
                "receiver_id_in_appchain": "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string(),
            }
        })
        .to_string(),
        &oct_token,
    ).await.unwrap().is_success());
    common::complex_viewer::print_appchain_notifications(&anchor).await;
    Ok(())
}
