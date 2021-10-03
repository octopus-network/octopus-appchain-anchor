mod anchor_viewer;
mod appchain_lifecycle;
mod near_fungible_token;
mod permissionless_actions;
mod settings_manager;
mod staking;
mod storage_key;
mod token_bridging;
pub mod types;
mod validator_set;
mod wrapped_appchain_token;

use anchor_viewer::AnchorEvents;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedSet, Vector};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_self, env, ext_contract, log, near_bindgen, AccountId, Balance, Duration, Gas, Promise,
    PromiseOrValue, PromiseResult, PublicKey, Timestamp,
};
use staking::StakingHistories;
use token_bridging::TokenBridgingHistories;
use types::*;
use validator_set::{ValidatorSet, ValidatorSetOfEra};

use crate::storage_key::StorageKey;

/// The value of decimals value of OCT token
const OCT_DECIMALS_VALUE: Balance = 1_000_000_000_000_000_000;
/// Multiple of nano seconds for a second
const NANO_SECONDS_MULTIPLE: u64 = 1_000_000_000;
/// Gas cap for function `complete_switching_era`
const GAS_CAP_FOR_COMPLETE_SWITCHING_ERA: Gas = 180_000_000_000_000;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainAnchor {
    /// The id of corresponding appchain.
    pub appchain_id: AppchainId,
    /// The account id of appchain registry contract.
    pub appchain_registry: AccountId,
    /// The info of OCT token.
    pub oct_token: LazyOption<OctToken>,
    /// The info of wrapped appchain token in NEAR protocol.
    pub wrapped_appchain_token: LazyOption<WrappedAppchainToken>,
    /// The set of symbols of NEP-141 tokens.
    pub near_fungible_token_symbols: UnorderedSet<String>,
    /// The NEP-141 tokens data, mapped by the symbol of the token.
    pub near_fungible_tokens: LookupMap<String, NearFungibleToken>,
    /// The history version of validator set, mapped by era number in appchain.
    pub validator_set_histories: LookupMap<u64, ValidatorSetOfEra>,
    /// The validator set of the next era in appchain.
    /// This validator set is only for checking staking rules.
    pub next_validator_set: LazyOption<ValidatorSet>,
    /// The validator list of eras
    pub validator_list_of_eras: LookupMap<u64, Vector<AppchainValidator>>,
    /// The map of unwithdrawed validator rewards in eras, in unit of wrapped appchain token.
    pub unwithdrawed_validator_rewards: LookupMap<(u64, AccountId), Balance>,
    /// The map of unwithdrawed delegator rewards in eras, in unit of wrapped appchain token.
    pub unwithdrawed_delegator_rewards: LookupMap<(u64, AccountId, AccountId), Balance>,
    /// The map of unbonded stakes in eras.
    pub unbonded_stakes: LookupMap<AccountId, Vec<UnbondedStake>>,
    /// The mapping for validators' accounts, from account id in the appchain to
    /// account id in NEAR protocol.
    pub validator_account_id_mapping: LookupMap<AccountIdInAppchain, AccountId>,
    /// The custom settings for appchain.
    pub appchain_settings: LazyOption<AppchainSettings>,
    /// The anchor settings for appchain.
    pub anchor_settings: LazyOption<AnchorSettings>,
    /// The protocol settings for appchain anchor.
    pub protocol_settings: LazyOption<ProtocolSettings>,
    /// The state of the corresponding appchain.
    pub appchain_state: AppchainState,
    /// The staking history data happened in this contract.
    pub staking_histories: LazyOption<StakingHistories>,
    /// The token bridging histories data happened in this contract.
    pub token_bridging_histories: LazyOption<TokenBridgingHistories>,
    /// The anchor events data.
    pub anchor_events: LazyOption<AnchorEvents>,
    /// The status of permissionless actions
    pub permissionless_actions_status: LazyOption<PermissionlessActionsStatus>,
}

