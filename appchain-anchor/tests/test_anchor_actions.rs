use appchain_anchor::types::{
    AnchorSettings, AnchorStatus, AppchainSettings, AppchainState, ProtocolSettings,
};
use near_sdk::serde_json;

mod anchor_viewer;
mod common;
mod oct_token_viewer;
mod permissionless_actions;
mod staking_actions;

const TOTAL_SUPPLY: u128 = 100_000_000;

#[test]
fn test_case1() {
    let total_supply = common::to_oct_amount(TOTAL_SUPPLY);
    let (root, oct_token, registry, anchor, users) = common::init(total_supply);
    //
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor),
        AppchainState::Staging
    );
    //
    let anchor_settings = anchor_viewer::get_anchor_settings(&anchor);
    println!(
        "Anchor settings: {}",
        serde_json::to_string::<AnchorSettings>(&anchor_settings).unwrap()
    );
    let appchain_settings = anchor_viewer::get_appchain_settings(&anchor);
    println!(
        "Appchain settings: {}",
        serde_json::to_string::<AppchainSettings>(&appchain_settings).unwrap()
    );
    let protocol_settings = anchor_viewer::get_protocol_settings(&anchor);
    println!(
        "Protocol settings: {}",
        serde_json::to_string::<ProtocolSettings>(&protocol_settings).unwrap()
    );
    //
    assert_eq!(
        protocol_settings.minimum_validator_deposit.0,
        common::to_oct_amount(10_000)
    );
    //
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    assert_eq!(anchor_status.total_stake_in_next_era.0, 0);
    assert_eq!(anchor_status.validator_count_in_next_era.0, 0);
    //
    let user0_balance = oct_token_viewer::get_ft_balance_of(&users[0], &oct_token);
    let amount = common::to_oct_amount(9999);
    let user0_id_in_appchain = "user0_id_in_appchain".to_string();
    let outcome = staking_actions::register_validator(
        &users[0],
        &oct_token,
        &anchor,
        &user0_id_in_appchain,
        amount,
        true,
    );
    outcome.assert_success();
    assert_eq!(
        oct_token_viewer::get_ft_balance_of(&users[0], &oct_token).0,
        user0_balance.0
    );
    let anchor_status = anchor_viewer::get_anchor_status(&anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    assert_eq!(anchor_status.total_stake_in_next_era.0, 0);
    assert_eq!(anchor_status.validator_count_in_next_era.0, 0);
}
