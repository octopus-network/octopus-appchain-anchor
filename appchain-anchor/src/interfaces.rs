use borsh::maybestd::collections::HashMap;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;

use crate::*;

pub trait AnchorViewer {
    /// Get anchor settings detail.
    fn get_anchor_settings(&self) -> AnchorSettings;
    /// Get appchain settings detail.
    fn get_appchain_settings(&self) -> AppchainSettings;
    /// Get protocol settings detail.
    fn get_protocol_settings(&self) -> ProtocolSettings;
    /// Get info of OCT token.
    fn get_oct_token(&self) -> OctToken;
    /// Get info of wrapped appchain token.
    fn get_wrapped_appchain_token(&self) -> WrappedAppchainToken;
    /// Get info of near fungible tokens which has registered in this contract.
    fn get_near_fungible_tokens(&self) -> Vec<NearFungibleToken>;
    /// Get state of corresponding appchain.
    fn get_appchain_state(&self) -> AppchainState;
    /// Get current status of anchor.
    fn get_anchor_status(&self) -> AnchorStatus;
    /// Get validator set history info.
    fn get_validator_set_info_of(&self, era_number: U64) -> Option<ValidatorSetInfo>;
    /// Get processing status of validator set of era.
    fn get_processing_status_of(&self, era_number: U64) -> Option<ValidatorSetProcessingStatus>;
    /// Get the index range of staking histories stored in anchor.
    fn get_index_range_of_staking_history(&self) -> IndexRange;
    /// Get staking history by start index and quantity.
    /// If the param `quantity` is omitted, up to 50 events will be returned.
    fn get_staking_histories(&self, start_index: U64, quantity: Option<U64>)
        -> Vec<StakingHistory>;
    /// Get staking history by index.
    /// If the param `index `is omitted, the latest history will be returned.
    /// If the paran `index` is smaller than the start index, or bigger than the end index
    /// stored in anchor, or there is no history in anchor yet, `Option::None` will be returned.
    fn get_staking_history(&self, index: Option<U64>) -> Option<StakingHistory>;
    /// Get the index range of anchor events stored in anchor.
    fn get_index_range_of_anchor_event_history(&self) -> IndexRange;
    /// Get anchor event by index.
    /// If the param `index `is omitted, the latest event will be returned.
    /// If the paran `index` is smaller than the start index, or bigger than the end index
    /// stored in anchor, or there is no event in anchor yet, `Option::None` will be returned.
    fn get_anchor_event_history(&self, index: Option<U64>) -> Option<AnchorEventHistory>;
    /// Get anchor event by start index and quantity.
    /// If the param `quantity` is omitted, up to 50 events will be returned.
    fn get_anchor_event_histories(
        &self,
        start_index: U64,
        quantity: Option<U64>,
    ) -> Vec<AnchorEventHistory>;
    /// Get the index range of appchain notification histories stored in anchor.
    fn get_index_range_of_appchain_notification_history(&self) -> IndexRange;
    /// Get appchain notification by index.
    /// If the param `index `is omitted, the latest notification will be returned.
    /// If the paran `index` is smaller than the start index, or bigger than the end index
    /// stored in anchor, or there is no event in anchor yet, `Option::None` will be returned.
    fn get_appchain_notification_history(
        &self,
        index: Option<U64>,
    ) -> Option<AppchainNotificationHistory>;
    /// Get appchain notification history by start index and quantity.
    /// If the param `quantity` is omitted, up to 50 events will be returned.
    fn get_appchain_notification_histories(
        &self,
        start_index: U64,
        quantity: Option<U64>,
    ) -> Vec<AppchainNotificationHistory>;
    /// Get the validator list of a certain era.
    fn get_validator_list_of(&self, era_number: Option<U64>) -> Vec<AppchainValidator>;
    /// Get the delegators of a validator of a certain era.
    /// If the param `era_number` is omitted, the latest validator set will be used.
    fn get_delegators_of_validator_in_era(
        &self,
        era_number: Option<U64>,
        validator_id: AccountId,
    ) -> Vec<AppchainDelegator>;
    /// Get unbonded stakes of an account.
    fn get_unbonded_stakes_of(&self, account_id: AccountId) -> Vec<UnbondedStake>;
    /// Get validator rewards of a certain era range.
    fn get_validator_rewards_of(
        &self,
        start_era: U64,
        end_era: U64,
        validator_id: AccountId,
    ) -> Vec<RewardHistory>;
    /// Get validator rewards of a certain era range.
    fn get_delegator_rewards_of(
        &self,
        start_era: U64,
        end_era: U64,
        delegator_id: AccountId,
        validator_id: AccountId,
    ) -> Vec<RewardHistory>;
    /// Get current storage balance needed by this contract account.
    fn get_storage_balance(&self) -> U128;
    /// Get deposit of a certain validator in a certain era.
    fn get_validator_deposit_of(&self, validator_id: AccountId, era_number: Option<U64>) -> U128;
    /// Get deposit of a certain delegator in a certain era.
    fn get_delegator_deposit_of(
        &self,
        delegator_id: AccountId,
        validator_id: AccountId,
        era_number: Option<U64>,
    ) -> U128;
    /// Get delegation list of a certain delegator in a certain era.
    fn get_delegations_of(
        &self,
        delegator_id: AccountId,
        era_number: Option<U64>,
    ) -> Vec<AppchainDelegator>;
    /// Get profile of a certain validator.
    fn get_validator_profile(&self, validator_id: AccountId) -> Option<ValidatorProfile>;
    /// Get profile of a certain validator.
    fn get_validator_profiles(&self) -> Vec<ValidatorProfile>;
    /// Get validator profile by his/her account id in appchain.
    fn get_validator_profile_by_id_in_appchain(
        &self,
        validator_id_in_appchain: String,
    ) -> Option<ValidatorProfile>;
    /// Get the latest commitment data of appchain state.
    fn get_latest_commitment_of_appchain(&self) -> Option<AppchainCommitment>;
    /// Get status of the beefy light client.
    fn get_beefy_light_client_status(&self) -> BeefyLightClientStatus;
    /// Get staking histories related to the given account id.
    fn get_user_staking_histories_of(&self, account_id: AccountId) -> Vec<UserStakingHistory>;
}

