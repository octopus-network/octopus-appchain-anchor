use appchain_anchor::appchain_challenge::AppchainChallenge;
use appchain_anchor::types::{
    AnchorSettings, AnchorStatus, AppchainCommitment, AppchainDelegator,
    AppchainMessageProcessingResult, AppchainNotificationHistory, AppchainSettings, AppchainState,
    AppchainValidator, IndexRange, NearFungibleToken, RewardHistory, StakingHistory, UnbondedStake,
    UserStakingHistory, ValidatorProfile, ValidatorSetInfo, WrappedAppchainToken,
};
use appchain_anchor::AppchainMessage;
use near_sdk::json_types::U64;
use near_sdk::serde_json::json;
use near_sdk::AccountId;
use workspaces::{network::Sandbox, Account, Contract, Worker};

pub async fn get_anchor_settings(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<AnchorSettings> {
    anchor
        .call(worker, "get_anchor_settings")
        .view()
        .await?
        .json::<AnchorSettings>()
}

pub async fn get_appchain_settings(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<AppchainSettings> {
    anchor
        .call(worker, "get_appchain_settings")
        .view()
        .await?
        .json::<AppchainSettings>()
}

pub async fn get_wrapped_appchain_token(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<WrappedAppchainToken> {
    anchor
        .call(worker, "get_wrapped_appchain_token")
        .view()
        .await?
        .json::<WrappedAppchainToken>()
}

pub async fn get_near_fungible_tokens(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<Vec<NearFungibleToken>> {
    anchor
        .call(worker, "get_near_fungible_tokens")
        .view()
        .await?
        .json::<Vec<NearFungibleToken>>()
}

pub async fn get_appchain_state(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<AppchainState> {
    anchor
        .call(worker, "get_appchain_state")
        .view()
        .await?
        .json::<AppchainState>()
}

pub async fn get_anchor_status(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<AnchorStatus> {
    anchor
        .call(worker, "get_anchor_status")
        .view()
        .await?
        .json::<AnchorStatus>()
}

pub async fn get_validator_set_info_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    index: U64,
) -> anyhow::Result<ValidatorSetInfo> {
    anchor
        .call(worker, "get_validator_set_info_of")
        .args_json(json!({ "era_number": index }))?
        .view()
        .await?
        .json::<ValidatorSetInfo>()
}

pub async fn get_index_range_of_appchain_notification_history(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<IndexRange> {
    anchor
        .call(worker, "get_index_range_of_appchain_notification_history")
        .view()
        .await?
        .json::<IndexRange>()
}

pub async fn get_appchain_notification_history(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    index: u64,
) -> anyhow::Result<Option<AppchainNotificationHistory>> {
    anchor
        .call(worker, "get_appchain_notification_history")
        .args_json(json!({ "index": Some(U64::from(index)) }))?
        .view()
        .await?
        .json::<Option<AppchainNotificationHistory>>()
}

pub async fn get_appchain_notification_histories(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    start_index: u64,
    quantity: Option<U64>,
) -> anyhow::Result<Vec<AppchainNotificationHistory>> {
    anchor
        .call(worker, "get_appchain_notification_histories")
        .args_json(json!({
            "start_index": U64::from(start_index),
            "quantity": quantity
        }))?
        .view()
        .await?
        .json::<Vec<AppchainNotificationHistory>>()
}

pub async fn get_index_range_of_staking_history(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<IndexRange> {
    anchor
        .call(worker, "get_index_range_of_staking_history")
        .view()
        .await?
        .json::<IndexRange>()
}

pub async fn get_staking_history(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    index: u64,
) -> anyhow::Result<Option<StakingHistory>> {
    anchor
        .call(worker, "get_staking_history")
        .args_json(json!({ "index": Some(U64::from(index)) }))?
        .view()
        .await?
        .json::<Option<StakingHistory>>()
}

pub async fn get_validator_list_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    index: Option<u64>,
) -> anyhow::Result<Vec<AppchainValidator>> {
    let index = index.map_or(None, |i| Some(U64::from(i)));
    anchor
        .call(worker, "get_validator_list_of")
        .args_json(json!({
            "era_number": index.map_or_else(|| Option::<U64>::None, |i| Some(U64::from(i)))
        }))?
        .view()
        .await?
        .json::<Vec<AppchainValidator>>()
}

pub async fn get_validator_profile(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    account_id: &AccountId,
) -> anyhow::Result<Option<ValidatorProfile>> {
    anchor
        .call(worker, "get_validator_profile")
        .args_json(json!({ "validator_id": account_id }))?
        .view()
        .await?
        .json::<Option<ValidatorProfile>>()
}

pub async fn get_validator_profile_by_id_in_appchain(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    account_id_in_appchain: &String,
) -> anyhow::Result<Option<ValidatorProfile>> {
    anchor
        .call(worker, "get_validator_profile_by_id_in_appchain")
        .args_json(json!({
            "validator_id_in_appchain": account_id_in_appchain
        }))?
        .view()
        .await?
        .json::<Option<ValidatorProfile>>()
}

pub async fn get_delegators_of_validator_in_era(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    index: u64,
    validator: &Account,
) -> anyhow::Result<Vec<AppchainDelegator>> {
    anchor
        .call(worker, "get_delegators_of_validator_in_era")
        .args_json(json!({
            "era_number": Some(U64::from(index)),
            "validator_id": validator.id()
        }))?
        .view()
        .await?
        .json::<Vec<AppchainDelegator>>()
}

pub async fn get_unbonded_stakes_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    account: &Account,
) -> anyhow::Result<Vec<UnbondedStake>> {
    anchor
        .call(worker, "get_unbonded_stakes_of")
        .args_json(json!({
            "account_id": account.id()
        }))?
        .view()
        .await?
        .json::<Vec<UnbondedStake>>()
}

pub async fn get_validator_rewards_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    start_era: u64,
    end_era: u64,
    validator: &Account,
) -> anyhow::Result<Vec<RewardHistory>> {
    anchor
        .call(worker, "get_validator_rewards_of")
        .args_json(json!({
            "start_era": U64::from(start_era),
            "end_era": U64::from(end_era),
            "validator_id": validator.id()
        }))?
        .view()
        .await?
        .json::<Vec<RewardHistory>>()
}

pub async fn get_delegator_rewards_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    start_era: u64,
    end_era: u64,
    delegator: &Account,
    validator: &Account,
) -> anyhow::Result<Vec<RewardHistory>> {
    anchor
        .call(worker, "get_delegator_rewards_of")
        .args_json(json!({
            "start_era": U64::from(start_era),
            "end_era": U64::from(end_era),
            "delegator_id": delegator.id(),
            "validator_id": validator.id()
        }))?
        .view()
        .await?
        .json::<Vec<RewardHistory>>()
}

pub async fn get_latest_commitment_of_appchain(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<Option<AppchainCommitment>> {
    anchor
        .call(worker, "get_latest_commitment_of_appchain")
        .view()
        .await?
        .json::<Option<AppchainCommitment>>()
}

pub async fn get_user_staking_histories_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    account_id: AccountId,
) -> anyhow::Result<Vec<UserStakingHistory>> {
    anchor
        .call(worker, "get_user_staking_histories_of")
        .args_json(json!({ "account_id": account_id }))?
        .view()
        .await?
        .json::<Vec<UserStakingHistory>>()
}

pub async fn get_appchain_messages(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    start_nonce: u32,
    quantity: Option<u32>,
) -> anyhow::Result<Vec<AppchainMessage>> {
    anchor
        .call(worker, "get_appchain_messages")
        .args_json(json!({
            "start_nonce": start_nonce,
            "quantity": quantity
        }))?
        .view()
        .await?
        .json::<Vec<AppchainMessage>>()
}

pub async fn get_appchain_message_processing_results(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    start_nonce: u32,
    quantity: Option<u32>,
) -> anyhow::Result<Vec<AppchainMessageProcessingResult>> {
    anchor
        .call(worker, "get_appchain_message_processing_results")
        .args_json(json!({
            "start_nonce": start_nonce,
            "quantity": quantity
        }))?
        .view()
        .await?
        .json::<Vec<AppchainMessageProcessingResult>>()
}

pub async fn get_appchain_challenge(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    index: u64,
) -> anyhow::Result<Option<AppchainChallenge>> {
    anchor
        .call(worker, "get_appchain_challenge")
        .args_json(json!({ "index": Some(U64::from(index)) }))?
        .view()
        .await?
        .json::<Option<AppchainChallenge>>()
}

pub async fn get_appchain_challenges(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    start_index: u64,
    quantity: Option<U64>,
) -> anyhow::Result<Vec<AppchainChallenge>> {
    anchor
        .call(worker, "get_appchain_challenges")
        .args_json(json!({
            "start_index": U64::from(start_index),
            "quantity": quantity
        }))?
        .view()
        .await?
        .json::<Vec<AppchainChallenge>>()
}
