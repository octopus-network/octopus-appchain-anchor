use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{env, near_bindgen, AccountId, Balance};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldStakingHistories {
    /// The staking history data happened in this contract.
    histories: LookupMap<u64, StakingHistory>,
    /// The start index of valid staking history.
    start_index: u64,
    /// The end index of valid staking history.
    end_index: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldAnchorEventHistories {
    /// The anchor event data map.
    histories: LookupMap<u64, AnchorEventHistory>,
    /// The start index of valid anchor event.
    start_index: u64,
    /// The end index of valid anchor event.
    end_index: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldAppchainNotificationHistories {
    /// The anchor event data map.
    histories: LookupMap<u64, AppchainNotificationHistory>,
    /// The start index of valid anchor event.
    start_index: u64,
    /// The end index of valid anchor event.
    end_index: u64,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldValidatorSetHistories {
    /// The history version of validator set, mapped by era number in appchain.
    histories: LookupMap<u64, ValidatorSetOfEra>,
    /// The start index of valid validator set.
    start_index: u64,
    /// The end index of valid validator set.
    end_index: u64,
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
    validator_set_histories: LazyOption<OldValidatorSetHistories>,
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
    protocol_settings: LazyOption<ProtocolSettings>,
    /// The state of the corresponding appchain.
    appchain_state: AppchainState,
    /// The staking history data happened in this contract.
    staking_histories: LazyOption<OldStakingHistories>,
    /// The anchor event history data.
    anchor_event_histories: LazyOption<OldAnchorEventHistories>,
    /// The appchain notification history data.
    appchain_notification_histories: LazyOption<OldAppchainNotificationHistories>,
    /// The status of permissionless actions.
    permissionless_actions_status: LazyOption<PermissionlessActionsStatus>,
    /// The state of beefy light client
    beefy_light_client_state: LazyOption<LightClient>,
    /// The reward distribution records data
    reward_distribution_records: LazyOption<RewardDistributionRecords>,
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
        let old_validator_set_histories = old_contract.validator_set_histories.get().unwrap();
        let old_staking_histories = old_contract.staking_histories.get().unwrap();
        let old_anchor_event_histories = old_contract.anchor_event_histories.get().unwrap();
        let old_appchain_notification_histories =
            old_contract.appchain_notification_histories.get().unwrap();
        // Create the new contract using the data from the old contract.
        let new_contract = AppchainAnchor {
            appchain_id: old_contract.appchain_id,
            appchain_registry: old_contract.appchain_registry,
            owner: old_contract.owner,
            oct_token: old_contract.oct_token,
            wrapped_appchain_token: old_contract.wrapped_appchain_token,
            near_fungible_tokens: old_contract.near_fungible_tokens,
            validator_set_histories: LazyOption::new(
                StorageKey::ValidatorSetHistories.into_bytes(),
                Some(&IndexedHistories::<ValidatorSetOfEra>::migrate_from(
                    StorageKey::ValidatorSetHistoriesMap,
                    old_validator_set_histories.start_index,
                    old_validator_set_histories.end_index,
                )),
            ),
            next_validator_set: old_contract.next_validator_set,
            unwithdrawn_validator_rewards: old_contract.unwithdrawn_validator_rewards,
            unwithdrawn_delegator_rewards: old_contract.unwithdrawn_delegator_rewards,
            unbonded_stakes: old_contract.unbonded_stakes,
            validator_profiles: old_contract.validator_profiles,
            appchain_settings: old_contract.appchain_settings,
            anchor_settings: old_contract.anchor_settings,
            protocol_settings: old_contract.protocol_settings,
            appchain_state: old_contract.appchain_state,
            staking_histories: LazyOption::new(
                StorageKey::StakingHistories.into_bytes(),
                Some(&IndexedHistories::<StakingHistory>::migrate_from(
                    StorageKey::StakingHistoriesMap,
                    old_staking_histories.start_index,
                    old_staking_histories.end_index,
                )),
            ),
            anchor_event_histories: LazyOption::new(
                StorageKey::AnchorEventHistories.into_bytes(),
                Some(&IndexedHistories::<AnchorEventHistory>::migrate_from(
                    StorageKey::AnchorEventHistoriesMap,
                    old_anchor_event_histories.start_index,
                    old_anchor_event_histories.end_index,
                )),
            ),
            appchain_notification_histories: LazyOption::new(
                StorageKey::AppchainNotificationHistories.into_bytes(),
                Some(
                    &IndexedHistories::<AppchainNotificationHistory>::migrate_from(
                        StorageKey::AppchainNotificationHistoriesMap,
                        old_appchain_notification_histories.start_index,
                        old_appchain_notification_histories.end_index,
                    ),
                ),
            ),
            permissionless_actions_status: old_contract.permissionless_actions_status,
            beefy_light_client_state: old_contract.beefy_light_client_state,
            reward_distribution_records: old_contract.reward_distribution_records,
        };
        //
        new_contract
    }
}