impl Default for AppchainAnchor {
    fn default() -> Self {
        env::panic(b"The contract needs be initialized before use.")
    }
}

#[near_bindgen]
impl AppchainAnchor {
    #[init]
    pub fn new(
        appchain_id: AppchainId,
        appchain_registry: AccountId,
        oct_token: AccountId,
    ) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized.");
        Self {
            appchain_id,
            appchain_registry,
            oct_token: LazyOption::new(
                StorageKey::OctToken.into_bytes(),
                Some(&OctToken {
                    contract_account: oct_token,
                    price_in_usd: U64::from(0),
                    total_stake: 0,
                }),
            ),
            wrapped_appchain_token: LazyOption::new(
                StorageKey::WrappedAppchainToken.into_bytes(),
                Some(&WrappedAppchainToken::default()),
            ),
            near_fungible_token_symbols: UnorderedSet::new(
                StorageKey::NearFungibleTokenSymbols.into_bytes(),
            ),
            near_fungible_tokens: LookupMap::new(StorageKey::NearFungibleTokens.into_bytes()),
            validator_set_histories: LookupMap::new(StorageKey::ValidatorSetHistories.into_bytes()),
            next_validator_set: LazyOption::new(
                StorageKey::NextValidatorSet.into_bytes(),
                Some(&ValidatorSet::new(u64::MAX)),
            ),
            validator_list_of_eras: LookupMap::new(StorageKey::ValidatorListOfEras.into_bytes()),
            unwithdrawed_validator_rewards: LookupMap::new(
                StorageKey::UnwithdrawedValidatorRewards.into_bytes(),
            ),
            unwithdrawed_delegator_rewards: LookupMap::new(
                StorageKey::UnwithdrawedDelegatorRewards.into_bytes(),
            ),
            unbonded_stakes: LookupMap::new(StorageKey::UnbondedStakes.into_bytes()),
            validator_account_id_mapping: LookupMap::new(
                StorageKey::LookupMapOfValidatorIdsInAppchain.into_bytes(),
            ),
            appchain_settings: LazyOption::new(
                StorageKey::AppchainSettings.into_bytes(),
                Some(&AppchainSettings {
                    chain_spec: String::new(),
                    raw_chain_spec: String::new(),
                    boot_nodes: String::new(),
                    rpc_endpoint: String::new(),
                    era_reward: 0,
                }),
            ),
            anchor_settings: LazyOption::new(
                StorageKey::AnchorSettings.into_bytes(),
                Some(&AnchorSettings {
                    token_price_maintainer_account: AccountId::new(),
                }),
            ),
            protocol_settings: LazyOption::new(
                StorageKey::ProtocolSettings.into_bytes(),
                Some(&ProtocolSettings::default()),
            ),
            appchain_state: AppchainState::Staging,
            staking_histories: LazyOption::new(
                StorageKey::StakingHistories.into_bytes(),
                Some(&StakingHistories {
                    histories: LookupMap::new(StorageKey::StakingHistoriesMap.into_bytes()),
                    start_index: 0,
                    end_index: 0,
                }),
            ),
            token_bridging_histories: LazyOption::new(
                StorageKey::TokenBridgingHistories.into_bytes(),
                Some(&TokenBridgingHistories {
                    histories: LookupMap::new(StorageKey::TokenBridgingHistoriesMap.into_bytes()),
                    start_index: 0,
                    end_index: 0,
                }),
            ),
            anchor_events: LazyOption::new(
                StorageKey::AnchorEvents.into_bytes(),
                Some(&AnchorEvents {
                    events: LookupMap::new(StorageKey::AnchorEventsMap.into_bytes()),
                    start_index: 0,
                    end_index: 0,
                }),
            ),
            permissionless_actions_status: LazyOption::new(
                StorageKey::PermissionlessActionsStatus.into_bytes(),
                Some(&PermissionlessActionsStatus {
                    switching_era_number: Option::None,
                    distributing_reward_era_number: Option::None,
                }),
            ),
        }
    }
}
