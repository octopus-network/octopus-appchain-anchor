use appchain_anchor::types::{
    AnchorSettings, AnchorStatus, AppchainDelegator, AppchainSettings, AppchainState,
    AppchainValidator, ProtocolSettings, RewardHistory, StakingHistory, UnbondedStake,
    ValidatorSetInfo, ValidatorSetProcessingStatus,
};
use appchain_anchor::AppchainAnchorContract;

use near_sdk::json_types::U64;
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

pub fn get_processing_status_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    index: u64,
) -> ValidatorSetProcessingStatus {
    let view_result = view!(anchor.get_processing_status_of(U64::from(index)));
    assert!(view_result.is_ok());
    view_result.unwrap_json::<ValidatorSetProcessingStatus>()
}

pub fn get_validator_set_info_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    index: u64,
) -> ValidatorSetInfo {
    let view_result = view!(anchor.get_validator_set_info_of(U64::from(index)));
    assert!(view_result.is_ok());
    view_result.unwrap_json::<ValidatorSetInfo>()
}

pub fn get_staking_history(
    anchor: &ContractAccount<AppchainAnchorContract>,
    index: u64,
) -> StakingHistory {
    let view_result = view!(anchor.get_staking_history(Some(U64::from(index))));
    assert!(view_result.is_ok());
    view_result.unwrap_json::<StakingHistory>()
}

pub fn get_validator_list_of_era(
    anchor: &ContractAccount<AppchainAnchorContract>,
    index: u64,
) -> Vec<AppchainValidator> {
    let view_result = view!(anchor.get_validator_list_of_era(U64::from(index)));
    assert!(view_result.is_ok());
    view_result.unwrap_json::<Vec<AppchainValidator>>()
}

pub fn get_delegators_of_validator_in_era(
    anchor: &ContractAccount<AppchainAnchorContract>,
    index: u64,
    validator: &UserAccount,
) -> Vec<AppchainDelegator> {
    let view_result = view!(anchor.get_delegators_of_validator_in_era(
        Some(U64::from(index)),
        validator.valid_account_id().to_string()
    ));
    assert!(view_result.is_ok());
    view_result.unwrap_json::<Vec<AppchainDelegator>>()
}

pub fn get_unbonded_stakes_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    account: &UserAccount,
) -> Vec<UnbondedStake> {
    let view_result = view!(anchor.get_unbonded_stakes_of(account.valid_account_id().to_string()));
    assert!(view_result.is_ok());
    view_result.unwrap_json::<Vec<UnbondedStake>>()
}

pub fn get_validator_rewards_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    start_era: u64,
    end_era: u64,
    validator: &UserAccount,
) -> Vec<RewardHistory> {
    let view_result = view!(anchor.get_validator_rewards_of(
        U64::from(start_era),
        U64::from(end_era),
        validator.valid_account_id().to_string()
    ));
    assert!(view_result.is_ok());
    view_result.unwrap_json::<Vec<RewardHistory>>()
}

pub fn get_delegator_rewards_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    start_era: u64,
    end_era: u64,
    delegator: &UserAccount,
    validator: &UserAccount,
) -> Vec<RewardHistory> {
    let view_result = view!(anchor.get_delegator_rewards_of(
        U64::from(start_era),
        U64::from(end_era),
        delegator.valid_account_id().to_string(),
        validator.valid_account_id().to_string()
    ));
    assert!(view_result.is_ok());
    view_result.unwrap_json::<Vec<RewardHistory>>()
}
