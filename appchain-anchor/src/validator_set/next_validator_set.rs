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
    pub fn validator_set(&self) -> &ValidatorSet {
        &self.validator_set
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
    ///
    pub fn get_validator_list(&self) -> Vec<AppchainValidator> {
        self.validator_set
            .get_validator_ids()
            .iter()
            .map(|validator_id| {
                AppchainValidator::from_validator(
                    self.validator_set.get_validator(validator_id).unwrap(),
                    self.validator_set.get_delegator_count_of(validator_id),
                    self.unbonding_validator_ids.contains(validator_id)
                        || self.auto_unbonding_validator_ids.contains(validator_id),
                )
            })
            .collect()
    }
}

impl ValidatorSetViewer for NextValidatorSet {
    //
    fn contains_validator(&self, validator_id: &AccountId) -> bool {
        self.validator_set.contains_validator(validator_id)
            && !self.unbonding_validator_ids.contains(validator_id)
            && !self.auto_unbonding_validator_ids.contains(validator_id)
    }
    //
    fn contains_delegator(&self, delegator_id: &AccountId, validator_id: &AccountId) -> bool {
        if self.contains_validator(validator_id) {
            self.validator_set
                .contains_delegator(delegator_id, validator_id)
        } else {
            false
        }
    }
    //
    fn get_validator(&self, validator_id: &AccountId) -> Option<Validator> {
        if self.contains_validator(validator_id) {
            self.validator_set.get_validator(validator_id)
        } else {
            None
        }
    }
    //
    fn get_validator_by_index(&self, index: &u64) -> Option<Validator> {
        assert!(
            self.auto_unbonding_validator_ids.len() == 0 && self.unbonding_validator_ids.len() == 0,
            "Can not use method 'get_validator_by_index' while next validator set has not been finalized."
        );
        self.validator_set.get_validator_by_index(index)
    }
    //
    fn get_delegator(
        &self,
        delegator_id: &AccountId,
        validator_id: &AccountId,
    ) -> Option<Delegator> {
        if self.contains_validator(validator_id) {
            self.validator_set.get_delegator(delegator_id, validator_id)
        } else {
            None
        }
    }
    //
    fn get_delegator_by_index(&self, index: &u64, validator_id: &AccountId) -> Option<Delegator> {
        self.validator_set
            .get_delegator_by_index(index, validator_id)
    }
    //
    fn get_validator_ids(&self) -> Vec<AccountId> {
        self.validator_set
            .get_validator_ids()
            .iter()
            .filter(|validator_id| {
                !self.unbonding_validator_ids.contains(validator_id)
                    && !self.auto_unbonding_validator_ids.contains(validator_id)
            })
            .cloned()
            .collect()
    }
    //
    fn get_validator_ids_of(&self, delegator_id: &AccountId) -> Vec<AccountId> {
        self.validator_set
            .get_validator_ids_of(delegator_id)
            .iter()
            .filter(|validator_id| {
                !self.unbonding_validator_ids.contains(validator_id)
                    && !self.auto_unbonding_validator_ids.contains(validator_id)
            })
            .cloned()
            .collect()
    }
    //
    fn get_delegator_ids_of(&self, validator_id: &AccountId) -> Vec<AccountId> {
        if self.contains_validator(validator_id) {
            self.validator_set.get_delegator_ids_of(validator_id)
        } else {
            Vec::new()
        }
    }
    //
    fn get_validator_count_of(&self, delegator_id: &AccountId) -> u64 {
        self.get_validator_ids_of(delegator_id)
            .len()
            .try_into()
            .unwrap_or(0)
    }
    //
    fn get_delegator_count_of(&self, validator_id: &AccountId) -> u64 {
        if self.contains_validator(validator_id) {
            self.validator_set.get_delegator_count_of(validator_id)
        } else {
            0
        }
    }
    //
    fn era_number(&self) -> u64 {
        self.validator_set.era_number()
    }
    //
    fn total_stake(&self) -> u128 {
        let mut total_stake = self.validator_set.total_stake();
        self.auto_unbonding_validator_ids.iter().for_each(|id| {
            total_stake -= self.get_validator(id).map(|v| v.total_stake).unwrap_or(0);
        });
        self.unbonding_validator_ids.iter().for_each(|id| {
            total_stake -= self.get_validator(id).map(|v| v.total_stake).unwrap_or(0);
        });
        total_stake
    }
    //
    fn validator_count(&self) -> u64 {
        self.get_validator_ids().len().try_into().unwrap()
    }
    //
    fn delegator_count(&self) -> u64 {
        self.get_validator_ids()
            .iter()
            .map(|id| self.get_delegator_count_of(id))
            .reduce(|s1, s2| s1 + s2)
            .unwrap_or(0)
    }
}
