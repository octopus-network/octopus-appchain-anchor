use near_sdk::BlockHeight;

use crate::*;

/// Appchain validator of an appchain.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Validator {
    /// The validator's id in NEAR protocol.
    pub validator_id: AccountId,
    /// The validator's id in the appchain.
    pub validator_id_in_appchain: AccountIdInAppchain,
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
    pub validator_ids: UnorderedSet<AccountId>,
    /// The map from validator id to its delegators' ids.
    pub validator_id_to_delegator_ids: LookupMap<AccountId, UnorderedSet<AccountId>>,
    /// The map from delegator id to the validators' ids that
    /// the delegator delegates his/her voting rights to.
    pub delegator_id_to_validator_ids: LookupMap<AccountId, UnorderedSet<AccountId>>,
    /// The validators data, mapped by their account id in NEAR protocol.
    pub validators: LookupMap<AccountId, Validator>,
    /// The delegators data, mapped by the tuple of their delegator account id and
    /// validator account id in NEAR protocol.
    pub delegators: LookupMap<(AccountId, AccountId), Delegator>,
    /// Total stake of current set
    pub total_stake: Balance,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum ProcessingStatus {
    CopyingFromLastEra {
        copying_validator_index: u64,
        copying_delegator_index: u64,
    },
    ApplyingStakingHistory {
        applying_index: u64,
    },
    ReadyForDistributingReward,
    DistributingReward {
        distributing_validator_index: u64,
        distributing_delegator_index: u64,
    },
    Completed,
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
    pub unprofitable_validator_ids: UnorderedSet<AccountId>,
    /// Total stake excluding all unprofitable validators' stake.
    pub valid_total_stake: Balance,
    /// The status of creation of this set
    pub processing_status: ProcessingStatus,
}

pub trait ValidatorSetActions {
    /// Apply a certain `staking history` to the validator set.
    fn apply_staking_history(&mut self, staking_history: &StakingHistory);
}

impl ValidatorSet {
    ///
    pub fn new(era_number: u64) -> Self {
        Self {
            era_number,
            validator_ids: UnorderedSet::new(
                StorageKey::ValidatorIdsInValidatorSet(era_number).into_bytes(),
            ),
            validator_id_to_delegator_ids: LookupMap::new(
                StorageKey::LookupMapOfVToDInValidatorSet(era_number).into_bytes(),
            ),
            delegator_id_to_validator_ids: LookupMap::new(
                StorageKey::LookupMapOfDToVInValidatorSet(era_number).into_bytes(),
            ),
            validators: LookupMap::new(
                StorageKey::ValidatorsInValidatorSet(era_number).into_bytes(),
            ),
            delegators: LookupMap::new(
                StorageKey::DelegatorsInValidatorSet(era_number).into_bytes(),
            ),
            total_stake: 0,
        }
    }
}

