mod anchor_viewer;
pub mod appchain_challenge;
pub mod appchain_messages;
mod assets;
pub mod interfaces;
pub mod lookup_array;
mod permissionless_actions;
mod reward_distribution_records;
mod storage_key;
pub mod storage_migration;
pub mod types;
mod user_actions;
mod user_staking_histories;
mod validator_profiles;
mod validator_set;

use core::convert::TryInto;
use getrandom::{register_custom_getrandom, Error};
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedSet};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, log, near_bindgen, serde_json, AccountId, Balance, Gas, PanicOnDefault, Promise,
    PublicKey, Timestamp,
};

pub use appchain_messages::AppchainMessage;
pub use permissionless_actions::AppchainEvent;

use appchain_challenge::AppchainChallenge;
use appchain_messages::AppchainMessages;
use assets::near_fungible_tokens::NearFungibleTokens;
use assets::wrapped_appchain_nfts::WrappedAppchainNFTs;
use beefy_light_client::Hash;
use beefy_light_client::LightClient;
use lookup_array::{IndexedAndClearable, LookupArray};
use reward_distribution_records::RewardDistributionRecords;
use storage_key::StorageKey;
use types::*;
use user_actions::UnbondedStakeReference;
use user_staking_histories::UserStakingHistories;
use validator_profiles::ValidatorProfiles;
use validator_set::next_validator_set::NextValidatorSet;
use validator_set::validator_set_of_era::ValidatorSetOfEra;
use validator_set::ValidatorSetViewer;

register_custom_getrandom!(get_random_in_near);

