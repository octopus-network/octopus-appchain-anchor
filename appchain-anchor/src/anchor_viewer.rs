use crate::{interfaces::AnchorViewer, *};

#[near_bindgen]
impl AnchorViewer for AppchainAnchor {
    //
    fn get_anchor_settings(&self) -> AnchorSettings {
        self.anchor_settings.get().unwrap()
    }
    //
    fn get_appchain_settings(&self) -> AppchainSettings {
        self.appchain_settings.get().unwrap()
    }
    //
    fn get_protocol_settings(&self) -> ProtocolSettings {
        self.protocol_settings.get().unwrap()
    }
    //
    fn get_oct_token(&self) -> OctToken {
        self.oct_token.get().unwrap()
    }
    //
    fn get_wrapped_appchain_token(&self) -> WrappedAppchainToken {
        self.wrapped_appchain_token.get().unwrap()
    }
    //
    fn get_near_fungible_tokens(&self) -> Vec<NearFungibleToken> {
        self.near_fungible_tokens.get().unwrap().to_vec()
    }
    //
    fn get_appchain_state(&self) -> AppchainState {
        self.appchain_state.clone()
    }
    //
    fn get_anchor_status(&self) -> AnchorStatus {
        AnchorStatus {
            total_stake_in_next_era: self.next_validator_set.get().unwrap().total_stake.into(),
            validator_count_in_next_era: self
                .next_validator_set
                .get()
                .unwrap()
                .validator_id_set
                .len()
                .into(),
            index_range_of_validator_set_history: self
                .validator_set_histories
                .get()
                .unwrap()
                .index_range(),
            index_range_of_anchor_event_history: self
                .anchor_event_histories
                .get()
                .unwrap()
                .index_range(),
            index_range_of_staking_history: self.staking_histories.get().unwrap().index_range(),
            permissionless_actions_status: self.permissionless_actions_status.get().unwrap(),
        }
    }
    //
    fn get_validator_set_info_of(&self, era_number: U64) -> Option<ValidatorSetInfo> {
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        if validator_set_histories.contains(&era_number.0) {
            let validator_set = validator_set_histories.get(&era_number.0).unwrap();
            Some(validator_set.to_validator_set_info())
        } else {
            None
        }
    }
    //
    fn get_processing_status_of(&self, era_number: U64) -> Option<ValidatorSetProcessingStatus> {
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        if validator_set_histories.contains(&era_number.0) {
            let validator_set = validator_set_histories.get(&era_number.0).unwrap();
            Some(validator_set.processing_status.clone())
        } else {
            None
        }
    }
    //
    fn get_index_range_of_staking_history(&self) -> IndexRange {
        self.staking_histories.get().unwrap().index_range()
    }
    //
    fn get_staking_histories(
        &self,
        start_index: U64,
        quantity: Option<U64>,
    ) -> Vec<StakingHistory> {
        let staking_histories = self.staking_histories.get().unwrap();
        let index_range = staking_histories.index_range();
        let mut result = Vec::<StakingHistory>::new();
        let start_index = match index_range.start_index.0 > start_index.0 {
            true => index_range.start_index.0,
            false => start_index.0,
        };
        let mut end_index = start_index
            + match quantity {
                Some(quantity) => match quantity.0 > 50 {
                    true => 49,
                    false => quantity.0 - 1,
                },
                None => 49,
            };
        end_index = match end_index < index_range.end_index.0 {
            true => end_index,
            false => index_range.end_index.0,
        };
        for index in start_index..end_index + 1 {
            if let Some(record) = staking_histories.get(&index) {
                result.push(record);
            }
        }
        result
    }
    //
    fn get_staking_history(&self, index: Option<U64>) -> Option<StakingHistory> {
        let index = match index {
            Some(index) => index,
            None => {
                self.staking_histories
                    .get()
                    .unwrap()
                    .index_range()
                    .end_index
            }
        };
        self.staking_histories.get().unwrap().get(&index.0)
    }
    //
    fn get_index_range_of_anchor_event_history(&self) -> IndexRange {
        self.anchor_event_histories.get().unwrap().index_range()
    }
    //
    fn get_anchor_event_history(&self, index: Option<U64>) -> Option<AnchorEventHistory> {
        let index = match index {
            Some(index) => index,
            None => {
                self.anchor_event_histories
                    .get()
                    .unwrap()
                    .index_range()
                    .end_index
            }
        };
        self.anchor_event_histories.get().unwrap().get(&index.0)
    }
    //
    fn get_anchor_event_histories(
        &self,
        start_index: U64,
        quantity: Option<U64>,
    ) -> Vec<AnchorEventHistory> {
        let anchor_event_histories = self.anchor_event_histories.get().unwrap();
        let index_range = anchor_event_histories.index_range();
        let mut result = Vec::<AnchorEventHistory>::new();
        let start_index = match index_range.start_index.0 > start_index.0 {
            true => index_range.start_index.0,
            false => start_index.0,
        };
        let mut end_index = index_range.start_index.0
            + match quantity {
                Some(quantity) => match quantity.0 > 50 {
                    true => 49,
                    false => quantity.0 - 1,
                },
                None => 49,
            };
        end_index = match end_index < index_range.end_index.0 {
            true => end_index,
            false => index_range.end_index.0,
        };
        for index in start_index..end_index + 1 {
            if let Some(record) = anchor_event_histories.get(&index) {
                result.push(record);
            }
        }
        result
    }
    //
    fn get_index_range_of_appchain_notification_history(&self) -> IndexRange {
        self.appchain_notification_histories
            .get()
            .unwrap()
            .index_range()
    }
    //
    fn get_appchain_notification_history(
        &self,
        index: Option<U64>,
    ) -> Option<AppchainNotificationHistory> {
        let index = match index {
            Some(index) => index,
            None => {
                self.appchain_notification_histories
                    .get()
                    .unwrap()
                    .index_range()
                    .end_index
            }
        };
        self.appchain_notification_histories
            .get()
            .unwrap()
            .get(&index.0)
    }
    //
    fn get_appchain_notification_histories(
        &self,
        start_index: U64,
        quantity: Option<U64>,
    ) -> Vec<AppchainNotificationHistory> {
        let appchain_notification_histories = self.appchain_notification_histories.get().unwrap();
        let index_range = appchain_notification_histories.index_range();
        let mut result = Vec::<AppchainNotificationHistory>::new();
        let start_index = match index_range.start_index.0 > start_index.0 {
            true => index_range.start_index.0,
            false => start_index.0,
        };
        let mut end_index = start_index
            + match quantity {
                Some(quantity) => match quantity.0 > 50 {
                    true => 49,
                    false => quantity.0 - 1,
                },
                None => 49,
            };
        end_index = match end_index < index_range.end_index.0 {
            true => end_index,
            false => index_range.end_index.0,
        };
        for index in start_index..end_index + 1 {
            if let Some(record) = appchain_notification_histories.get(&index) {
                result.push(record);
            }
        }
        result
    }
    //
    fn get_validator_list_of(&self, era_number: Option<U64>) -> Vec<AppchainValidator> {
        if let Some(era_number) = era_number {
            if let Some(validator_set_of_era) = self
                .validator_set_histories
                .get()
                .unwrap()
                .get(&era_number.0)
            {
                validator_set_of_era.validator_set.get_validator_list()
            } else {
                Vec::new()
            }
        } else {
            self.next_validator_set.get().unwrap().get_validator_list()
        }
    }
    //
    fn get_delegators_of_validator_in_era(
        &self,
        era_number: Option<U64>,
        validator_id: AccountId,
    ) -> Vec<AppchainDelegator> {
        let mut result = Vec::<AppchainDelegator>::new();
        match era_number {
            Some(era_number) => {
                let validator_set_histories = self.validator_set_histories.get().unwrap();
                match validator_set_histories.get(&era_number.0) {
                    Some(validator_set) => {
                        match validator_set
                            .validator_set
                            .validator_id_to_delegator_id_set
                            .get(&validator_id)
                        {
                            Some(delegator_id_set) => {
                                let delegator_ids = delegator_id_set.to_vec();
                                delegator_ids.iter().for_each(|delegator_id| {
                                    let delegator = validator_set
                                        .validator_set
                                        .delegators
                                        .get(&(delegator_id.clone(), validator_id.clone()))
                                        .unwrap();
                                    result.push(AppchainDelegator {
                                        delegator_id: delegator_id.clone(),
                                        validator_id: validator_id.clone(),
                                        delegation_amount: U128::from(delegator.deposit_amount),
                                    });
                                });
                            }
                            None => (),
                        }
                    }
                    None => (),
                }
            }
            None => {
                let next_validator_set = self.next_validator_set.get().unwrap();
                match next_validator_set
                    .validator_id_to_delegator_id_set
                    .get(&validator_id)
                {
                    Some(delegator_id_set) => {
                        let delegator_ids = delegator_id_set.to_vec();
                        delegator_ids.iter().for_each(|delegator_id| {
                            let delegator = next_validator_set
                                .delegators
                                .get(&(delegator_id.clone(), validator_id.clone()))
                                .unwrap();
                            result.push(AppchainDelegator {
                                delegator_id: delegator_id.clone(),
                                validator_id: validator_id.clone(),
                                delegation_amount: U128::from(delegator.deposit_amount),
                            });
                        });
                    }
                    None => (),
                }
            }
        };
        result
    }
    //
    fn get_unbonded_stakes_of(&self, account_id: AccountId) -> Vec<UnbondedStake> {
        let protocol_settings = self.protocol_settings.get().unwrap();
        let mut result = Vec::<UnbondedStake>::new();
        if let Some(unbonded_stake_references) = self.unbonded_stakes.get(&account_id) {
            unbonded_stake_references.iter().for_each(|reference| {
                let validator_set = self
                    .validator_set_histories
                    .get()
                    .unwrap()
                    .get(&reference.era_number)
                    .unwrap();
                let staking_history = self
                    .staking_histories
                    .get()
                    .unwrap()
                    .get(&reference.staking_history_index)
                    .unwrap();
                match staking_history.staking_fact {
                    StakingFact::StakeDecreased {
                        validator_id,
                        amount,
                    }
                    | StakingFact::ValidatorUnbonded {
                        validator_id,
                        amount,
                    } => result.push(UnbondedStake {
                        era_number: U64::from(reference.era_number),
                        account_id: validator_id,
                        amount,
                        unlock_time: U64::from(
                            validator_set.start_timestamp
                                + protocol_settings.unlock_period_of_validator_deposit.0
                                    * SECONDS_OF_A_DAY
                                    * NANO_SECONDS_MULTIPLE,
                        ),
                    }),
                    StakingFact::DelegationDecreased {
                        delegator_id,
                        validator_id: _,
                        amount,
                    }
                    | StakingFact::DelegatorUnbonded {
                        delegator_id,
                        validator_id: _,
                        amount,
                    } => result.push(UnbondedStake {
                        era_number: U64::from(reference.era_number),
                        account_id: delegator_id,
                        amount,
                        unlock_time: U64::from(
                            validator_set.start_timestamp
                                + protocol_settings.unlock_period_of_delegator_deposit.0
                                    * SECONDS_OF_A_DAY
                                    * NANO_SECONDS_MULTIPLE,
                        ),
                    }),
                    _ => (),
                };
            });
        }
        result
    }
    //
    fn get_validator_rewards_of(
        &self,
        start_era: U64,
        end_era: U64,
        validator_id: AccountId,
    ) -> Vec<RewardHistory> {
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        let mut reward_histories = Vec::<RewardHistory>::new();
        for era_number in start_era.0..end_era.0 + 1 {
            if let Some(validator_set) = validator_set_histories.get(&era_number) {
                if let Some(reward) = validator_set.validator_rewards.get(&validator_id) {
                    let unwithdrawn_reward = match self
                        .unwithdrawn_validator_rewards
                        .get(&(era_number, validator_id.clone()))
                    {
                        Some(reward) => reward,
                        None => 0,
                    };
                    reward_histories.push(RewardHistory {
                        era_number: U64::from(era_number),
                        total_reward: U128::from(reward),
                        unwithdrawn_reward: U128::from(unwithdrawn_reward),
                    });
                }
            }
        }
        reward_histories
    }
    //
    fn get_delegator_rewards_of(
        &self,
        start_era: U64,
        end_era: U64,
        delegator_id: AccountId,
        validator_id: AccountId,
    ) -> Vec<RewardHistory> {
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        let mut reward_histories = Vec::<RewardHistory>::new();
        for era_number in start_era.0..end_era.0 + 1 {
            if let Some(validator_set) = validator_set_histories.get(&era_number) {
                if let Some(reward) = validator_set
                    .delegator_rewards
                    .get(&(delegator_id.clone(), validator_id.clone()))
                {
                    let unwithdrawn_reward = match self.unwithdrawn_delegator_rewards.get(&(
                        era_number,
                        delegator_id.clone(),
                        validator_id.clone(),
                    )) {
                        Some(reward) => reward,
                        None => 0,
                    };
                    reward_histories.push(RewardHistory {
                        era_number: U64::from(era_number),
                        total_reward: U128::from(reward),
                        unwithdrawn_reward: U128::from(unwithdrawn_reward),
                    });
                }
            }
        }
        reward_histories
    }
    //
    fn get_storage_balance(&self) -> U128 {
        U128::from(u128::from(env::storage_usage()) * env::storage_byte_cost())
    }
    //
    fn get_validator_deposit_of(&self, validator_id: AccountId, era_number: Option<U64>) -> U128 {
        if let Some(era_number) = era_number {
            let validator_set_histories = self.validator_set_histories.get().unwrap();
            if validator_set_histories.contains(&era_number.0) {
                let validator_set = validator_set_histories.get(&era_number.0).unwrap();
                if let Some(validator) = validator_set.validator_set.validators.get(&validator_id) {
                    return U128::from(validator.deposit_amount);
                }
            }
        } else {
            if let Some(validator) = self
                .next_validator_set
                .get()
                .unwrap()
                .validators
                .get(&validator_id)
            {
                return U128::from(validator.deposit_amount);
            }
        }
        U128::from(0)
    }
    //
    fn get_delegator_deposit_of(
        &self,
        delegator_id: AccountId,
        validator_id: AccountId,
        era_number: Option<U64>,
    ) -> U128 {
        if let Some(era_number) = era_number {
            let validator_set_histories = self.validator_set_histories.get().unwrap();
            if validator_set_histories.contains(&era_number.0) {
                let validator_set = validator_set_histories.get(&era_number.0).unwrap();
                if let Some(delegator) = validator_set
                    .validator_set
                    .delegators
                    .get(&(delegator_id.clone(), validator_id.clone()))
                {
                    return U128::from(delegator.deposit_amount);
                }
            }
        } else {
            if let Some(delegator) = self
                .next_validator_set
                .get()
                .unwrap()
                .delegators
                .get(&(delegator_id.clone(), validator_id.clone()))
            {
                return U128::from(delegator.deposit_amount);
            }
        }
        U128::from(0)
    }
    //
    fn get_delegations_of(
        &self,
        delegator_id: AccountId,
        era_number: Option<U64>,
    ) -> Vec<AppchainDelegator> {
        let validator_set = match era_number {
            Some(era_number) => {
                if let Some(validator_set_of_era) = self
                    .validator_set_histories
                    .get()
                    .unwrap()
                    .get(&era_number.0)
                {
                    validator_set_of_era.validator_set
                } else {
                    return Vec::new();
                }
            }
            None => self.next_validator_set.get().unwrap(),
        };
        let mut result = Vec::<AppchainDelegator>::new();
        if let Some(validator_id_set) = validator_set
            .delegator_id_to_validator_id_set
            .get(&delegator_id)
        {
            let validator_ids = validator_id_set.to_vec();
            validator_ids.iter().for_each(|validator_id| {
                if let Some(delegator) = validator_set
                    .delegators
                    .get(&(delegator_id.clone(), validator_id.clone()))
                {
                    result.push(AppchainDelegator {
                        delegator_id: delegator_id.clone(),
                        validator_id: validator_id.clone(),
                        delegation_amount: U128::from(delegator.deposit_amount),
                    });
                }
            });
        }
        result
    }
    //
    fn get_validator_profile(&self, validator_id: AccountId) -> Option<ValidatorProfile> {
        self.validator_profiles.get().unwrap().get(&validator_id)
    }
    //
    fn get_validator_profiles(&self) -> Vec<ValidatorProfile> {
        let mut results = Vec::<ValidatorProfile>::new();
        let validator_profiles = self.validator_profiles.get().unwrap();
        validator_profiles
            .get_validator_ids()
            .iter()
            .for_each(|validator_id| {
                if let Some(profile) = validator_profiles.get(validator_id) {
                    results.push(profile);
                }
            });
        results
    }
    //
    fn get_validator_profile_by_id_in_appchain(
        &self,
        validator_id_in_appchain: String,
    ) -> Option<ValidatorProfile> {
        let formatted_id = AccountIdInAppchain::new(Some(validator_id_in_appchain.clone()));
        formatted_id.assert_valid();
        self.validator_profiles
            .get()
            .unwrap()
            .get_by_id_in_appchain(&formatted_id.to_string())
    }
    //
    fn get_latest_commitment_of_appchain(&self) -> Option<AppchainCommitment> {
        if let Some(light_client) = self.beefy_light_client_state.get() {
            if let Some(commitment) = light_client.get_latest_commitment() {
                return Some(AppchainCommitment {
                    payload: commitment.payload,
                    block_number: U64::from(commitment.block_number),
                    validator_set_id: commitment.validator_set_id,
                });
            }
        }
        None
    }
    //
    fn get_beefy_light_client_status(&self) -> BeefyLightClientStatus {
        if let Some(light_client) = self.beefy_light_client_state.get() {
            if light_client.is_updating_state() {
                BeefyLightClientStatus::UpdatingState
            } else {
                BeefyLightClientStatus::Ready
            }
        } else {
            BeefyLightClientStatus::Uninitialized
        }
    }
}
