use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::{env, near_bindgen, AccountId};

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
            native_near_token: old_contract.native_near_token,
        };
        //
        //
        new_contract
    }
    //
    pub fn clear_all_state(&mut self) {
        self.assert_owner();
        if let Some(validator_set_histories) = self.validator_set_histories.get() {
            if validator_set_histories.len() > 0 {
                panic!("There are still some validator set histories.");
            }
        }
        self.validator_set_histories.remove();
        if let Some(staking_histories) = self.staking_histories.get() {
            if staking_histories.len() > 0 {
                panic!("There are still some staking histories.");
            }
        }
        self.staking_histories.remove();
        if let Some(user_staking_histories) = self.user_staking_histories.get() {
            if user_staking_histories.len() > 0 {
                panic!("There are still some user staking histories.");
            }
        }
        self.user_staking_histories.remove();
        if let Some(appchain_messages) = self.appchain_messages.get() {
            if appchain_messages.len() > 0 {
                panic!("There are still some appchain messages.");
            }
        }
        self.appchain_messages.remove();
        if let Some(appchain_notification_histories) = self.appchain_notification_histories.get() {
            if appchain_notification_histories.len() > 0 {
                panic!("There are still some appchain notification histories.");
            }
        }
        self.appchain_notification_histories.remove();
        //
        //
        //
        if let Some(mut next_validator_set) = self.next_validator_set.get() {
            next_validator_set.clear(Gas::ONE_TERA * 150);
        }
        self.next_validator_set.remove();
        if let Some(mut appchain_challenges) = self.appchain_challenges.get() {
            appchain_challenges.clear(Gas::ONE_TERA * 10);
        }
        self.appchain_challenges.remove();
        if let Some(mut near_fungible_tokens) = self.near_fungible_tokens.get() {
            near_fungible_tokens.clear();
        }
        self.near_fungible_tokens.remove();
        if let Some(mut validator_profiles) = self.validator_profiles.get() {
            validator_profiles.clear();
        }
        self.validator_profiles.remove();
        if let Some(mut wrapped_appchain_nfts) = self.wrapped_appchain_nfts.get() {
            wrapped_appchain_nfts.clear();
        }
        self.wrapped_appchain_nfts.remove();
        //
        self.oct_token.remove();
        self.wrapped_appchain_token.remove();
        self.appchain_settings.remove();
        self.anchor_settings.remove();
        self.protocol_settings.remove();
        self.permissionless_actions_status.remove();
        self.beefy_light_client_state.remove();
        self.reward_distribution_records.remove();
        self.native_near_token.remove();
        // self.unwithdrawn_validator_rewards.clear();
        // self.unwithdrawn_delegator_rewards.clear();
        // self.unbonded_stakes.clear();
    }
    //
    pub fn clear_validator_set_histories(&mut self) -> String {
        self.assert_owner();
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        if validator_set_histories.len() == 0 {
            return "No more validator set histories.".to_string();
        }
        let max_gas = Gas::ONE_TERA * 170;
        let mut era_number = validator_set_histories.index_range().start_index;
        while env::used_gas() < max_gas && validator_set_histories.get(&era_number.0).is_none() {
            validator_set_histories.remove_first(max_gas);
            era_number = validator_set_histories.index_range().start_index;
        }
        let mut validator_set_of_era = validator_set_histories.get(&era_number.0).unwrap();
        let mut result = (MultiTxsOperationProcessingResult::NeedMoreGas, None);
        while env::used_gas() < max_gas && result.0.is_need_more_gas() {
            result = match RemovingValidatorSetSteps::recover() {
                RemovingValidatorSetSteps::ClearingRewardDistributionRecords {
                    appchain_message_nonce_index,
                    validator_index,
                    delegator_index,
                } => {
                    let mut reward_distribution_records =
                        self.reward_distribution_records.get().unwrap();
                    let mut result = reward_distribution_records.clear(
                        &validator_set_of_era,
                        &era_number.0,
                        appchain_message_nonce_index,
                        validator_index,
                        delegator_index,
                        max_gas,
                    );
                    self.reward_distribution_records
                        .set(&reward_distribution_records);
                    if result.is_ok() {
                        RemovingValidatorSetSteps::ClearingRewardDistributionRecordsInValidatorSet {
                            validator_index: 0,
                            delegator_index: 0,
                        }
                        .save();
                        result = MultiTxsOperationProcessingResult::NeedMoreGas;
                    }
                    (result, Some(RemovingValidatorSetSteps::recover()))
                }
                RemovingValidatorSetSteps::ClearingRewardDistributionRecordsInValidatorSet {
                    validator_index,
                    delegator_index,
                } => {
                    let mut result = validator_set_of_era.clear_reward_distribution_records(
                        validator_index,
                        delegator_index,
                        max_gas,
                    );
                    validator_set_histories.insert(&era_number.0, &validator_set_of_era);
                    if result.is_ok() {
                        RemovingValidatorSetSteps::ClearingUnwithdrawnRewardRecordsForValidatorSet {
                            validator_index: 0,
                            delegator_index: 0,
                        }
                        .save();
                        result = MultiTxsOperationProcessingResult::NeedMoreGas;
                    }
                    (result, Some(RemovingValidatorSetSteps::recover()))
                }
                RemovingValidatorSetSteps::ClearingUnwithdrawnRewardRecordsForValidatorSet {
                    validator_index,
                    delegator_index,
                } => {
                    let mut result = self.clear_unwithdrawn_reward_records(
                        &validator_set_of_era,
                        validator_index,
                        delegator_index,
                        max_gas,
                    );
                    if result.is_ok() {
                        RemovingValidatorSetSteps::ClearingOldestValidatorSet.save();
                        result = MultiTxsOperationProcessingResult::NeedMoreGas;
                    }
                    (result, Some(RemovingValidatorSetSteps::recover()))
                }
                RemovingValidatorSetSteps::ClearingOldestValidatorSet => {
                    let result = validator_set_histories.remove_first(max_gas);
                    if result.is_ok() {
                        RemovingValidatorSetSteps::clear();
                        (result, None)
                    } else {
                        (result, Some(RemovingValidatorSetSteps::recover()))
                    }
                }
            };
        }
        self.validator_set_histories.set(&validator_set_histories);
        format!("Era {}: {:?}", era_number.0, result)
    }
    //
    pub fn clear_appchain_notification_histories(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let mut appchain_notification_histories =
            self.appchain_notification_histories.get().unwrap();
        let max_gas = Gas::ONE_TERA * 170;
        let mut result = MultiTxsOperationProcessingResult::Ok;
        while env::used_gas() < max_gas && result.is_ok() {
            result = appchain_notification_histories.remove_first(max_gas);
        }
        self.appchain_notification_histories
            .set(&appchain_notification_histories);
        result
    }
    //
    pub fn clear_staking_histories(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let mut staking_histories = self.staking_histories.get().unwrap();
        let max_gas = Gas::ONE_TERA * 170;
        let mut result = MultiTxsOperationProcessingResult::Ok;
        while env::used_gas() < max_gas && result.is_ok() {
            result = staking_histories.remove_first(max_gas);
        }
        self.staking_histories.set(&staking_histories);
        result
    }
    //
    pub fn clear_user_staking_histories(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let mut user_staking_histories = self.user_staking_histories.get().unwrap();
        let mut result = MultiTxsOperationProcessingResult::Ok;
        while result.is_ok() {
            result = user_staking_histories.clear();
        }
        self.user_staking_histories.set(&user_staking_histories);
        result
    }
    //
    pub fn clear_appchain_messages(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let mut appchain_messages = self.appchain_messages.get().unwrap();
        let mut result = MultiTxsOperationProcessingResult::Ok;
        while result.is_ok() {
            result = appchain_messages.clear();
        }
        self.appchain_messages.set(&appchain_messages);
        result
    }
}

