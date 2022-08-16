use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{env, near_bindgen, AccountId, Balance};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OldAppchainSettings {
    pub rpc_endpoint: String,
    pub subql_endpoint: String,
    pub era_reward: U128,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldAppchainAnchor {
    /// The id of corresponding appchain.
    appchain_id: AppchainId,
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
    appchain_settings: LazyOption<OldAppchainSettings>,
    /// The anchor settings for appchain.
    anchor_settings: LazyOption<AnchorSettings>,
    /// The protocol settings for appchain anchor.
    protocol_settings: LazyOption<ProtocolSettings>,
    /// The state of the corresponding appchain.
    appchain_state: AppchainState,
    /// The staking history data happened in this contract.
    staking_histories: LazyOption<LookupArray<StakingHistory>>,
    /// The anchor event history data.
    anchor_event_histories: LazyOption<LookupArray<AnchorEventHistory>>,
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
            &env::current_account_id(),
            "Can only be called by self"
        );
        //
        let old_appchain_settings = old_contract.appchain_settings.get().unwrap();
        // Create the new contract using the data from the old contract.
        let new_contract = AppchainAnchor {
            appchain_id: old_contract.appchain_id,
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
            appchain_settings: LazyOption::new(
                StorageKey::AppchainSettings.into_bytes(),
                Some(&AppchainSettings::from_old_version(old_appchain_settings)),
            ),
            anchor_settings: old_contract.anchor_settings,
            protocol_settings: old_contract.protocol_settings,
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
            appchain_messages: old_contract.appchain_messages,
            appchain_challenges: old_contract.appchain_challenges,
            wrapped_appchain_nfts: old_contract.wrapped_appchain_nfts,
        };
        //
        //
        new_contract
    }
}

impl AppchainSettings {
    //
    pub fn from_old_version(old_version: OldAppchainSettings) -> Self {
        Self {
            rpc_endpoint: old_version.rpc_endpoint,
            subql_endpoint: old_version.subql_endpoint,
            era_reward: old_version.era_reward,
            bonus_for_new_validator: U128::from(0),
        }
    }
}
