use crate::*;

use super::{Delegator, Validator, ValidatorSet};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NextValidatorSet {
    /// The validator set of next era
    validator_set: ValidatorSet,
    /// The unbonding validator ids in next era
    unbonding_validator_ids: Vec<AccountId>,
    /// The auto-unbonding validator ids in next era
    auto_unbonding_validator_ids: Vec<AccountId>,
}

impl NextValidatorSet {
    ///
    pub fn new(era_number: u64) -> Self {
        Self {
            validator_set: ValidatorSet::new(era_number),
            unbonding_validator_ids: Vec::<AccountId>::new(),
            auto_unbonding_validator_ids: Vec::<AccountId>::new(),
        }
    }
    ///
    pub fn from_validator_set(validator_set: ValidatorSet) -> Self {
        Self {
            validator_set,
            unbonding_validator_ids: Vec::<AccountId>::new(),
            auto_unbonding_validator_ids: Vec::<AccountId>::new(),
        }
    }
    ///
    pub fn clear(&mut self) {
        self.validator_set.clear();
        self.unbonding_validator_ids.clear();
        self.auto_unbonding_validator_ids.clear();
    }
    ///
    pub fn apply_staking_fact(&mut self, staking_fact: &StakingFact) {
        self.validator_set.apply_staking_fact(staking_fact);
    }
    ///
    pub fn add_unbonding_validator(&mut self, validator_id: &AccountId) {
        if !self.unbonding_validator_ids.contains(validator_id) {
            self.unbonding_validator_ids.push(validator_id.clone());
        }
    }
    ///
    pub fn add_auto_unbonding_validator(&mut self, validator_id: &AccountId) {
        if !self.auto_unbonding_validator_ids.contains(validator_id) {
            self.auto_unbonding_validator_ids.push(validator_id.clone());
        }
    }
    ///
    pub fn get_unbonding_validator_ids(&self) -> Vec<AccountId> {
        self.unbonding_validator_ids.to_vec()
    }
    ///
    pub fn get_auto_unbonding_validator_ids(&self) -> Vec<AccountId> {
        self.auto_unbonding_validator_ids.to_vec()
    }
    ///
    pub fn clear_unbonding_validator_ids(&mut self) {
        self.unbonding_validator_ids.clear();
    }
    ///
    pub fn clear_auto_unbonding_validator_ids(&mut self) {
        self.auto_unbonding_validator_ids.clear();
    }
}

impl ValidatorSetViewer for NextValidatorSet {
    //
    fn contains_validator(&self, validator_id: &AccountId) -> bool {
        self.validator_set.validator_id_set.contains(validator_id)
            && !self.unbonding_validator_ids.contains(validator_id)
            && !self.auto_unbonding_validator_ids.contains(validator_id)
    }
    //
    fn contains_delegator(&self, delegator_id: &AccountId, validator_id: &AccountId) -> bool {
        if self.contains_validator(validator_id) {
            if let Some(delegator_id_set) = self
                .validator_set
                .validator_id_to_delegator_id_set
                .get(validator_id)
            {
                return delegator_id_set.contains(delegator_id);
            }
        }
        false
    }
    //
    fn get_validator(&self, validator_id: &AccountId) -> Option<Validator> {
        self.validator_set.validators.get(validator_id)
    }
    //
    fn get_delegator(
        &self,
        delegator_id: &AccountId,
        validator_id: &AccountId,
    ) -> Option<Delegator> {
        self.validator_set
            .delegators
            .get(&(delegator_id.clone(), validator_id.clone()))
    }
    //
    fn get_validator_ids(&self) -> Vec<AccountId> {
        let validator_ids = self.validator_set.validator_id_set.to_vec();
        let mut results = Vec::<AccountId>::new();
        validator_ids.iter().for_each(|validator_id| {
            if !self.unbonding_validator_ids.contains(validator_id)
                && !self.auto_unbonding_validator_ids.contains(validator_id)
            {
                results.push(validator_id.clone());
            }
        });
        results
    }
    //
    fn get_validator_ids_of(&self, delegator_id: &AccountId) -> Vec<AccountId> {
        let mut results = Vec::<AccountId>::new();
        if let Some(validator_id_set) = self
            .validator_set
            .delegator_id_to_validator_id_set
            .get(delegator_id)
        {
            validator_id_set.to_vec().iter().for_each(|validator_id| {
                if !self.unbonding_validator_ids.contains(validator_id)
                    && !self.auto_unbonding_validator_ids.contains(validator_id)
                {
                    results.push(validator_id.clone());
                }
            });
        }
        results
    }
    //
    fn get_delegator_ids_of(&self, validator_id: &AccountId) -> Vec<AccountId> {
        if let Some(delegator_id_set) = self
            .validator_set
            .validator_id_to_delegator_id_set
            .get(validator_id)
        {
            delegator_id_set.to_vec()
        } else {
            Vec::new()
        }
    }
    //
    fn get_validator_list(&self) -> Vec<AppchainValidator> {
        let validator_ids = self.validator_set.validator_id_set.to_vec();
        let mut results = Vec::<AppchainValidator>::new();
        validator_ids.iter().for_each(|validator_id| {
            let validator = self.validator_set.validators.get(validator_id).unwrap();
            let mut delegators_count: u64 = 0;
            if let Some(delegator_id_set) = self
                .validator_set
                .validator_id_to_delegator_id_set
                .get(validator_id)
            {
                delegators_count = delegator_id_set.len();
            }
            results.push(AppchainValidator {
                validator_id: validator.validator_id,
                validator_id_in_appchain: validator.validator_id_in_appchain,
                deposit_amount: U128::from(validator.deposit_amount),
                total_stake: U128::from(validator.total_stake),
                delegators_count: U64::from(delegators_count),
                can_be_delegated_to: validator.can_be_delegated_to,
                is_unbonding: self.unbonding_validator_ids.contains(validator_id)
                    || self.auto_unbonding_validator_ids.contains(validator_id),
            });
        });
        results
    }
    //
    fn get_validator_count_of(&self, delegator_id: &AccountId) -> u64 {
        if let Some(validator_id_set) = self
            .validator_set
            .delegator_id_to_validator_id_set
            .get(&delegator_id)
        {
            validator_id_set.len()
        } else {
            0
        }
    }
    //
    fn total_stake(&self) -> u128 {
        self.get_validator_list()
            .iter()
            .map(|v| v.total_stake.0)
            .reduce(|s1, s2| s1 + s2)
            .unwrap_or(0)
    }
    //
    fn validator_count(&self) -> u64 {
        self.get_validator_list().len().try_into().unwrap()
    }
    //
    fn delegator_count(&self) -> u64 {
        let mut delegator_count: u64 = 0;
        self.get_validator_ids().iter().for_each(|validator_id| {
            if let Some(delegator_id_set) = self
                .validator_set
                .validator_id_to_delegator_id_set
                .get(validator_id)
            {
                delegator_count += delegator_id_set.len();
            }
        });
        delegator_count
    }
}
