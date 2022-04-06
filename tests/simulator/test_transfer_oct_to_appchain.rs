use near_sdk::{json_types::U128, serde_json::json};

use crate::{common, owner_actions};

#[test]
fn test_transfer_oct_to_appchain() {
    //
    let (root, oct_token, _, _, anchor, users, _) = common::test_normal_actions(false, false);
    //
    let execution_result = owner_actions::register_near_fungible_token(
        &root,
        &anchor,
        "OCT".to_string(),
        "Oct token".to_string(),
        18,
        oct_token.account_id(),
        U128::from(1000000),
    );
    assert!(execution_result.is_ok());
    common::print_near_fungible_tokens(&anchor);
    //
    let execution_result = common::ft_transfer_call_oct_token(
        &users[0],
        &anchor.user_account,
        common::to_oct_amount(200),
        json!({
            "BridgeToAppchain": {
                "receiver_id_in_appchain": "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string(),
            }
        })
        .to_string(),
        &oct_token,
    );
    assert!(execution_result.is_ok());
    common::print_anchor_events(&anchor);
    common::print_appchain_notifications(&anchor);
}
