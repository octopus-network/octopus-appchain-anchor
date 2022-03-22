use near_sdk::BlockHeight;

use crate::*;

pub mod next_validator_set;
pub mod validator_set_of_era;

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
    era_number: u64,
    /// The set of account id of validators.
    validator_id_set: UnorderedSet<AccountId>,
    /// The map from validator id to the set of its delegators' id.
    validator_id_to_delegator_id_set: LookupMap<AccountId, UnorderedSet<AccountId>>,
    /// The map from delegator id to the set of its validators' id that
    /// the delegator delegates his/her voting rights to.
    delegator_id_to_validator_id_set: LookupMap<AccountId, UnorderedSet<AccountId>>,
    /// The validators data, mapped by their account id in NEAR protocol.
    validators: LookupMap<AccountId, Validator>,
    /// The delegators data, mapped by the tuple of their delegator account id and
    /// validator account id in NEAR protocol.
    delegators: LookupMap<(AccountId, AccountId), Delegator>,
    /// Total stake of current set
    total_stake: Balance,
}

pub trait ValidatorSetViewer {
    ///
    fn contains_validator(&self, validator_id: &AccountId) -> bool;
    ///
    fn contains_delegator(&self, delegator_id: &AccountId, validator_id: &AccountId) -> bool;
    ///
    fn get_validator(&self, validator_id: &AccountId) -> Option<Validator>;
    ///
    fn get_validator_by_index(&self, index: &u64) -> Option<Validator>;
    ///
    fn get_delegator(
        &self,
        delegator_id: &AccountId,
        validator_id: &AccountId,
    ) -> Option<Delegator>;
    ///
    fn get_delegator_by_index(&self, index: &u64, validator_id: &AccountId) -> Option<Delegator>;
    ///
    fn get_validator_ids(&self) -> Vec<AccountId>;
    ///
    fn get_validator_ids_of(&self, delegator_id: &AccountId) -> Vec<AccountId>;
    ///
    fn get_delegator_ids_of(&self, validator_id: &AccountId) -> Vec<AccountId>;
    ///
    fn get_validator_count_of(&self, delegator_id: &AccountId) -> u64;
    ///
    fn get_delegator_count_of(&self, validator_id: &AccountId) -> u64;
    ///
    fn era_number(&self) -> u64;
    ///
    fn total_stake(&self) -> u128;
    ///
    fn validator_count(&self) -> u64;
    ///
    fn delegator_count(&self) -> u64;
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
    //
    fn apply_staking_fact(&mut self, staking_fact: &StakingFact) {
        match staking_fact {
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
                        registered_block_height: env::block_height(),
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
                assert!(
                    !self
                        .validator_id_to_delegator_id_set
                        .contains_key(validator_id),
                    "All delegators should be unbonded first, before unbonding validator '{}'.",
                    validator_id
                );
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
                        registered_block_height: env::block_height(),
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
            StakingFact::ValidatorIdInAppchainChanged {
                validator_id,
                validator_id_in_appchain,
            } => {
                let mut validator = self.validators.get(validator_id).unwrap();
                validator.validator_id_in_appchain = validator_id_in_appchain.to_string();
                self.validators.insert(validator_id, &validator);
            }
        }
    }
}

impl ValidatorSetViewer for ValidatorSet {
    //
    fn contains_validator(&self, validator_id: &AccountId) -> bool {
        self.validators.contains_key(validator_id)
    }
    //
    fn contains_delegator(&self, delegator_id: &AccountId, validator_id: &AccountId) -> bool {
        self.delegators
            .contains_key(&(delegator_id.clone(), validator_id.clone()))
    }
    //
    fn get_validator(&self, validator_id: &AccountId) -> Option<Validator> {
        self.validators.get(validator_id)
    }
    //
    fn get_validator_by_index(&self, index: &u64) -> Option<Validator> {
        match self.validator_id_set.as_vector().get(*index) {
            Some(validator_id) => self.validators.get(&validator_id),
            None => None,
        }
    }
    //
    fn get_delegator(
        &self,
        delegator_id: &AccountId,
        validator_id: &AccountId,
    ) -> Option<Delegator> {
        self.delegators
            .get(&(delegator_id.clone(), validator_id.clone()))
    }
    //
    fn get_delegator_by_index(&self, index: &u64, validator_id: &AccountId) -> Option<Delegator> {
        if let Some(delegator_id_set) = self.validator_id_to_delegator_id_set.get(validator_id) {
            if let Some(delegator_id) = delegator_id_set.as_vector().get(*index) {
                return self.get_delegator(&delegator_id, validator_id);
            }
        }
        None
    }
    //
    fn get_validator_ids(&self) -> Vec<AccountId> {
        self.validator_id_set.to_vec()
    }
    //
    fn get_validator_ids_of(&self, delegator_id: &AccountId) -> Vec<AccountId> {
        match self.delegator_id_to_validator_id_set.get(delegator_id) {
            Some(validator_id_set) => validator_id_set.to_vec(),
            None => Vec::new(),
        }
    }
    //
    fn get_delegator_ids_of(&self, validator_id: &AccountId) -> Vec<AccountId> {
        match self.validator_id_to_delegator_id_set.get(validator_id) {
            Some(delegator_id_set) => delegator_id_set.to_vec(),
            None => Vec::new(),
        }
    }
    //
    fn get_validator_count_of(&self, delegator_id: &AccountId) -> u64 {
        match self.delegator_id_to_validator_id_set.get(delegator_id) {
            Some(validator_id_set) => validator_id_set.len(),
            None => 0,
        }
    }
    //
    fn get_delegator_count_of(&self, validator_id: &AccountId) -> u64 {
        match self.validator_id_to_delegator_id_set.get(validator_id) {
            Some(delegator_id_set) => delegator_id_set.len(),
            None => 0,
        }
    }
    //
    fn era_number(&self) -> u64 {
        self.era_number
    }
    //
    fn total_stake(&self) -> u128 {
        self.total_stake
    }
    //
    fn validator_count(&self) -> u64 {
        self.validator_id_set.len()
    }
    //
    fn delegator_count(&self) -> u64 {
        self.get_validator_ids()
            .iter()
            .map(|v| self.get_delegator_count_of(v))
            .reduce(|s1, s2| s1 + s2)
            .unwrap_or(0)
    }
}

impl AppchainValidator {
    ///
    pub fn from_validator(validator: Validator, delegators_count: u64, is_unbonding: bool) -> Self {
        Self {
            validator_id: validator.validator_id,
            validator_id_in_appchain: validator.validator_id_in_appchain,
            deposit_amount: U128::from(validator.deposit_amount),
            total_stake: U128::from(validator.total_stake),
            delegators_count: U64::from(delegators_count),
            can_be_delegated_to: validator.can_be_delegated_to,
            is_unbonding,
        }
    }
}
