use appchain_anchor::appchain_challenge::AppchainChallenge;
use appchain_anchor::types::{
    AnchorSettings, AnchorStatus, AppchainDelegator, AppchainMessageProcessingResult,
    AppchainNotificationHistory, AppchainSettings, AppchainState, AppchainValidator, IndexRange,
    NativeNearToken, NearFungibleToken, RewardHistory, StakingHistory, UnbondedStake,
    UserStakingHistory, ValidatorProfile, ValidatorSetInfo, WrappedAppchainToken,
};
use appchain_anchor::AppchainMessage;
use near_sdk::json_types::U64;
use near_sdk::serde_json::json;
use near_sdk::AccountId;
use workspaces::{error::Error, Account, Contract};

pub async fn get_anchor_settings(anchor: &Contract) -> Result<AnchorSettings, Error> {
    anchor
        .call("get_anchor_settings")
        .view()
        .await?
        .json::<AnchorSettings>()
}

pub async fn get_appchain_settings(anchor: &Contract) -> Result<AppchainSettings, Error> {
    anchor
        .call("get_appchain_settings")
        .view()
        .await?
        .json::<AppchainSettings>()
}

pub async fn get_wrapped_appchain_token(anchor: &Contract) -> Result<WrappedAppchainToken, Error> {
    anchor
        .call("get_wrapped_appchain_token")
        .view()
        .await?
        .json::<WrappedAppchainToken>()
}

pub async fn get_near_fungible_tokens(anchor: &Contract) -> Result<Vec<NearFungibleToken>, Error> {
    anchor
        .call("get_near_fungible_tokens")
        .view()
        .await?
        .json::<Vec<NearFungibleToken>>()
}

pub async fn get_native_near_token(anchor: &Contract) -> Result<NativeNearToken, Error> {
    anchor
        .call("get_native_near_token")
        .view()
        .await?
        .json::<NativeNearToken>()
}

pub async fn get_appchain_state(anchor: &Contract) -> Result<AppchainState, Error> {
    anchor
        .call("get_appchain_state")
        .view()
        .await?
        .json::<AppchainState>()
}

pub async fn get_anchor_status(anchor: &Contract) -> Result<AnchorStatus, Error> {
    anchor
        .call("get_anchor_status")
        .view()
        .await?
        .json::<AnchorStatus>()
}

pub async fn get_validator_set_info_of(
    anchor: &Contract,
    index: U64,
) -> Result<ValidatorSetInfo, Error> {
    anchor
        .call("get_validator_set_info_of")
        .args_json(json!({ "era_number": index }))
        .view()
        .await?
        .json::<ValidatorSetInfo>()
}

pub async fn get_index_range_of_appchain_notification_history(
    anchor: &Contract,
) -> Result<IndexRange, Error> {
    anchor
        .call("get_index_range_of_appchain_notification_history")
        .view()
        .await?
        .json::<IndexRange>()
}

pub async fn get_appchain_notification_history(
    anchor: &Contract,
    index: u64,
) -> Result<Option<AppchainNotificationHistory>, Error> {
    anchor
        .call("get_appchain_notification_history")
        .args_json(json!({ "index": Some(U64::from(index)) }))
        .view()
        .await?
        .json::<Option<AppchainNotificationHistory>>()
}

pub async fn get_appchain_notification_histories(
    anchor: &Contract,
    start_index: u64,
    quantity: Option<U64>,
) -> Result<Vec<AppchainNotificationHistory>, Error> {
    anchor
        .call("get_appchain_notification_histories")
        .args_json(json!({
            "start_index": U64::from(start_index),
            "quantity": quantity
        }))
        .view()
        .await?
        .json::<Vec<AppchainNotificationHistory>>()
}

pub async fn get_index_range_of_staking_history(anchor: &Contract) -> Result<IndexRange, Error> {
    anchor
        .call("get_index_range_of_staking_history")
        .view()
        .await?
        .json::<IndexRange>()
}

pub async fn get_staking_history(
    anchor: &Contract,
    index: u64,
) -> Result<Option<StakingHistory>, Error> {
    anchor
        .call("get_staking_history")
        .args_json(json!({ "index": Some(U64::from(index)) }))
        .view()
        .await?
        .json::<Option<StakingHistory>>()
}

pub async fn get_validator_list_of(
    anchor: &Contract,
    index: Option<u64>,
) -> Result<Vec<AppchainValidator>, Error> {
    let index = index.map_or(None, |i| Some(U64::from(i)));
    anchor
        .call("get_validator_list_of")
        .args_json(json!({
            "era_number": index.map_or_else(|| Option::<U64>::None, |i| Some(U64::from(i)))
        }))
        .view()
        .await?
        .json::<Vec<AppchainValidator>>()
}