pub trait AppchainLifecycleManager {
    /// Verify and change the state of corresponding appchain to `booting`.
    fn go_booting(&mut self);
    /// Verify and change the state of corresponding appchain to `active`.
    fn go_live(&mut self);
    /// Initialize the beefy light client
    fn initialize_beefy_light_client(&mut self, initial_public_keys: Vec<String>);
}

pub trait NearFungibleTokenManager {
    ///
    fn register_near_fungible_token(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        contract_account: AccountId,
        price: U128,
    );
    ///
    fn change_near_fungible_token_metadata(
        &mut self,
        symbol: String,
        name: String,
        decimals: u8,
        contract_account: AccountId,
    );
    ///
    fn set_price_of_near_fungible_token(&mut self, symbol: String, price: U128);
    ///
    fn open_bridging_of_near_fungible_token(&mut self, symbol: String);
    ///
    fn close_bridging_of_near_fungible_token(&mut self, symbol: String);
}

pub trait OwnerActions {
    ///
    fn remove_validator_set_before(&mut self, era_number: U64);
    ///
    fn remove_staking_history_before(&mut self, index: U64);
    ///
    fn remove_anchor_event_history_before(&mut self, index: U64);
    ///
    fn remove_appchain_notification_history_before(&mut self, index: U64);
}

pub trait PermissionlessActions {
    ///
    fn start_updating_state_of_beefy_light_client(
        &mut self,
        signed_commitment: Vec<u8>,
        validator_proofs: Vec<ValidatorMerkleProof>,
        mmr_leaf: Vec<u8>,
        mmr_proof: Vec<u8>,
    );
    ///
    fn try_complete_updating_state_of_beefy_light_client(
        &mut self,
    ) -> MultiTxsOperationProcessingResult;
    ///
    fn verify_and_apply_appchain_messages(
        &mut self,
        encoded_messages: Vec<u8>,
        header: Vec<u8>,
        mmr_leaf: Vec<u8>,
        mmr_proof: Vec<u8>,
    ) -> Vec<AppchainMessageProcessingResult>;
    ///
    fn try_complete_switching_era(&mut self) -> MultiTxsOperationProcessingResult;
    ///
    fn try_complete_distributing_reward(&mut self) -> MultiTxsOperationProcessingResult;
}

pub trait ProtocolSettingsManager {
    ///
    fn change_minimum_validator_deposit(&mut self, value: U128);
    ///
    fn change_minimum_delegator_deposit(&mut self, value: U128);
    ///
    fn change_minimum_total_stake_price_for_booting(&mut self, value: U128);
    ///
    fn change_maximum_market_value_percent_of_near_fungible_tokens(&mut self, value: u16);
    ///
    fn change_maximum_market_value_percent_of_wrapped_appchain_token(&mut self, value: u16);
    ///
    fn change_minimum_validator_count(&mut self, value: U64);
    ///
    fn change_maximum_validator_count(&mut self, value: U64);
    ///
    fn change_maximum_validators_per_delegator(&mut self, value: U64);
    ///
    fn change_unlock_period_of_validator_deposit(&mut self, value: U64);
    ///
    fn change_unlock_period_of_delegator_deposit(&mut self, value: U64);
    ///
    fn change_maximum_era_count_of_unwithdrawn_reward(&mut self, value: U64);
    ///
    fn change_maximum_era_count_of_valid_appchain_message(&mut self, value: U64);
    ///
    fn change_validator_commission_percent(&mut self, value: u16);
}

