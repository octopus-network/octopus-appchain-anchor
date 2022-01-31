use near_sdk::BlockHeight;

use crate::*;

use super::{Delegator, Validator, ValidatorSet};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorSetOfEra {
    /// The validator set of this era
    validator_set: ValidatorSet,
    /// The block height when the era starts.
    start_block_height: BlockHeight,
    /// The timestamp when the era starts.
    start_timestamp: Timestamp,
    /// The index of the latest staking history happened in the era of corresponding appchain.
    staking_history_index: u64,
    /// The set of validator id which will not be profited.
    unprofitable_validator_id_set: UnorderedSet<AccountId>,
    /// Total stake excluding all unprofitable validators' stake.
    valid_total_stake: Balance,
    /// The rewards of validators in this era
    validator_rewards: LookupMap<AccountId, Balance>,
    /// The rewards of delegators in this era
    delegator_rewards: LookupMap<(AccountId, AccountId), Balance>,
    /// The status of creation of this set
    processing_status: ValidatorSetProcessingStatus,
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
    pub fn start_timestamp(&self) -> Timestamp {
        self.start_timestamp
    }
    ///
    pub fn staking_history_index(&self) -> u64 {
        self.staking_history_index
    }
    ///
    pub fn get_validator_rewards_of(&self, validator_id: &AccountId) -> Option<u128> {
        self.validator_rewards.get(validator_id)
    }
    ///
    pub fn get_delegator_rewards_of(
        &self,
        delegator_id: &AccountId,
        validator_id: &AccountId,
    ) -> Option<u128> {
        self.delegator_rewards
            .get(&(delegator_id.clone(), validator_id.clone()))
    }
    ///
    pub fn unprofitable_validator_ids(&self) -> Vec<AccountId> {
        self.unprofitable_validator_id_set.to_vec()
    }
    ///
    pub fn processing_status(&self) -> ValidatorSetProcessingStatus {
        self.processing_status.clone()
    }
    ///
    pub fn valid_total_stake(&self) -> u128 {
        self.valid_total_stake
    }
    //
    fn all_staking_histories_are_applied(&self) -> bool {
        match self.processing_status {
            ValidatorSetProcessingStatus::ReadyForDistributingReward
            | ValidatorSetProcessingStatus::DistributingReward { .. }
            | ValidatorSetProcessingStatus::CheckingForAutoUnbondingValidator { .. }
            | ValidatorSetProcessingStatus::Completed => true,
            _ => false,
        }
    }
    ///
    pub fn set_staking_history_index(&mut self, index: u64) {
        self.staking_history_index = index;
    }
    ///
    pub fn set_total_stake(&mut self, total_stake: u128) {
        self.validator_set.total_stake = total_stake;
    }
    ///
    pub fn set_processing_status(&mut self, process_status: ValidatorSetProcessingStatus) {
        self.processing_status = process_status
    }
    ///
    pub fn set_unprofitable_validator_ids(&mut self, unprofitable_validator_ids: Vec<AccountId>) {
        unprofitable_validator_ids.iter().for_each(|v_id| {
            self.unprofitable_validator_id_set.insert(&v_id);
        });
    }
    ///
    pub fn insert_validator(&mut self, validator: &Validator) {
        self.validator_set
            .validator_id_set
            .insert(&validator.validator_id);
        self.validator_set
            .validators
            .insert(&validator.validator_id, &validator);
    }
    ///
    pub fn insert_delegator(&mut self, delegator: &Delegator) {
        let delegator_id = &delegator.delegator_id;
        let validator_id = &delegator.validator_id;
        self.validator_set
            .delegators
            .insert(&(delegator_id.clone(), validator_id.clone()), delegator);
        if !self
            .validator_set
            .validator_id_to_delegator_id_set
            .contains_key(validator_id)
        {
            self.validator_set.validator_id_to_delegator_id_set.insert(
                &delegator.validator_id,
                &UnorderedSet::new(
                    StorageKey::DelegatorIdsInMapOfVToDOfEra {
                        era_number: self.validator_set.era_number,
                        validator_id: validator_id.clone(),
                    }
                    .into_bytes(),
                ),
            );
        }
        let mut delegator_id_set = self
            .validator_set
            .validator_id_to_delegator_id_set
            .get(validator_id)
            .unwrap();
        delegator_id_set.insert(delegator_id);
        self.validator_set
            .validator_id_to_delegator_id_set
            .insert(validator_id, &delegator_id_set);
        //
        if !self
            .validator_set
            .delegator_id_to_validator_id_set
            .contains_key(delegator_id)
        {
            self.validator_set.delegator_id_to_validator_id_set.insert(
                delegator_id,
                &UnorderedSet::new(
                    StorageKey::ValidatorIdsInMapOfDToVOfEra {
                        era_number: self.validator_set.era_number,
                        delegator_id: delegator_id.clone(),
                    }
                    .into_bytes(),
                ),
            );
        }
        let mut validator_id_set = self
            .validator_set
            .delegator_id_to_validator_id_set
            .get(delegator_id)
            .unwrap();
        validator_id_set.insert(validator_id);
        self.validator_set
            .delegator_id_to_validator_id_set
            .insert(delegator_id, &validator_id_set);
    }
    ///
    pub fn set_validator_reward(&mut self, validator_id: &AccountId, amount: u128) {
        self.validator_rewards.insert(validator_id, &amount);
    }
    ///
    pub fn set_delegator_reward(
        &mut self,
        delegator_id: &AccountId,
        validator_id: &AccountId,
        amount: u128,
    ) {
        self.delegator_rewards
            .insert(&(delegator_id.clone(), validator_id.clone()), &amount);
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
            validator_list: self.get_validator_list(),
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
    //
    pub fn apply_staking_fact(&mut self, staking_fact: &StakingFact) {
        self.validator_set.apply_staking_fact(staking_fact);
    }
    ///
    pub fn get_validator_list(&self) -> Vec<AppchainValidator> {
        if !self.all_staking_histories_are_applied() {
            return Vec::new();
        }
        self.validator_set
            .get_validator_ids()
            .iter()
            .map(|validator_id| {
                AppchainValidator::from_validator(
                    self.validator_set.get_validator(validator_id).unwrap(),
                    self.validator_set.get_delegator_count_of(validator_id),
                    false,
                )
            })
            .collect()
    }
}

impl ValidatorSetViewer for ValidatorSetOfEra {
    //
    fn contains_validator(&self, validator_id: &AccountId) -> bool {
        if !self.all_staking_histories_are_applied() {
            return false;
        }
        self.validator_set.contains_validator(validator_id)
    }
    //
    fn contains_delegator(&self, delegator_id: &AccountId, validator_id: &AccountId) -> bool {
        if !self.all_staking_histories_are_applied() {
            return false;
        }
        self.validator_set
            .contains_delegator(delegator_id, validator_id)
    }
    //
    fn get_validator(&self, validator_id: &AccountId) -> Option<Validator> {
        if !self.all_staking_histories_are_applied() {
            return None;
        }
        self.validator_set.get_validator(validator_id)
    }
    //
    fn get_validator_by_index(&self, index: &u64) -> Option<Validator> {
        if !self.all_staking_histories_are_applied() {
            return None;
        }
        self.validator_set.get_validator_by_index(index)
    }
    //
    fn get_delegator(
        &self,
        delegator_id: &AccountId,
        validator_id: &AccountId,
    ) -> Option<Delegator> {
        if !self.all_staking_histories_are_applied() {
            return None;
        }
        self.validator_set.get_delegator(delegator_id, validator_id)
    }
    //
    fn get_delegator_by_index(&self, index: &u64, validator_id: &AccountId) -> Option<Delegator> {
        if !self.all_staking_histories_are_applied() {
            return None;
        }
        self.validator_set
            .get_delegator_by_index(index, validator_id)
    }
    //
    fn get_validator_ids(&self) -> Vec<AccountId> {
        if !self.all_staking_histories_are_applied() {
            return Vec::new();
        }
        self.validator_set.get_validator_ids()
    }
    //
    fn get_validator_ids_of(&self, delegator_id: &AccountId) -> Vec<AccountId> {
        if !self.all_staking_histories_are_applied() {
            return Vec::new();
        }
        self.validator_set.get_validator_ids_of(delegator_id)
    }
    //
    fn get_delegator_ids_of(&self, validator_id: &AccountId) -> Vec<AccountId> {
        if !self.all_staking_histories_are_applied() {
            return Vec::new();
        }
        self.validator_set.get_delegator_ids_of(validator_id)
    }
    //
    fn get_validator_count_of(&self, delegator_id: &AccountId) -> u64 {
        if !self.all_staking_histories_are_applied() {
            return 0;
        }
        self.validator_set.get_validator_count_of(delegator_id)
    }
    //
    fn get_delegator_count_of(&self, validator_id: &AccountId) -> u64 {
        if !self.all_staking_histories_are_applied() {
            return 0;
        }
        self.validator_set.get_delegator_count_of(validator_id)
    }
    //
    fn era_number(&self) -> u64 {
        self.validator_set.era_number()
    }
    //
    fn total_stake(&self) -> u128 {
        if !self.all_staking_histories_are_applied() {
            return 0;
        }
        self.validator_set.total_stake()
    }
    //
    fn validator_count(&self) -> u64 {
        if !self.all_staking_histories_are_applied() {
            return 0;
        }
        self.validator_set.validator_count()
    }
    //
    fn delegator_count(&self) -> u64 {
        if !self.all_staking_histories_are_applied() {
            return 0;
        }
        self.validator_set.delegator_count()
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
