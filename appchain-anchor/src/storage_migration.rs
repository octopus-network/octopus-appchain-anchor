use crate::validator_set::ValidatorSet;
use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{env, near_bindgen, AccountId, Balance};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldProtocolSettings {
    /// A validator has to deposit a certain amount of OCT token to this contract for
    /// being validator of the appchain.
    pub minimum_validator_deposit: U128,
    /// The minimum deposit amount for a delegator to delegate his voting weight to
    /// a certain validator.
    pub minimum_delegator_deposit: U128,
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
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldAppchainAnchor {
    /// The id of corresponding appchain.
    appchain_id: AppchainId,
    /// The account id of appchain registry contract.
    appchain_registry: AccountId,
    /// The owner account id.
    owner: AccountId,
    /// The info of OCT token.
    oct_token: LazyOption<OctToken>,
    /// The info of wrapped appchain token in NEAR protocol.
    wrapped_appchain_token: LazyOption<WrappedAppchainToken>,
    /// The NEP-141 tokens data.
    near_fungible_tokens: LazyOption<NearFungibleTokens>,
    /// The history data of validator set.
    validator_set_histories: LazyOption<IndexedHistories<ValidatorSetOfEra>>,
    /// The validator set of the next era in appchain.
    /// This validator set is only for checking staking rules.
    next_validator_set: LazyOption<ValidatorSet>,
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
    appchain_state: AppchainState,
    /// The staking history data happened in this contract.
    staking_histories: LazyOption<IndexedHistories<StakingHistory>>,
    /// The anchor event history data.
    anchor_event_histories: LazyOption<IndexedHistories<AnchorEventHistory>>,
    /// The appchain notification history data.
    appchain_notification_histories: LazyOption<IndexedHistories<AppchainNotificationHistory>>,
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
}

#[near_bindgen]
impl AppchainAnchor {
    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let old_contract: OldAppchainAnchor = env::state_read().expect("Old state doesn't exist");
        // Verify that the migration can only be done by the owner.
        // This is not necessary, if the upgrade is done internally.
        assert_eq!(
            &env::predecessor_account_id(),
            &old_contract.owner,
            "Can only be called by the owner"
        );
        //
        let old_next_validator_set = old_contract.next_validator_set.get().unwrap();
        let old_protocol_settings = old_contract.protocol_settings.get().unwrap();
        // Create the new contract using the data from the old contract.
        let new_contract = AppchainAnchor {
            appchain_id: old_contract.appchain_id,
            appchain_registry: old_contract.appchain_registry,
            owner: old_contract.owner,
            oct_token: old_contract.oct_token,
            wrapped_appchain_token: old_contract.wrapped_appchain_token,
            near_fungible_tokens: old_contract.near_fungible_tokens,
            validator_set_histories: old_contract.validator_set_histories,
            next_validator_set: LazyOption::new(
                StorageKey::NextValidatorSet.into_bytes(),
                Some(&NextValidatorSet::from_validator_set(
                    old_next_validator_set,
                )),
            ),
            unwithdrawn_validator_rewards: old_contract.unwithdrawn_validator_rewards,
            unwithdrawn_delegator_rewards: old_contract.unwithdrawn_delegator_rewards,
            unbonded_stakes: old_contract.unbonded_stakes,
            validator_profiles: old_contract.validator_profiles,
            appchain_settings: old_contract.appchain_settings,
            anchor_settings: old_contract.anchor_settings,
            protocol_settings: LazyOption::new(
                StorageKey::ProtocolSettings.into_bytes(),
                Some(&ProtocolSettings::from_old_version(old_protocol_settings)),
            ),
            appchain_state: old_contract.appchain_state,
            staking_histories: old_contract.staking_histories,
            anchor_event_histories: old_contract.anchor_event_histories,
            appchain_notification_histories: old_contract.appchain_notification_histories,
            permissionless_actions_status: old_contract.permissionless_actions_status,
            beefy_light_client_state: old_contract.beefy_light_client_state,
            reward_distribution_records: old_contract.reward_distribution_records,
            asset_transfer_is_paused: old_contract.asset_transfer_is_paused,
            user_staking_histories: old_contract.user_staking_histories,
            rewards_withdrawal_is_paused: old_contract.rewards_withdrawal_is_paused,
            appchain_message_processing_results: LazyOption::new(
                StorageKey::AppchainMessageProcessingResults.into_bytes(),
                Some(&AppchainMessageProcessingResults::new()),
            ),
        };
        //
        //
        new_contract
    }
}

impl ProtocolSettings {
    pub fn from_old_version(old_version: OldProtocolSettings) -> ProtocolSettings {
        ProtocolSettings {
            minimum_validator_deposit: old_version.minimum_validator_deposit,
            minimum_validator_deposit_changing_amount: U128::from(1000 * OCT_DECIMALS_VALUE),
            maximum_validator_stake_percent: 25,
            minimum_delegator_deposit: old_version.minimum_delegator_deposit,
            minimum_delegator_deposit_changing_amount: U128::from(100 * OCT_DECIMALS_VALUE),
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
            maximum_allowed_unprofitable_era_count: 3,
        }
    }
}
