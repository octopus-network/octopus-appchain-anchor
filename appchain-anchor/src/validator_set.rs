use near_sdk::BlockHeight;

use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Validator {
    /// The validator's id in NEAR protocol.
    pub validator_id: AccountId,
    /// The validator's id in the appchain.
    pub validator_id_in_appchain: String,
    /// The block height when the validator is registered.
    pub registered_block_height: BlockHeight,
    /// The timestamp when the validator is registered.
    pub registered_timestamp: Timestamp,
    /// Total deposited balance of the validator.
    pub deposit_amount: Balance,
    /// Total stake of the validator, including delegations of all delegators.
    pub total_stake: Balance,
    /// Whether the validator accepts delegation from delegators.
    pub can_be_delegated_to: bool,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Delegator {
    /// The delegator's id in NEAR protocol.
    pub delegator_id: AccountId,
    /// The validator's id in NEAR protocol, which the delegator delegates his rights to.
    pub validator_id: AccountId,
    /// The block height when the delegator is registered.
    pub registered_block_height: BlockHeight,
    /// The timestamp when the delegator is registered.
    pub registered_timestamp: Timestamp,
    /// Delegated balance of the delegator.
    pub deposit_amount: Balance,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorSet {
    /// The number of era in appchain.
    pub era_number: u64,
    /// The set of account id of validators.
    pub validator_id_set: UnorderedSet<AccountId>,
    /// The map from validator id to the set of its delegators' id.
    pub validator_id_to_delegator_id_set: LookupMap<AccountId, UnorderedSet<AccountId>>,
    /// The map from delegator id to the set of its validators' id that
    /// the delegator delegates his/her voting rights to.
    pub delegator_id_to_validator_id_set: LookupMap<AccountId, UnorderedSet<AccountId>>,
    /// The validators data, mapped by their account id in NEAR protocol.
    pub validators: LookupMap<AccountId, Validator>,
    /// The delegators data, mapped by the tuple of their delegator account id and
    /// validator account id in NEAR protocol.
    pub delegators: LookupMap<(AccountId, AccountId), Delegator>,
    /// Total stake of current set
    pub total_stake: Balance,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorSetOfEra {
    /// The validator set of this era
    pub validator_set: ValidatorSet,
    /// The block height when the era starts.
    pub start_block_height: BlockHeight,
    /// The timestamp when the era starts.
    pub start_timestamp: Timestamp,
    /// The index of the latest staking history happened in the era of corresponding appchain.
    pub staking_history_index: u64,
    /// The set of validator id which will not be profited.
    pub unprofitable_validator_id_set: UnorderedSet<AccountId>,
    /// Total stake excluding all unprofitable validators' stake.
    pub valid_total_stake: Balance,
    /// The rewards of validators in this era
    pub validator_rewards: LookupMap<AccountId, Balance>,
    /// The rewards of delegators in this era
    pub delegator_rewards: LookupMap<(AccountId, AccountId), Balance>,
    /// The status of creation of this set
    pub processing_status: ValidatorSetProcessingStatus,
}

pub trait ValidatorSetActions {
    /// Get validator list of current validator set
    fn get_validator_list(&self) -> Vec<AppchainValidator>;
    /// Apply a certain `staking history` to the validator set.
    fn apply_staking_history(&mut self, staking_history: &StakingHistory);
}

impl ValidatorSet {
    ///
    pub fn new(era_number: u64) -> Self {
        Self {
            era_number,
            validator_id_set: UnorderedSet::new(
                StorageKey::ValidatorIdsOfEra(era_number).into_bytes(),
            ),
            validator_id_to_delegator_id_set: LookupMap::new(
                StorageKey::ValidatorToDelegatorsMapOfEra(era_number).into_bytes(),
            ),
            delegator_id_to_validator_id_set: LookupMap::new(
                StorageKey::DelegatorToValidatorsMapOfEra(era_number).into_bytes(),
            ),
            validators: LookupMap::new(StorageKey::ValidatorsOfEra(era_number).into_bytes()),
            delegators: LookupMap::new(StorageKey::DelegatorsOfEra(era_number).into_bytes()),
            total_stake: 0,
        }
    }
    ///
    pub fn clear(&mut self) {
        let validator_ids = self.validator_id_set.to_vec();
        for validator_id in validator_ids {
            if let Some(mut delegator_id_set) =
                self.validator_id_to_delegator_id_set.get(&validator_id)
            {
                let delegator_ids = delegator_id_set.to_vec();
                for delegator_id in delegator_ids {
                    self.delegators
                        .remove(&(delegator_id.clone(), validator_id.clone()));
                    if let Some(mut validator_id_set_of_delegator) =
                        self.delegator_id_to_validator_id_set.get(&delegator_id)
                    {
                        validator_id_set_of_delegator.clear();
                        self.delegator_id_to_validator_id_set.remove(&delegator_id);
                    }
                }
                delegator_id_set.clear();
                self.validator_id_to_delegator_id_set.remove(&validator_id);
                self.validators.remove(&validator_id);
            }
        }
        self.validator_id_set.clear();
        self.total_stake = 0;
    }
}

impl ValidatorSetActions for ValidatorSet {
    //
    fn get_validator_list(&self) -> Vec<AppchainValidator> {
        let validator_ids = self.validator_id_set.to_vec();
        validator_ids
            .iter()
            .map(|validator_id| {
                let validator = self.validators.get(validator_id).unwrap();
                let mut delegators_count: u64 = 0;
                if let Some(delegator_id_set) =
                    self.validator_id_to_delegator_id_set.get(validator_id)
                {
                    delegators_count = delegator_id_set.len();
                }
                return AppchainValidator {
                    validator_id: validator.validator_id,
                    validator_id_in_appchain: validator.validator_id_in_appchain,
                    deposit_amount: U128::from(validator.deposit_amount),
                    total_stake: U128::from(validator.total_stake),
                    delegators_count: U64::from(delegators_count),
                    can_be_delegated_to: validator.can_be_delegated_to,
                };
            })
            .collect()
    }
    //
    fn apply_staking_history(&mut self, staking_history: &StakingHistory) {
        match &staking_history.staking_fact {
            StakingFact::ValidatorRegistered {
                validator_id,
                validator_id_in_appchain,
                amount,
                can_be_delegated_to,
            } => {
                self.validator_id_set.insert(validator_id);
                self.validators.insert(
                    validator_id,
                    &Validator {
                        validator_id: validator_id.clone(),
                        validator_id_in_appchain: validator_id_in_appchain.to_string(),
                        registered_block_height: env::block_index(),
                        registered_timestamp: env::block_timestamp(),
                        deposit_amount: amount.0,
                        total_stake: amount.0,
                        can_be_delegated_to: *can_be_delegated_to,
                    },
                );
                self.total_stake += amount.0;
            }
            StakingFact::StakeIncreased {
                validator_id,
                amount,
            } => {
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.deposit_amount += amount.0;
                validator.total_stake += amount.0;
                self.validators.insert(validator_id, &validator);
                self.total_stake += amount.0;
            }
            StakingFact::StakeDecreased {
                validator_id,
                amount,
            } => {
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.deposit_amount -= amount.0;
                validator.total_stake -= amount.0;
                self.validators.insert(validator_id, &validator);
                self.total_stake -= amount.0;
            }
            StakingFact::ValidatorUnbonded {
                validator_id,
                amount: _,
            }
            | StakingFact::ValidatorAutoUnbonded {
                validator_id,
                amount: _,
            } => {
                if let Some(delegator_id_set) =
                    self.validator_id_to_delegator_id_set.get(validator_id)
                {
                    let delegator_ids = delegator_id_set.to_vec();
                    delegator_ids.iter().for_each(|delegator_id| {
                        self.delegators
                            .remove(&(delegator_id.clone(), validator_id.clone()));
                        if let Some(mut validator_id_set) =
                            self.delegator_id_to_validator_id_set.get(delegator_id)
                        {
                            validator_id_set.remove(validator_id);
                            if validator_id_set.len() > 0 {
                                self.delegator_id_to_validator_id_set
                                    .insert(delegator_id, &validator_id_set);
                            } else {
                                self.delegator_id_to_validator_id_set.remove(delegator_id);
                            }
                        }
                    });
                    self.validator_id_to_delegator_id_set.remove(validator_id);
                }
                let validator = self.validators.remove(validator_id).unwrap();
                self.total_stake -= validator.total_stake;
                self.validator_id_set.remove(validator_id);
            }
            StakingFact::ValidatorDelegationEnabled { validator_id } => {
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.can_be_delegated_to = true;
                self.validators.insert(validator_id, &validator);
            }
            StakingFact::ValidatorDelegationDisabled { validator_id } => {
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.can_be_delegated_to = false;
                self.validators.insert(validator_id, &validator);
            }
            StakingFact::DelegatorRegistered {
                delegator_id,
                validator_id,
                amount,
            } => {
                self.delegators.insert(
                    &(delegator_id.clone(), validator_id.clone()),
                    &Delegator {
                        delegator_id: delegator_id.clone(),
                        validator_id: validator_id.clone(),
                        registered_block_height: env::block_index(),
                        registered_timestamp: env::block_timestamp(),
                        deposit_amount: amount.0,
                    },
                );
                if !self
                    .validator_id_to_delegator_id_set
                    .contains_key(validator_id)
                {
                    self.validator_id_to_delegator_id_set.insert(
                        validator_id,
                        &UnorderedSet::new(
                            StorageKey::DelegatorIdsInMapOfVToDOfEra {
                                era_number: self.era_number,
                                validator_id: validator_id.clone(),
                            }
                            .into_bytes(),
                        ),
                    );
                }
                let mut delegator_id_set = self
                    .validator_id_to_delegator_id_set
                    .get(validator_id)
                    .unwrap();
                delegator_id_set.insert(delegator_id);
                self.validator_id_to_delegator_id_set
                    .insert(validator_id, &delegator_id_set);
                if !self
                    .delegator_id_to_validator_id_set
                    .contains_key(delegator_id)
                {
                    self.delegator_id_to_validator_id_set.insert(
                        delegator_id,
                        &UnorderedSet::new(
                            StorageKey::ValidatorIdsInMapOfDToVOfEra {
                                era_number: self.era_number,
                                delegator_id: delegator_id.clone(),
                            }
                            .into_bytes(),
                        ),
                    );
                }
                let mut validator_id_set = self
                    .delegator_id_to_validator_id_set
                    .get(delegator_id)
                    .unwrap();
                validator_id_set.insert(validator_id);
                self.delegator_id_to_validator_id_set
                    .insert(delegator_id, &validator_id_set);
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.total_stake += amount.0;
                self.validators.insert(validator_id, &validator);
                self.total_stake += amount.0;
            }
            StakingFact::DelegationIncreased {
                delegator_id,
                validator_id,
                amount,
            } => {
                let mut delegator = self
                    .delegators
                    .get(&(delegator_id.clone(), validator_id.clone()))
                    .unwrap();
                delegator.deposit_amount += amount.0;
                self.delegators
                    .insert(&(delegator_id.clone(), validator_id.clone()), &delegator);
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.total_stake += amount.0;
                self.validators.insert(validator_id, &validator);
                self.total_stake += amount.0;
            }
            StakingFact::DelegationDecreased {
                delegator_id,
                validator_id,
                amount,
            } => {
                let mut delegator = self
                    .delegators
                    .get(&(delegator_id.clone(), validator_id.clone()))
                    .unwrap();
                delegator.deposit_amount -= amount.0;
                self.delegators
                    .insert(&(delegator_id.clone(), validator_id.clone()), &delegator);
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.total_stake -= amount.0;
                self.validators.insert(validator_id, &validator);
                self.total_stake -= amount.0;
            }
            StakingFact::DelegatorUnbonded {
                delegator_id,
                validator_id,
                amount: _,
            }
            | StakingFact::DelegatorAutoUnbonded {
                delegator_id,
                validator_id,
                amount: _,
            } => {
                let mut delegator_id_set = self
                    .validator_id_to_delegator_id_set
                    .get(validator_id)
                    .unwrap();
                delegator_id_set.remove(delegator_id);
                if delegator_id_set.len() > 0 {
                    self.validator_id_to_delegator_id_set
                        .insert(validator_id, &delegator_id_set);
                } else {
                    self.validator_id_to_delegator_id_set.remove(validator_id);
                }
                let mut validator_id_set = self
                    .delegator_id_to_validator_id_set
                    .get(delegator_id)
                    .unwrap();
                validator_id_set.remove(validator_id);
                if validator_id_set.len() > 0 {
                    self.delegator_id_to_validator_id_set
                        .insert(delegator_id, &validator_id_set);
                } else {
                    self.delegator_id_to_validator_id_set.remove(delegator_id);
                }
                let delegator = self
                    .delegators
                    .remove(&(delegator_id.clone(), validator_id.clone()))
                    .unwrap();
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.total_stake -= delegator.deposit_amount;
                self.validators.insert(validator_id, &validator);
                self.total_stake -= delegator.deposit_amount;
            }
        }
    }
}

impl ValidatorSetOfEra {
    ///
    pub fn new(era_number: u64, staking_history_index: u64) -> Self {
        Self {
            start_block_height: env::block_index(),
            start_timestamp: env::block_timestamp(),
            staking_history_index,
            unprofitable_validator_id_set: UnorderedSet::new(
                StorageKey::UnprofitableValidatorIdsOfEra(era_number).into_bytes(),
            ),
            validator_set: ValidatorSet::new(era_number),
            valid_total_stake: 0,
            validator_rewards: LookupMap::new(
                StorageKey::ValidatorRewardsOfEra(era_number).into_bytes(),
            ),
            delegator_rewards: LookupMap::new(
                StorageKey::DelegatorRewardsOfEra(era_number).into_bytes(),
            ),
            processing_status: ValidatorSetProcessingStatus::CopyingFromLastEra {
                copying_validator_index: U64::from(0),
                copying_delegator_index: U64::from(0),
            },
        }
    }
    ///
    pub fn set_unprofitable_validator_ids(&mut self, unprofitable_validator_ids: Vec<AccountId>) {
        unprofitable_validator_ids.iter().for_each(|v_id| {
            self.unprofitable_validator_id_set.insert(&v_id);
        });
    }
    ///
    pub fn calculate_valid_total_stake(&mut self) {
        let unprofitable_validator_ids = self.unprofitable_validator_id_set.to_vec();
        self.valid_total_stake = self.validator_set.total_stake;
        unprofitable_validator_ids.iter().for_each(|v_id| {
            let validator = self.validator_set.validators.get(v_id).unwrap();
            self.valid_total_stake -= validator.total_stake;
        });
    }
    ///
    pub fn to_validator_set_info(&self) -> ValidatorSetInfo {
        ValidatorSetInfo {
            era_number: U64::from(self.validator_set.era_number),
            total_stake: U128::from(self.validator_set.total_stake),
            validator_list: self.validator_set.get_validator_list(),
            start_block_height: U64::from(self.start_block_height),
            start_timestamp: U64::from(self.start_timestamp),
            staking_history_index: U64::from(self.staking_history_index),
            unprofitable_validator_ids: self.unprofitable_validator_id_set.to_vec(),
            valid_total_stake: U128::from(self.valid_total_stake),
            processing_status: self.processing_status.clone(),
        }
    }
    ///
    pub fn clear_reward_distribution_records(&mut self) {
        let validator_ids = self.validator_set.validator_id_set.to_vec();
        for validator_id in validator_ids {
            if self.unprofitable_validator_id_set.contains(&validator_id) {
                continue;
            }
            if let Some(delegator_id_set) = self
                .validator_set
                .validator_id_to_delegator_id_set
                .get(&validator_id)
            {
                let delegator_ids = delegator_id_set.to_vec();
                for delegator_id in delegator_ids {
                    self.delegator_rewards
                        .remove(&(delegator_id.clone(), validator_id.clone()));
                }
            }
            self.validator_rewards.remove(&validator_id);
        }
        self.unprofitable_validator_id_set.clear();
    }
    ///
    pub fn clear(&mut self) {
        self.clear_reward_distribution_records();
        self.validator_set.clear();
    }
}

impl ValidatorSetActions for ValidatorSetOfEra {
    //
    fn get_validator_list(&self) -> Vec<AppchainValidator> {
        match self.processing_status {
            ValidatorSetProcessingStatus::ReadyForDistributingReward
            | ValidatorSetProcessingStatus::DistributingReward { .. }
            | ValidatorSetProcessingStatus::Completed => self.validator_set.get_validator_list(),
            _ => Vec::new(),
        }
    }
    //
    fn apply_staking_history(&mut self, staking_history: &StakingHistory) {
        self.validator_set.apply_staking_history(staking_history);
    }
}

impl IndexedAndClearable for ValidatorSetOfEra {
    //
    fn set_index(&mut self, _index: &u64) {
        ()
    }
    //
    fn clear_extra_storage(&mut self) {
        self.clear();
    }
}
