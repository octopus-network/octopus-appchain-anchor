use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{env, near_bindgen, AccountId, Balance, BlockHeight};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum OldAppchainEvent {
    /// The fact that a certain amount of bridge token has been burnt in the appchain.
    NearFungibleTokenBurnt {
        contract_account: String,
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        amount: U128,
    },
    /// The fact that a certain amount of appchain native token has been locked in the appchain.
    NativeTokenLocked {
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        amount: U128,
    },
    /// The fact that the era switch is planed in the appchain.
    EraSwitchPlaned { era_number: u32 },
    /// The fact that the total reward and unprofitable validator list
    /// is concluded in the appchain.
    EraRewardConcluded {
        era_number: u32,
        unprofitable_validator_ids: Vec<String>,
    },
    /// The fact that a certain non-fungible token is locked in the appchain.
    NonFungibleTokenLocked {
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        class_id: String,
        instance_id: String,
        token_metadata: TokenMetadata,
    },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OldAppchainMessage {
    pub appchain_event: OldAppchainEvent,
    // pub block_height: U64,
    // pub timestamp: U64,
    pub nonce: u32,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OldAnchorEventHistory {
    pub anchor_event: AnchorEvent,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub index: U64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OldAppchainNotificationHistory {
    pub appchain_notification: AppchainNotification,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub index: U64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OldStakingHistory {
    pub staking_fact: StakingFact,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub index: U64,
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
    appchain_settings: LazyOption<AppchainSettings>,
    /// The anchor settings for appchain.
    anchor_settings: LazyOption<AnchorSettings>,
    /// The protocol settings for appchain anchor.
    protocol_settings: LazyOption<ProtocolSettings>,
    /// The state of the corresponding appchain.
    appchain_state: AppchainState,
    /// The staking history data happened in this contract.
    staking_histories: LazyOption<LookupArray<StakingHistory>>,
    /// The anchor event history data.
    anchor_event_histories: LazyOption<LookupArray<OldAnchorEventHistory>>,
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
        let mut old_contract: OldAppchainAnchor =
            env::state_read().expect("Old state doesn't exist");
        //
        near_sdk::assert_self();
        //
        old_contract.clear_anchor_events();
        // Create the new contract using the data from the old contract.
        let new_contract = AppchainAnchor {
            appchain_id: old_contract.appchain_id,
            appchain_template_type: AppchainTemplateType::Barnacle,
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
            protocol_settings: old_contract.protocol_settings,
            appchain_state: old_contract.appchain_state,
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
        };
        //
        //
        new_contract
    }
    ///
    pub fn migrate_staking_histories(
        &mut self,
        start_index: U64,
    ) -> MultiTxsOperationProcessingResult {
        near_sdk::assert_self();
        let staking_histories = self.staking_histories.get().unwrap();
        let index_range = staking_histories.index_range();
        for index in start_index.0..index_range.end_index.0 + 1 {
            if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
                return MultiTxsOperationProcessingResult::Error(format!(
                    "Not all records are migrated. Call this function again with start_index '{}'.",
                    index
                ));
            }
            if let Some(old_data) = env::storage_read(&get_storage_key_in_lookup_array(
                &StorageKey::StakingHistoriesMap,
                &index,
            )) {
                if let Ok(old_version) = OldStakingHistory::try_from_slice(&old_data) {
                    env::storage_write(
                        &get_storage_key_in_lookup_array(&StorageKey::StakingHistoriesMap, &index),
                        &StakingHistory::from_old_version(old_version)
                            .try_to_vec()
                            .unwrap(),
                    );
                }
            }
        }
        MultiTxsOperationProcessingResult::Ok
    }
    ///
    pub fn migrate_appchain_notification_histories(
        &mut self,
        start_index: U64,
    ) -> MultiTxsOperationProcessingResult {
        near_sdk::assert_self();
        let appchain_notification_histories = self.appchain_notification_histories.get().unwrap();
        let index_range = appchain_notification_histories.index_range();
        for index in start_index.0..index_range.end_index.0 + 1 {
            if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
                return MultiTxsOperationProcessingResult::Error(format!(
                    "Not all records are migrated. Call this function again with start_index '{}'.",
                    index
                ));
            }
            if let Some(old_data) = env::storage_read(&get_storage_key_in_lookup_array(
                &StorageKey::AppchainNotificationHistoriesMap,
                &index,
            )) {
                if let Ok(old_version) = OldAppchainNotificationHistory::try_from_slice(&old_data) {
                    env::storage_write(
                        &get_storage_key_in_lookup_array(
                            &StorageKey::AppchainNotificationHistoriesMap,
                            &index,
                        ),
                        &AppchainNotificationHistory::from_old_version(old_version)
                            .try_to_vec()
                            .unwrap(),
                    );
                }
            }
        }
        MultiTxsOperationProcessingResult::Ok
    }
    ///
    pub fn migrate_appchain_messages(
        &mut self,
        start_nonce: u32,
    ) -> MultiTxsOperationProcessingResult {
        near_sdk::assert_self();
        let appchain_messages = self.appchain_messages.get().unwrap();
        for nonce in start_nonce..appchain_messages.max_nonce() + 1 {
            if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
                return MultiTxsOperationProcessingResult::Error(format!(
                    "Not all records are migrated. Call this function again with start_nonce '{}'.",
                    nonce
                ));
            }
            if let Some(old_data) = env::storage_read(&get_storage_key_in_lookup_array(
                &StorageKey::AppchainMessageMap,
                &nonce,
            )) {
                if let Ok(old_version) = OldAppchainMessage::try_from_slice(&old_data) {
                    env::storage_write(
                        &get_storage_key_in_lookup_array(&StorageKey::AppchainMessageMap, &nonce),
                        &AppchainMessage::from_old_version(old_version)
                            .try_to_vec()
                            .unwrap(),
                    );
                }
            }
        }
        MultiTxsOperationProcessingResult::Ok
    }
}