pub async fn get_validator_profile(
    anchor: &Contract,
    account_id: &AccountId,
) -> Result<Option<ValidatorProfile>, Error> {
    anchor
        .call("get_validator_profile")
        .args_json(json!({ "validator_id": account_id }))
        .view()
        .await?
        .json::<Option<ValidatorProfile>>()
}

pub async fn get_validator_profile_by_id_in_appchain(
    anchor: &Contract,
    account_id_in_appchain: &String,
) -> Result<Option<ValidatorProfile>, Error> {
    anchor
        .call("get_validator_profile_by_id_in_appchain")
        .args_json(json!({
            "validator_id_in_appchain": account_id_in_appchain
        }))
        .view()
        .await?
        .json::<Option<ValidatorProfile>>()
}

pub async fn get_delegators_of_validator_in_era(
    anchor: &Contract,
    index: u64,
    validator: &Account,
) -> Result<Vec<AppchainDelegator>, Error> {
    anchor
        .call("get_delegators_of_validator_in_era")
        .args_json(json!({
            "era_number": Some(U64::from(index)),
            "validator_id": validator.id()
        }))
        .view()
        .await?
        .json::<Vec<AppchainDelegator>>()
}

pub async fn get_unbonded_stakes_of(
    anchor: &Contract,
    account: &Account,
) -> Result<Vec<UnbondedStake>, Error> {
    anchor
        .call("get_unbonded_stakes_of")
        .args_json(json!({
            "account_id": account.id()
        }))
        .view()
        .await?
        .json::<Vec<UnbondedStake>>()
}

pub async fn get_validator_rewards_of(
    anchor: &Contract,
    start_era: u64,
    end_era: u64,
    validator: &Account,
) -> Result<Vec<RewardHistory>, Error> {
    anchor
        .call("get_validator_rewards_of")
        .args_json(json!({
            "start_era": U64::from(start_era),
            "end_era": U64::from(end_era),
            "validator_id": validator.id()
        }))
        .view()
        .await?
        .json::<Vec<RewardHistory>>()
}

pub async fn get_delegator_rewards_of(
    anchor: &Contract,
    start_era: u64,
    end_era: u64,
    delegator: &Account,
    validator: &Account,
) -> Result<Vec<RewardHistory>, Error> {
    anchor
        .call("get_delegator_rewards_of")
        .args_json(json!({
            "start_era": U64::from(start_era),
            "end_era": U64::from(end_era),
            "delegator_id": delegator.id(),
            "validator_id": validator.id()
        }))
        .view()
        .await?
        .json::<Vec<RewardHistory>>()
}

pub async fn get_user_staking_histories_of(
    anchor: &Contract,
    account_id: AccountId,
) -> Result<Vec<UserStakingHistory>, Error> {
    anchor
        .call("get_user_staking_histories_of")
        .args_json(json!({ "account_id": account_id }))
        .view()
        .await?
        .json::<Vec<UserStakingHistory>>()
}

pub async fn get_appchain_messages(
    anchor: &Contract,
    start_nonce: u32,
    quantity: Option<u32>,
) -> Result<Vec<AppchainMessage>, Error> {
    anchor
        .call("get_appchain_messages")
        .args_json(json!({
            "start_nonce": start_nonce,
            "quantity": quantity
        }))
        .view()
        .await?
        .json::<Vec<AppchainMessage>>()
}

pub async fn get_appchain_message_processing_results(
    anchor: &Contract,
    start_nonce: u32,
    quantity: Option<u32>,
) -> Result<Vec<AppchainMessageProcessingResult>, Error> {
    anchor
        .call("get_appchain_message_processing_results")
        .args_json(json!({
            "start_nonce": start_nonce,
            "quantity": quantity
        }))
        .view()
        .await?
        .json::<Vec<AppchainMessageProcessingResult>>()
}

pub async fn get_appchain_challenge(
    anchor: &Contract,
    index: u64,
) -> Result<Option<AppchainChallenge>, Error> {
    anchor
        .call("get_appchain_challenge")
        .args_json(json!({ "index": Some(U64::from(index)) }))
        .view()
        .await?
        .json::<Option<AppchainChallenge>>()
}

pub async fn get_appchain_challenges(
    anchor: &Contract,
    start_index: u64,
    quantity: Option<U64>,
) -> Result<Vec<AppchainChallenge>, Error> {
    anchor
        .call("get_appchain_challenges")
        .args_json(json!({
            "start_index": U64::from(start_index),
            "quantity": quantity
        }))
        .view()
        .await?
        .json::<Vec<AppchainChallenge>>()
}