/// Version of this contract (the same as in Cargo.toml)
const ANCHOR_VERSION: &str = "v2.4.1";
/// Constants for gas.
const T_GAS_FOR_SYNC_STATE_TO_REGISTRY: u64 = 10;
const T_GAS_CAP_FOR_MULTI_TXS_PROCESSING: u64 = 130;
/// The value of decimals value of USD.
const USD_DECIMALS_VALUE: Balance = 1_000_000;
/// The value of decimals value of OCT token.
const OCT_DECIMALS_VALUE: Balance = 1_000_000_000_000_000_000;
/// The seconds of a day.
const SECONDS_OF_A_DAY: u64 = 86400;
/// Multiple of nano seconds for a second.
const NANO_SECONDS_MULTIPLE: u64 = 1_000_000_000;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AppchainAnchor {
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
    protocol_settings: LazyOption<ProtocolSettings>,
    /// The state of the corresponding appchain.
    appchain_state: AppchainState,
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
    #[init]
    pub fn new(appchain_template_type: AppchainTemplateType, oct_token: AccountId) -> Self {
        assert!(!env::state_exists(), "The contract is already initialized.");
        let account_id = env::current_account_id().to_string();
        let parts = account_id.split(".").collect::<Vec<&str>>();
        assert!(
            parts.len() > 2,
            "This contract must be deployed as a sub-account of octopus appchain registry.",
        );
        let (appchain_id, appchain_registry) = account_id.split_once(".").unwrap();
        Self {
            appchain_id: appchain_id.to_string(),
            appchain_template_type,
            appchain_registry: AccountId::try_from(appchain_registry.to_string()).unwrap(),
            owner: env::predecessor_account_id(),
            owner_pk: env::signer_account_pk(),
            oct_token: LazyOption::new(
                StorageKey::OctToken.into_bytes(),
                Some(&OctToken {
                    contract_account: oct_token,
                    price_in_usd: U128::from(0),
                }),
            ),
            wrapped_appchain_token: LazyOption::new(
                StorageKey::WrappedAppchainToken.into_bytes(),
                Some(&WrappedAppchainToken::default()),
            ),
            near_fungible_tokens: LazyOption::new(
                StorageKey::NearFungibleTokens.into_bytes(),
                Some(&NearFungibleTokens::new()),
            ),
            validator_set_histories: LazyOption::new(
                StorageKey::ValidatorSetHistories.into_bytes(),
                Some(&LookupArray::new(StorageKey::ValidatorSetHistoriesMap)),
            ),
            next_validator_set: LazyOption::new(
                StorageKey::NextValidatorSet.into_bytes(),
                Some(&NextValidatorSet::new(u64::MAX)),
            ),
            unwithdrawn_validator_rewards: LookupMap::new(
                StorageKey::UnwithdrawnValidatorRewards.into_bytes(),
            ),
            unwithdrawn_delegator_rewards: LookupMap::new(
                StorageKey::UnwithdrawnDelegatorRewards.into_bytes(),
            ),
            unbonded_stakes: LookupMap::new(StorageKey::UnbondedStakes.into_bytes()),
            validator_profiles: LazyOption::new(
                StorageKey::ValidatorProfiles.into_bytes(),
                Some(&ValidatorProfiles::new()),
            ),
            appchain_settings: LazyOption::new(
                StorageKey::AppchainSettings.into_bytes(),
                Some(&AppchainSettings::default()),
            ),
            anchor_settings: LazyOption::new(
                StorageKey::AnchorSettings.into_bytes(),
                Some(&AnchorSettings::default()),
            ),
            protocol_settings: LazyOption::new(
                StorageKey::ProtocolSettings.into_bytes(),
                Some(&ProtocolSettings::default()),
            ),
            appchain_state: AppchainState::Booting,
            staking_histories: LazyOption::new(
                StorageKey::StakingHistories.into_bytes(),
                Some(&LookupArray::new(StorageKey::StakingHistoriesMap)),
            ),
            appchain_notification_histories: LazyOption::new(
                StorageKey::AppchainNotificationHistories.into_bytes(),
                Some(&LookupArray::new(
                    StorageKey::AppchainNotificationHistoriesMap,
                )),
            ),
            permissionless_actions_status: LazyOption::new(
                StorageKey::PermissionlessActionsStatus.into_bytes(),
                Some(&PermissionlessActionsStatus {
                    switching_era_number: None,
                    distributing_reward_era_number: None,
                    processing_appchain_message_nonce: None,
                    max_nonce_of_staged_appchain_messages: 0,
                    latest_applied_appchain_message_nonce: 0,
                }),
            ),
            beefy_light_client_state: LazyOption::new(
                StorageKey::BeefyLightClientState.into_bytes(),
                None,
            ),
            reward_distribution_records: LazyOption::new(
                StorageKey::RewardDistributionRecords.into_bytes(),
                Some(&RewardDistributionRecords::new()),
            ),
            asset_transfer_is_paused: false,
            user_staking_histories: LazyOption::new(
                StorageKey::UserStakingHistories.into_bytes(),
                Some(&UserStakingHistories::new()),
            ),
            rewards_withdrawal_is_paused: false,
            appchain_messages: LazyOption::new(
                StorageKey::AppchainMessages.into_bytes(),
                Some(&AppchainMessages::new()),
            ),
            appchain_challenges: LazyOption::new(
                StorageKey::AppchainChallenges.into_bytes(),
                Some(&LookupArray::new(StorageKey::AppchainChallengesMap)),
            ),
            wrapped_appchain_nfts: LazyOption::new(
                StorageKey::WrappedAppchainNFTs.into_bytes(),
                Some(&WrappedAppchainNFTs::new()),
            ),
            native_near_token: LazyOption::new(
                StorageKey::NativeNearToken.into_bytes(),
                Some(&NativeNearToken::default()),
            ),
        }
    }
    // Assert that the function is called by the owner.
    fn assert_owner(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.owner,
            "Function can only be called by owner."
        );
    }
    // Assert that the function is called by appchain registry.
    fn assert_registry(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.appchain_registry,
            "Function can only be called by appchain registry contract."
        );
    }
    //
    fn assert_token_price_maintainer(&self) {
        let anchor_settings = self.anchor_settings.get().unwrap();
        let token_price_maintainer_account = anchor_settings
            .token_price_maintainer_account
            .expect("Token price maintainer account is not set.");
        assert_eq!(
            env::predecessor_account_id(),
            token_price_maintainer_account,
            "Only '{}' can call this function.",
            token_price_maintainer_account
        );
    }
    /// Set the price (in USD) of OCT token
    pub fn set_price_of_oct_token(&mut self, price: U128) {
        self.assert_token_price_maintainer();
        let mut oct_token = self.oct_token.get().unwrap();
        oct_token.price_in_usd = price;
        self.oct_token.set(&oct_token);
    }
    ///
    pub fn get_market_value_of_staked_oct_token(&self) -> U128 {
        U128::from(
            self.next_validator_set.get().unwrap().total_stake() / OCT_DECIMALS_VALUE
                * self.oct_token.get().unwrap().price_in_usd.0,
        )
    }
}