pub trait AppchainSettingsManager {
    ///
    fn set_rpc_endpoint(&mut self, rpc_endpoint: String);
    ///
    fn set_subql_endpoint(&mut self, subql_endpoint: String);
    ///
    fn set_era_reward(&mut self, era_reward: U128);
}

pub trait AnchorSettingsManager {
    ///
    fn set_token_price_maintainer_account(&mut self, account_id: AccountId);
    ///
    fn set_relayer_account(&mut self, account_id: AccountId);
    ///
    fn turn_on_beefy_light_client_witness_mode(&mut self);
    ///
    fn turn_off_beefy_light_client_witness_mode(&mut self);
}

pub trait StakingManager {
    /// Decrease stake of an account (validator).
    /// This function can only be called by a validator.
    fn decrease_stake(&mut self, amount: U128);
    /// Unbond stake of an account (validator).
    /// This function can only be called by a validator.
    fn unbond_stake(&mut self);
    /// Enable delegation for an account (validator).
    /// This function can only be called by a validator.
    fn enable_delegation(&mut self);
    /// Disable delegation for an account (validator).
    /// This function can only be called by a validator.
    fn disable_delegation(&mut self);
    /// Decrease delegation of an account (delegator) to a validator.
    /// This function can only be called by a delegator.
    fn decrease_delegation(&mut self, validator_id: AccountId, amount: U128);
    /// Unbond delegation of an account (delegator) to a validator.
    /// This function can only be called by a delegator.
    fn unbond_delegation(&mut self, validator_id: AccountId);
    /// Withdraw unbonded stake(s) of a certain account.
    /// This function can be called by any account.
    fn withdraw_stake(&mut self, account_id: AccountId);
    /// Withdraw rewards of a certain validator.
    /// This function can be called by any account.
    fn withdraw_validator_rewards(&mut self, validator_id: AccountId);
    /// Withdraw rewards of a certain delegator to a validator.
    /// This function can be called by any account.
    fn withdraw_delegator_rewards(&mut self, delegator_id: AccountId, validator_id: AccountId);
}

pub trait SudoActions {
    /// Apply a certain `AppchainMessage`
    fn apply_appchain_messages(
        &mut self,
        appchain_messages: Vec<AppchainMessage>,
    ) -> Vec<AppchainMessageProcessingResult>;
    ///
    fn set_metadata_of_wrapped_appchain_token(&mut self, metadata: FungibleTokenMetadata);
    ///
    fn set_premined_balance_of_wrapped_appchain_token(
        &mut self,
        premined_beneficiary: AccountId,
        value: U128,
    );
    ///
    fn reset_validator_set_histories_to(&mut self, era_number: U64);
    ///
    fn clear_anchor_event_histories(&mut self);
    ///
    fn clear_appchain_notification_histories(&mut self);
    ///
    fn reset_beefy_light_client(&mut self, initial_public_keys: Vec<String>);
    ///
    fn clear_reward_distribution_records(&mut self, era_number: U64);
    ///
    fn clear_unbonded_stakes(&mut self);
    ///
    fn clear_unwithdrawn_rewards(&mut self);
    ///
    fn reset_validator_profiles_to(&mut self, era_number: U64);
    ///
    fn pause_asset_transfer(&mut self);
    ///
    fn resume_asset_transfer(&mut self);
    ///
    fn remove_staking_history_at(&mut self, index: U64);
}

pub trait ValidatorActions {
    ///
    fn set_validator_id_in_appchain(&mut self, account_id_in_appchain: String);
    ///
    fn set_validator_profile(&mut self, profile: HashMap<String, String>);
}

pub trait WrappedAppchainTokenManager {
    ///
    fn sync_basedata_of_wrapped_appchain_token(
        &mut self,
        metadata: FungibleTokenMetadata,
        premined_beneficiary: AccountId,
        premined_balance: U128,
    );
    ///
    fn set_account_of_wrapped_appchain_token(&mut self, contract_account: AccountId);
    ///
    fn set_price_of_wrapped_appchain_token(&mut self, price: U128);
    ///
    fn burn_wrapped_appchain_token(&self, receiver_id: String, amount: U128);
}