impl AppchainAnchor {
    pub fn clear_unwithdrawn_reward_records(
        &mut self,
        validator_set_of_era: &ValidatorSetOfEra,
        validator_index_start: u64,
        delegator_index_start: u64,
        max_gas: Gas,
    ) -> MultiTxsOperationProcessingResult {
        let era_number = validator_set_of_era.era_number();
        let validator_ids = validator_set_of_era.get_validator_ids();
        let mut validator_index = 0;
        let mut delegator_index = 0;
        for validator_id in validator_ids {
            if validator_index < validator_index_start {
                validator_index += 1;
                continue;
            }
            let delegator_ids = validator_set_of_era.get_delegator_ids_of(&validator_id);
            for delegator_id in delegator_ids {
                if validator_index == validator_index_start
                    && delegator_index < delegator_index_start
                {
                    delegator_index += 1;
                    continue;
                }
                self.unwithdrawn_delegator_rewards.remove(&(
                    era_number,
                    delegator_id.clone(),
                    validator_id.clone(),
                ));
                delegator_index += 1;
                if env::used_gas() >= max_gas {
                    RemovingValidatorSetSteps::ClearingUnwithdrawnRewardRecordsForValidatorSet {
                        validator_index,
                        delegator_index,
                    }
                    .save();
                    return MultiTxsOperationProcessingResult::NeedMoreGas;
                }
            }
            self.unwithdrawn_validator_rewards
                .remove(&(era_number, validator_id.clone()));
            validator_index += 1;
            delegator_index = 0;
            if env::used_gas() >= max_gas {
                RemovingValidatorSetSteps::ClearingUnwithdrawnRewardRecordsForValidatorSet {
                    validator_index,
                    delegator_index,
                }
                .save();
                return MultiTxsOperationProcessingResult::NeedMoreGas;
            }
        }
        MultiTxsOperationProcessingResult::Ok
    }
}
