use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{env, near_bindgen, AccountId, Balance};

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum OldAppchainState {
    /// The initial state of an appchain, after it is successfully registered.
    /// This state is managed by appchain registry.
    Registered,
    /// The state while the appchain is under auditing by Octopus Network.
    /// This state is managed by appchain registry.
    Auditing,
    /// The state while voter can upvote or downvote an appchain.
    /// This state is managed by appchain registry.
    InQueue,
    /// The state while validator and delegator can deposit OCT tokens to this contract
    /// to indicate their willing of staking for an appchain.
    Staging,
    /// The state while an appchain is booting.
    Booting,
    /// The state while an appchain is active normally.
    Active,
    /// The state while an appchain is under challenging, which all deposit and withdraw actions
    /// are frozen.
    Frozen,
    /// The state which an appchain is broken for some technical or governance reasons.
    Broken,
    /// The state which the lifecycle of an appchain is end.
    Dead,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OldProtocolSettings {
    /// A validator has to deposit a certain amount of OCT token to this contract for
    /// being validator of the appchain.
    pub minimum_validator_deposit: U128,
    /// The minimum amount for a validator to increase or decrease his/her deposit.
    pub minimum_validator_deposit_changing_amount: U128,
    /// The maximum percent value that the deposit of a validator in total stake
    pub maximum_validator_stake_percent: u16,
    /// The minimum deposit amount for a delegator to delegate his voting weight to
    /// a certain validator.
    pub minimum_delegator_deposit: U128,
    /// The minimum amount for a delegator to increase or decrease his/her delegation
    /// to a validator.
    pub minimum_delegator_deposit_changing_amount: U128,
    /// The minimum price (in USD) of total stake in this contract for
    /// booting corresponding appchain
    pub minimum_total_stake_price_for_booting: U128,
    /// The maximum percentage of the total market value of all NEP-141 tokens to the total
    /// market value of OCT token staked in this contract
    pub maximum_market_value_percent_of_near_fungible_tokens: u16,
    /// The maximum percentage of the total market value of wrapped appchain token to the total
    /// market value of OCT token staked in this contract
    pub maximum_market_value_percent_of_wrapped_appchain_token: u16,
    /// The minimum number of validator(s) registered in this contract for
    /// booting the corresponding appchain and keep it alive.
    pub minimum_validator_count: U64,
    /// The maximum number of validator(s) registered in this contract for
    /// the corresponding appchain.
    pub maximum_validator_count: U64,
    /// The maximum number of validator(s) which a delegator can delegate to.
    pub maximum_validators_per_delegator: U64,
    /// The unlock period (in days) for validator(s) can withdraw their deposit after
    /// they are removed from the corresponding appchain.
    pub unlock_period_of_validator_deposit: U64,
    /// The unlock period (in days) for delegator(s) can withdraw their deposit after
    /// they no longer delegates their stake to a certain validator on the corresponding appchain.
    pub unlock_period_of_delegator_deposit: U64,
    /// The maximum number of historical eras that the validators or delegators are allowed to
    /// withdraw their reward
    pub maximum_era_count_of_unwithdrawn_reward: U64,
    /// The maximum number of valid appchain message.
    /// If the era number of appchain message is smaller than the latest era number minus
    /// this value, the message will be considered as `invalid`.
    pub maximum_era_count_of_valid_appchain_message: U64,
    /// The percent of commission fees of a validator's reward in an era
    pub validator_commission_percent: u16,
    /// The maximum unprofitable era count for auto-unbonding a validator
    pub maximum_allowed_unprofitable_era_count: u16,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldAppchainAnchor {
    /// The id of corresponding appchain.
    appchain_id: AppchainId,
    /// The type of appchain template of corresponding appchain.
    appchain_template_type: AppchainTemplateType,
    /// The account id of appchain registry contract.
    appchain_registry: AccountId,
    /// The owner account id.
    owner: AccountId,
    /// A certain public key of owner account
    owner_pk: PublicKey,
    /// The info of OCT token.
    oct_token: LazyOption<OctToken>,
    /// The info of wrapped appchain token in NEAR protocol.
    wrapped_appchain_token: LazyOption<WrappedAppchainToken>,
    /// The NEP-141 tokens data.
    near_fungible_tokens: LazyOption<NearFungibleTokens>,
    /// The history data of validator set.
    validator_set_histories: LazyOption<LookupArray<ValidatorSetOfEra>>,
    /// The validator set of the next era in appchain.
    /// This validator set is only for checking staking rules.
    next_validator_set: LazyOption<NextValidatorSet>,
    /// The map of unwithdrawn validator rewards in eras, in unit of wrapped appchain token.
    /// The key in map is `(era_number, account_id_of_validator)`
    unwithdrawn_validator_rewards: LookupMap<(u64, AccountId), Balance>,
    /// The map of unwithdrawn delegator rewards in eras, in unit of wrapped appchain token.
    /// The key in map is `(era_number, account_id_of_delegator, account_id_of_validator)`
    unwithdrawn_delegator_rewards: LookupMap<(u64, AccountId, AccountId), Balance>,
    /// The map of unbonded stakes in eras.
    unbonded_stakes: LookupMap<AccountId, Vec<UnbondedStakeReference>>,
    /// The validators' profiles data.
    validator_profiles: LazyOption<ValidatorProfiles>,
    /// The custom settings for appchain.
    appchain_settings: LazyOption<AppchainSettings>,
    /// The anchor settings for appchain.
    anchor_settings: LazyOption<AnchorSettings>,
    /// The protocol settings for appchain anchor.
    protocol_settings: LazyOption<OldProtocolSettings>,
    /// The state of the corresponding appchain.
    appchain_state: OldAppchainState,
    /// The staking history data happened in this contract.
    staking_histories: LazyOption<LookupArray<StakingHistory>>,
    /// The appchain notification history data.
    appchain_notification_histories: LazyOption<LookupArray<AppchainNotificationHistory>>,
    /// The status of permissionless actions.
    permissionless_actions_status: LazyOption<PermissionlessActionsStatus>,
    /// The state of beefy light client
    beefy_light_client_state: LazyOption<LightClient>,
    /// The reward distribution records data
    reward_distribution_records: LazyOption<RewardDistributionRecords>,
    /// Whether the asset transfer is paused
    asset_transfer_is_paused: bool,
    /// The staking histories organized by account id
    user_staking_histories: LazyOption<UserStakingHistories>,
    /// Whether the rewards withdrawal is paused
    rewards_withdrawal_is_paused: bool,
    /// The processing result of appchain messages
    appchain_messages: LazyOption<AppchainMessages>,
    /// The appchain challenges
    appchain_challenges: LazyOption<LookupArray<AppchainChallenge>>,
    /// The wrapped appchain NFT data
    wrapped_appchain_nfts: LazyOption<WrappedAppchainNFTs>,
    /// The native NEAR token data
    native_near_token: LazyOption<NativeNearToken>,
}

#[near_bindgen]
impl AppchainAnchor {
    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let old_contract: OldAppchainAnchor = env::state_read().expect("Old state doesn't exist");
        //
        near_sdk::assert_self();
        //
        // Create the new contract using the data from the old contract.
        let new_contract = AppchainAnchor {
            appchain_id: old_contract.appchain_id,
            appchain_template_type: old_contract.appchain_template_type,
            appchain_registry: old_contract.appchain_registry,
            owner: old_contract.owner,
            owner_pk: old_contract.owner_pk,
            oct_token: old_contract.oct_token,
            wrapped_appchain_token: old_contract.wrapped_appchain_token,
            near_fungible_tokens: old_contract.near_fungible_tokens,
            validator_set_histories: old_contract.validator_set_histories,
            next_validator_set: old_contract.next_validator_set,
            unwithdrawn_validator_rewards: old_contract.unwithdrawn_validator_rewards,
            unwithdrawn_delegator_rewards: old_contract.unwithdrawn_delegator_rewards,
            unbonded_stakes: old_contract.unbonded_stakes,
            validator_profiles: old_contract.validator_profiles,
            appchain_settings: old_contract.appchain_settings,
            anchor_settings: old_contract.anchor_settings,
            protocol_settings: LazyOption::new(
                StorageKey::ProtocolSettings.into_bytes(),
                Some(&ProtocolSettings::from_old_version(
                    old_contract.protocol_settings.get().unwrap(),
                )),
            ),
            appchain_state: AppchainState::from_old_version(old_contract.appchain_state),
            staking_histories: old_contract.staking_histories,
            appchain_notification_histories: old_contract.appchain_notification_histories,
            permissionless_actions_status: old_contract.permissionless_actions_status,
            beefy_light_client_state: old_contract.beefy_light_client_state,
            reward_distribution_records: old_contract.reward_distribution_records,
            asset_transfer_is_paused: old_contract.asset_transfer_is_paused,
            user_staking_histories: old_contract.user_staking_histories,
            rewards_withdrawal_is_paused: old_contract.rewards_withdrawal_is_paused,
            appchain_messages: old_contract.appchain_messages,
            appchain_challenges: old_contract.appchain_challenges,
            wrapped_appchain_nfts: old_contract.wrapped_appchain_nfts,
            native_near_token: old_contract.native_near_token,
        };
        //
        //
        new_contract
    }
}

impl AppchainState {
    pub fn from_old_version(old_version: OldAppchainState) -> Self {
        match old_version {
            OldAppchainState::Registered => AppchainState::Registered,
            OldAppchainState::Auditing => AppchainState::Audited,
            OldAppchainState::InQueue => AppchainState::Voting,
            OldAppchainState::Staging => AppchainState::Booting,
            OldAppchainState::Booting => AppchainState::Booting,
            OldAppchainState::Active => AppchainState::Active,
            OldAppchainState::Frozen => AppchainState::Closing,
            OldAppchainState::Broken => AppchainState::Closing,
            OldAppchainState::Dead => AppchainState::Closed,
        }
    }
}

impl ProtocolSettings {
    pub fn from_old_version(old_version: OldProtocolSettings) -> Self {
        ProtocolSettings {
            minimum_validator_deposit: old_version.minimum_validator_deposit,
            minimum_validator_deposit_changing_amount: old_version
                .minimum_validator_deposit_changing_amount,
            maximum_validator_stake_percent: old_version.maximum_validator_stake_percent,
            minimum_delegator_deposit: old_version.minimum_delegator_deposit,
            minimum_delegator_deposit_changing_amount: old_version
                .minimum_delegator_deposit_changing_amount,
            minimum_total_stake_price_for_booting: old_version
                .minimum_total_stake_price_for_booting,
            maximum_market_value_percent_of_near_fungible_tokens: old_version
                .maximum_market_value_percent_of_near_fungible_tokens,
            maximum_market_value_percent_of_wrapped_appchain_token: old_version
                .maximum_market_value_percent_of_wrapped_appchain_token,
            minimum_validator_count: old_version.minimum_validator_count,
            maximum_validator_count: old_version.maximum_validator_count,
            maximum_validators_per_delegator: old_version.maximum_validators_per_delegator,
            unlock_period_of_validator_deposit: old_version.unlock_period_of_validator_deposit,
            unlock_period_of_delegator_deposit: old_version.unlock_period_of_delegator_deposit,
            maximum_era_count_of_unwithdrawn_reward: old_version
                .maximum_era_count_of_unwithdrawn_reward,
            maximum_era_count_of_valid_appchain_message: old_version
                .maximum_era_count_of_valid_appchain_message,
            validator_commission_percent: old_version.validator_commission_percent,
            maximum_allowed_unprofitable_era_count: old_version
                .maximum_allowed_unprofitable_era_count,
            subaccount_for_council_keeper_contract: "octopus-council".to_string(),
        }
    }
}
