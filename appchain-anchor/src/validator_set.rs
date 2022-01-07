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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NextValidatorSet {
    /// The validator set of next era
    validator_set: ValidatorSet,
    /// The unbonding validator ids in next era
    unbonding_validator_ids: Vec<AccountId>,
    /// The auto-unbonding validator ids in next era
    auto_unbonding_validator_ids: Vec<AccountId>,
}

pub trait ValidatorSetViewer {
    ///
    fn contains_validator(&self, validator_id: &AccountId) -> bool;
    ///
    fn contains_delegator(&self, delegator_id: &AccountId, validator_id: &AccountId) -> bool;
    ///
    fn get_validator(&self, validator_id: &AccountId) -> Option<Validator>;
    ///
    fn get_delegator(
        &self,
        delegator_id: &AccountId,
        validator_id: &AccountId,
    ) -> Option<Delegator>;
    ///
    fn get_validator_ids(&self) -> Vec<AccountId>;
    ///
    fn get_validator_ids_of(&self, delegator_id: &AccountId) -> Vec<AccountId>;
    ///
    fn get_delegator_ids_of(&self, validator_id: &AccountId) -> Vec<AccountId>;
    ///
    fn get_validator_list(&self) -> Vec<AppchainValidator>;
    ///
    fn get_validator_count_of(&self, delegator_id: &AccountId) -> u64;
    ///
    fn total_stake(&self) -> u128;
    ///
    fn validator_count(&self) -> u64;
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
        self.validator_set.total_stake
    }
    //
    fn validator_count(&self) -> u64 {
        self.get_validator_list().len().try_into().unwrap()
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
    pub fn era_number(&self) -> u64 {
        self.validator_set.era_number
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
}

impl ValidatorSetViewer for ValidatorSetOfEra {
    //
    fn contains_validator(&self, validator_id: &AccountId) -> bool {
        if !self.all_staking_histories_are_applied() {
            return false;
        }
        self.validator_set.validator_id_set.contains(validator_id)
    }
    //
    fn contains_delegator(&self, delegator_id: &AccountId, validator_id: &AccountId) -> bool {
        if !self.all_staking_histories_are_applied() {
            return false;
        }
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
        if !self.all_staking_histories_are_applied() {
            return None;
        }
        self.validator_set.validators.get(validator_id)
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
        self.validator_set
            .delegators
            .get(&(delegator_id.clone(), validator_id.clone()))
    }
    //
    fn get_validator_ids(&self) -> Vec<AccountId> {
        if !self.all_staking_histories_are_applied() {
            return Vec::new();
        }
        self.validator_set.validator_id_set.to_vec()
    }
    //
    fn get_validator_ids_of(&self, delegator_id: &AccountId) -> Vec<AccountId> {
        if !self.all_staking_histories_are_applied() {
            return Vec::new();
        }
        if let Some(validator_id_set) = self
            .validator_set
            .delegator_id_to_validator_id_set
            .get(delegator_id)
        {
            validator_id_set.to_vec()
        } else {
            Vec::new()
        }
    }
    //
    fn get_delegator_ids_of(&self, validator_id: &AccountId) -> Vec<AccountId> {
        if !self.all_staking_histories_are_applied() {
            return Vec::new();
        }
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
        if !self.all_staking_histories_are_applied() {
            return Vec::new();
        }
        let validator_ids = self.validator_set.validator_id_set.to_vec();
        validator_ids
            .iter()
            .map(|validator_id| {
                let validator = self.validator_set.validators.get(validator_id).unwrap();
                let mut delegators_count: u64 = 0;
                if let Some(delegator_id_set) = self
                    .validator_set
                    .validator_id_to_delegator_id_set
                    .get(validator_id)
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
                    is_unbonding: false,
                };
            })
            .collect()
    }
    //
    fn get_validator_count_of(&self, delegator_id: &AccountId) -> u64 {
        if !self.all_staking_histories_are_applied() {
            return 0;
        }
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
        if !self.all_staking_histories_are_applied() {
            return 0;
        }
        self.validator_set.total_stake
    }
    //
    fn validator_count(&self) -> u64 {
        if !self.all_staking_histories_are_applied() {
            return 0;
        }
        self.validator_set.validator_id_set.len()
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