impl ValidatorSetActions for ValidatorSet {
    //
    fn apply_staking_history(&mut self, staking_history: &StakingHistory) {
        match &staking_history.staking_fact {
            types::StakingFact::ValidatorRegistered {
                validator_id,
                validator_id_in_appchain,
                amount,
                can_be_delegated_to,
            } => {
                self.validator_ids.insert(validator_id);
                self.validators.insert(
                    validator_id,
                    &Validator {
                        validator_id: validator_id.clone(),
                        validator_id_in_appchain: validator_id_in_appchain.clone(),
                        registered_block_height: env::block_index(),
                        registered_timestamp: env::block_timestamp(),
                        deposit_amount: amount.0,
                        total_stake: amount.0,
                        can_be_delegated_to: *can_be_delegated_to,
                    },
                );
                self.total_stake += amount.0;
            }
            types::StakingFact::StakeIncreased {
                validator_id,
                amount,
            } => {
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.deposit_amount += amount.0;
                self.validators.insert(validator_id, &validator);
                self.total_stake += amount.0;
            }
            types::StakingFact::StakeDecreased {
                validator_id,
                amount,
            } => {
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.deposit_amount -= amount.0;
                self.validators.insert(validator_id, &validator);
                self.total_stake -= amount.0;
            }
            types::StakingFact::ValidatorUnbonded { validator_id } => {
                let delegator_ids = self
                    .validator_id_to_delegator_ids
                    .get(validator_id)
                    .unwrap()
                    .to_vec();
                delegator_ids.iter().for_each(|d_id| {
                    self.delegators
                        .remove(&(d_id.clone(), validator_id.clone()));
                    if let Some(mut v_ids) = self.delegator_id_to_validator_ids.get(d_id) {
                        v_ids.remove(validator_id);
                    }
                });
                let validator = self.validators.remove(validator_id).unwrap();
                self.total_stake -= validator.total_stake;
                self.validator_ids.remove(validator_id);
            }
            types::StakingFact::ValidatorDelegationEnabled { validator_id } => {
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.can_be_delegated_to = true;
                self.validators.insert(validator_id, &validator);
            }
            types::StakingFact::ValidatorDelegationDisabled { validator_id } => {
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.can_be_delegated_to = false;
                self.validators.insert(validator_id, &validator);
            }
            types::StakingFact::DelegatorRegistered {
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
                let mut d_ids = match self.validator_id_to_delegator_ids.get(validator_id) {
                    Some(d_ids) => d_ids,
                    None => self
                        .validator_id_to_delegator_ids
                        .insert(
                            validator_id,
                            &UnorderedSet::new(
                                StorageKey::DelegatorIdsInLookupMapOfVToDInValidatorSet {
                                    era_number: self.era_number,
                                    validator_id: validator_id.clone(),
                                }
                                .into_bytes(),
                            ),
                        )
                        .unwrap(),
                };
                d_ids.insert(delegator_id);
                let mut v_ids = match self.delegator_id_to_validator_ids.get(delegator_id) {
                    Some(v_ids) => v_ids,
                    None => self
                        .delegator_id_to_validator_ids
                        .insert(
                            delegator_id,
                            &UnorderedSet::new(
                                StorageKey::ValidatorIdsInLookupMapOfDToVInValidatorSet {
                                    era_number: self.era_number,
                                    delegator_id: delegator_id.clone(),
                                }
                                .into_bytes(),
                            ),
                        )
                        .unwrap(),
                };
                v_ids.insert(validator_id);
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.total_stake += amount.0;
                self.validators.insert(validator_id, &validator);
                self.total_stake += amount.0;
            }
            types::StakingFact::DelegationIncreased {
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
            types::StakingFact::DelegationDecreased {
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
            types::StakingFact::DelegatorUnbonded {
                delegator_id,
                validator_id,
            } => {
                let mut d_ids = self
                    .validator_id_to_delegator_ids
                    .get(validator_id)
                    .unwrap();
                d_ids.remove(delegator_id);
                if d_ids.len() == 0 {
                    self.validator_id_to_delegator_ids.remove(validator_id);
                }
                let mut v_ids = self
                    .delegator_id_to_validator_ids
                    .get(delegator_id)
                    .unwrap();
                v_ids.remove(validator_id);
                if v_ids.len() == 0 {
                    self.delegator_id_to_validator_ids.remove(delegator_id);
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

impl ProcessingStatus {
    ///
    pub fn is_ready_for_distributing_reward(&self) -> bool {
        match self {
            ProcessingStatus::CopyingFromLastEra {
                copying_validator_index: _,
                copying_delegator_index: _,
            } => false,
            ProcessingStatus::ApplyingStakingHistory { applying_index: _ } => false,
            ProcessingStatus::ReadyForDistributingReward => true,
            ProcessingStatus::DistributingReward {
                distributing_validator_index: _,
                distributing_delegator_index: _,
            } => false,
            ProcessingStatus::Completed => false,
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
            unprofitable_validator_ids: UnorderedSet::new(
                StorageKey::UnprofitableValidatorIdsInValidatorSet(era_number).into_bytes(),
            ),
            validator_set: ValidatorSet::new(era_number),
            valid_total_stake: 0,
            processing_status: ProcessingStatus::CopyingFromLastEra {
                copying_validator_index: 0,
                copying_delegator_index: 0,
            },
        }
    }
    ///
    pub fn set_unprofitable_validator_ids(&mut self, unprofitable_validator_ids: Vec<AccountId>) {
        unprofitable_validator_ids.iter().for_each(|v_id| {
            self.unprofitable_validator_ids.insert(&v_id);
        });
    }
    ///
    pub fn calculate_valid_total_stake(&mut self) {
        let unprofitable_validator_ids = self.unprofitable_validator_ids.to_vec();
        self.valid_total_stake = self.validator_set.total_stake;
        unprofitable_validator_ids.iter().for_each(|v_id| {
            let validator = self.validator_set.validators.get(v_id).unwrap();
            self.valid_total_stake -= validator.total_stake;
        });
    }
}

impl ValidatorSetActions for ValidatorSetOfEra {
    //
    fn apply_staking_history(&mut self, staking_history: &StakingHistory) {
        self.validator_set.apply_staking_history(staking_history);
    }
}