impl AppchainAnchor {
    ///
    pub fn internal_append_appchain_notification(
        &mut self,
        appchain_notification: AppchainNotification,
    ) -> AppchainNotificationHistory {
        let mut appchain_notification_histories =
            self.appchain_notification_histories.get().unwrap();
        let appchain_notification_history =
            appchain_notification_histories.append(&mut AppchainNotificationHistory {
                appchain_notification,
                block_height: U64::from(env::block_height()),
                timestamp: U64::from(env::block_timestamp()),
                index: U64::from(0),
            });
        self.appchain_notification_histories
            .set(&appchain_notification_histories);
        appchain_notification_history
    }
    ///
    pub fn sync_state_to_registry(&self) {
        let next_validator_set = self.next_validator_set.get().unwrap();
        // sync state to appchain registry contract
        #[derive(near_sdk::serde::Serialize)]
        #[serde(crate = "near_sdk::serde")]
        struct Args {
            appchain_id: AppchainId,
            appchain_state: AppchainState,
            validator_count: u32,
            total_stake: U128,
        }
        let args = Args {
            appchain_id: self.appchain_id.clone(),
            appchain_state: self.appchain_state.clone(),
            validator_count: next_validator_set.validator_count().try_into().unwrap(),
            total_stake: U128::from(next_validator_set.total_stake()),
        };
        let args = near_sdk::serde_json::to_vec(&args)
            .expect("Failed to serialize the cross contract args using JSON.");
        Promise::new(self.appchain_registry.clone()).function_call(
            "sync_state_of".to_string(),
            args,
            0,
            Gas::ONE_TERA * T_GAS_FOR_SYNC_STATE_TO_REGISTRY,
        );
    }
}

pub fn get_random_in_near(buf: &mut [u8]) -> Result<(), Error> {
    let random = env::random_seed();
    buf.copy_from_slice(&random);
    Ok(())
}

impl IndexedAndClearable for AppchainNotificationHistory {
    //
    fn set_index(&mut self, index: &u64) {
        self.index = U64::from(*index);
    }
    //
    fn clear_extra_storage(&mut self, max_gas: Gas) -> MultiTxsOperationProcessingResult {
        if env::used_gas() > max_gas {
            MultiTxsOperationProcessingResult::NeedMoreGas
        } else {
            MultiTxsOperationProcessingResult::Ok
        }
    }
}

impl IndexedAndClearable for StakingHistory {
    //
    fn set_index(&mut self, index: &u64) {
        self.index = U64::from(*index);
    }
    //
    fn clear_extra_storage(&mut self, max_gas: Gas) -> MultiTxsOperationProcessingResult {
        if env::used_gas() > max_gas {
            MultiTxsOperationProcessingResult::NeedMoreGas
        } else {
            MultiTxsOperationProcessingResult::Ok
        }
    }
}

impl IndexedAndClearable for AppchainChallenge {
    //
    fn set_index(&mut self, _index: &u64) {
        ()
    }
    //
    fn clear_extra_storage(&mut self, max_gas: Gas) -> MultiTxsOperationProcessingResult {
        if env::used_gas() > max_gas {
            MultiTxsOperationProcessingResult::NeedMoreGas
        } else {
            MultiTxsOperationProcessingResult::Ok
        }
    }
}
