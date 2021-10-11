use appchain_anchor::types::{
    AnchorSettings, AnchorStatus, AppchainSettings, AppchainState, ProtocolSettings, StakingHistory,
};
use appchain_anchor::AppchainAnchorContract;

use near_sdk::json_types::{U128, U64};
use near_sdk_sim::{view, ContractAccount, UserAccount};

pub fn get_anchor_settings(anchor: &ContractAccount<AppchainAnchorContract>) -> AnchorSettings {
    let view_result = view!(anchor.get_anchor_settings());
    assert!(view_result.is_ok());
    view_result.unwrap_json::<AnchorSettings>()
}

pub fn get_appchain_settings(anchor: &ContractAccount<AppchainAnchorContract>) -> AppchainSettings {
    let view_result = view!(anchor.get_appchain_settings());
    assert!(view_result.is_ok());
    view_result.unwrap_json::<AppchainSettings>()
}

pub fn get_protocol_settings(anchor: &ContractAccount<AppchainAnchorContract>) -> ProtocolSettings {
    let view_result = view!(anchor.get_protocol_settings());
    assert!(view_result.is_ok());
    view_result.unwrap_json::<ProtocolSettings>()
}

pub fn get_appchain_state(anchor: &ContractAccount<AppchainAnchorContract>) -> AppchainState {
    let view_result = view!(anchor.get_appchain_state());
    assert!(view_result.is_ok());
    view_result.unwrap_json::<AppchainState>()
}

pub fn get_anchor_status(anchor: &ContractAccount<AppchainAnchorContract>) -> AnchorStatus {
    let view_result = view!(anchor.get_anchor_status());
    assert!(view_result.is_ok());
    view_result.unwrap_json::<AnchorStatus>()
}

pub fn get_staking_history(
    anchor: &ContractAccount<AppchainAnchorContract>,
    index: u64,
) -> StakingHistory {
    let view_result = view!(anchor.get_staking_history(Some(U64::from(index))));
    assert!(view_result.is_ok());
    view_result.unwrap_json::<StakingHistory>()
}
