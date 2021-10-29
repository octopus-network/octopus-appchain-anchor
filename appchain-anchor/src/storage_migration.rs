use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{env, near_bindgen, AccountId, Balance};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
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
    /// The percent of delegation fee of the a delegator's reward in an era
    pub delegation_fee_percent: u16,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldAppchainAnchor {
    /// The id of corresponding appchain.
    pub appchain_id: AppchainId,
    /// The account id of appchain registry contract.
    pub appchain_registry: AccountId,
    /// The owner account id.
    pub owner: AccountId,
    /// The info of OCT token.
    pub oct_token: LazyOption<OctToken>,
    /// The info of wrapped appchain token in NEAR protocol.
    pub wrapped_appchain_token: LazyOption<WrappedAppchainToken>,
    /// The NEP-141 tokens data.
    pub near_fungible_tokens: LazyOption<NearFungibleTokens>,
    /// The history data of validator set.
    pub validator_set_histories: LazyOption<ValidatorSetHistories>,
    /// The validator set of the next era in appchain.
    /// This validator set is only for checking staking rules.
    pub next_validator_set: LazyOption<ValidatorSet>,
    /// The map of unwithdrawn validator rewards in eras, in unit of wrapped appchain token.
    pub unwithdrawn_validator_rewards: LookupMap<(u64, AccountId), Balance>,
    /// The map of unwithdrawn delegator rewards in eras, in unit of wrapped appchain token.
    pub unwithdrawn_delegator_rewards: LookupMap<(u64, AccountId, AccountId), Balance>,
    /// The map of unbonded stakes in eras.
    pub unbonded_stakes: LookupMap<AccountId, Vec<UnbondedStakeReference>>,
    /// The mapping for validators' accounts, from account id in the appchain to
    /// account id in NEAR protocol.
    pub validator_account_id_mapping: LookupMap<String, AccountId>,
    /// The custom settings for appchain.
    pub appchain_settings: LazyOption<AppchainSettings>,
    /// The anchor settings for appchain.
    pub anchor_settings: LazyOption<AnchorSettings>,
    /// The protocol settings for appchain anchor.
    pub protocol_settings: LazyOption<OldProtocolSettings>,
    /// The state of the corresponding appchain.
    pub appchain_state: AppchainState,
    /// The staking history data happened in this contract.
    pub staking_histories: LazyOption<StakingHistories>,
    /// The anchor events data.
    pub anchor_event_histories: LazyOption<AnchorEventHistories>,
    /// The status of permissionless actions
    pub permissionless_actions_status: LazyOption<PermissionlessActionsStatus>,
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

        // Create the new contract using the data from the old contract.
        let new_contract = AppchainAnchor {
            appchain_id: old_contract.appchain_id,
            appchain_registry: old_contract.appchain_registry,
            owner: old_contract.owner,
            oct_token: old_contract.oct_token,
            wrapped_appchain_token: old_contract.wrapped_appchain_token,
            near_fungible_tokens: old_contract.near_fungible_tokens,
            validator_set_histories: old_contract.validator_set_histories,
            next_validator_set: old_contract.next_validator_set,
            unwithdrawn_validator_rewards: old_contract.unwithdrawn_validator_rewards,
            unwithdrawn_delegator_rewards: old_contract.unwithdrawn_delegator_rewards,
            unbonded_stakes: old_contract.unbonded_stakes,
            validator_account_id_mapping: old_contract.validator_account_id_mapping,
            appchain_settings: old_contract.appchain_settings,
            anchor_settings: old_contract.anchor_settings,
            protocol_settings: LazyOption::new(
                StorageKey::ProtocolSettings.into_bytes(),
                Some(&ProtocolSettings::default()),
            ),
            appchain_state: old_contract.appchain_state,
            staking_histories: old_contract.staking_histories,
            anchor_event_histories: old_contract.anchor_event_histories,
            permissionless_actions_status: old_contract.permissionless_actions_status,
        };

        new_contract
    }
}
