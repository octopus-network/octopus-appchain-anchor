use crate::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{env, near_bindgen, AccountId, Balance};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldAnchorSettings {
    pub token_price_maintainer_account: AccountId,
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
    /// The validators' profiles data.
    pub validator_profiles: LazyOption<ValidatorProfiles>,
    /// The custom settings for appchain.
    pub appchain_settings: LazyOption<AppchainSettings>,
    /// The anchor settings for appchain.
    pub anchor_settings: LazyOption<OldAnchorSettings>,
    /// The protocol settings for appchain anchor.
    pub protocol_settings: LazyOption<ProtocolSettings>,
    /// The state of the corresponding appchain.
    pub appchain_state: AppchainState,
    /// The staking history data happened in this contract.
    pub staking_histories: LazyOption<StakingHistories>,
    /// The anchor events data.
    pub anchor_event_histories: LazyOption<AnchorEventHistories>,
    /// The appchain notification history data.
    pub appchain_notification_histories: LazyOption<AppchainNotificationHistories>,
    /// The status of permissionless actions
    pub permissionless_actions_status: LazyOption<PermissionlessActionsStatus>,
    /// The state of beefy light client
    pub beefy_light_client_state: LazyOption<LightClient>,
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
        let old_anchor_settings = old_contract.anchor_settings.get().unwrap();
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
            validator_profiles: old_contract.validator_profiles,
            appchain_settings: old_contract.appchain_settings,
            anchor_settings: LazyOption::new(
                StorageKey::AnchorSettings.into_bytes(),
                Some(&AnchorSettings {
                    token_price_maintainer_account: old_anchor_settings
                        .token_price_maintainer_account,
                    relayer_account: AccountId::new(),
                    beefy_light_client_witness_mode: false,
                }),
            ),
            protocol_settings: old_contract.protocol_settings,
            appchain_state: old_contract.appchain_state,
            staking_histories: old_contract.staking_histories,
            anchor_event_histories: old_contract.anchor_event_histories,
            appchain_notification_histories: old_contract.appchain_notification_histories,
            permissionless_actions_status: old_contract.permissionless_actions_status,
            beefy_light_client_state: old_contract.beefy_light_client_state,
        };
        //
        new_contract
    }
}