fn get_storage_key_in_lookup_array<T: BorshSerialize>(prefix: &StorageKey, index: &T) -> Vec<u8> {
    [prefix.into_bytes(), index.try_to_vec().unwrap()].concat()
}

impl OldAppchainAnchor {
    ///
    pub fn clear_anchor_events(&mut self) {
        let mut anchor_event_histories = self.anchor_event_histories.get().unwrap();
        let result = anchor_event_histories.clear();
        self.anchor_event_histories.set(&anchor_event_histories);
        if result.is_ok() {
            self.anchor_event_histories.remove();
        } else {
            panic!("Should clear old anchor events first.");
        }
    }
}

impl IndexedAndClearable for OldAnchorEventHistory {
    //
    fn set_index(&mut self, index: &u64) {
        self.index = U64::from(*index);
    }
    //
    fn clear_extra_storage(&mut self) -> MultiTxsOperationProcessingResult {
        if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
            MultiTxsOperationProcessingResult::NeedMoreGas
        } else {
            MultiTxsOperationProcessingResult::Ok
        }
    }
}

impl IndexedAndClearable for OldAppchainNotificationHistory {
    //
    fn set_index(&mut self, index: &u64) {
        self.index = U64::from(*index);
    }
    //
    fn clear_extra_storage(&mut self) -> MultiTxsOperationProcessingResult {
        if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
            MultiTxsOperationProcessingResult::NeedMoreGas
        } else {
            MultiTxsOperationProcessingResult::Ok
        }
    }
}

impl IndexedAndClearable for OldStakingHistory {
    //
    fn set_index(&mut self, index: &u64) {
        self.index = U64::from(*index);
    }
    //
    fn clear_extra_storage(&mut self) -> MultiTxsOperationProcessingResult {
        if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
            MultiTxsOperationProcessingResult::NeedMoreGas
        } else {
            MultiTxsOperationProcessingResult::Ok
        }
    }
}

impl StakingHistory {
    //
    pub fn from_old_version(old_version: OldStakingHistory) -> Self {
        Self {
            staking_fact: old_version.staking_fact,
            block_height: U64::from(old_version.block_height),
            timestamp: U64::from(old_version.timestamp),
            index: old_version.index,
        }
    }
}

impl AppchainNotificationHistory {
    //
    pub fn from_old_version(old_version: OldAppchainNotificationHistory) -> Self {
        Self {
            appchain_notification: old_version.appchain_notification,
            block_height: U64::from(old_version.block_height),
            timestamp: U64::from(old_version.timestamp),
            index: old_version.index,
        }
    }
}

impl AppchainEvent {
    //
    pub fn from_old_version(old_version: OldAppchainEvent) -> Self {
        match old_version {
            OldAppchainEvent::NearFungibleTokenBurnt {
                contract_account,
                owner_id_in_appchain,
                receiver_id_in_near,
                amount,
            } => AppchainEvent::NearFungibleTokenBurnt {
                contract_account,
                owner_id_in_appchain,
                receiver_id_in_near,
                amount,
            },
            OldAppchainEvent::NativeTokenLocked {
                owner_id_in_appchain,
                receiver_id_in_near,
                amount,
            } => AppchainEvent::NativeTokenLocked {
                owner_id_in_appchain,
                receiver_id_in_near,
                amount,
            },
            OldAppchainEvent::EraSwitchPlaned { era_number } => {
                AppchainEvent::EraSwitchPlaned { era_number }
            }
            OldAppchainEvent::EraRewardConcluded {
                era_number,
                unprofitable_validator_ids,
            } => AppchainEvent::EraRewardConcluded {
                era_number,
                unprofitable_validator_ids,
                offenders: Vec::new(),
            },
            OldAppchainEvent::NonFungibleTokenLocked {
                owner_id_in_appchain,
                receiver_id_in_near,
                class_id,
                instance_id,
                token_metadata,
            } => AppchainEvent::NonFungibleTokenLocked {
                owner_id_in_appchain,
                receiver_id_in_near,
                class_id,
                instance_id,
                token_metadata,
            },
        }
    }
}

impl AppchainMessage {
    //
    pub fn from_old_version(old_version: OldAppchainMessage) -> Self {
        Self {
            appchain_event: AppchainEvent::from_old_version(old_version.appchain_event),
            nonce: old_version.nonce,
        }
    }
}
