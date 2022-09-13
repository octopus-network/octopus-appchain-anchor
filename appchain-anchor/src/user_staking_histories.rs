use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserStakingHistories {
    ///
    account_id_set: UnorderedSet<AccountId>,
    /// The map from `account_id` to array of the index of staking histories that
    /// related to the account.
    staking_histories_map: LookupMap<AccountId, Vec<u64>>,
}

impl UserStakingHistories {
    ///
    pub fn new() -> Self {
        Self {
            account_id_set: UnorderedSet::new(StorageKey::UserStakingHistoriesIdSet.into_bytes()),
            staking_histories_map: LookupMap::new(StorageKey::UserStakingHistoriesMap.into_bytes()),
        }
    }
    ///
    pub fn add_staking_history(&mut self, staking_history: &StakingHistory) {
        let account_id = match &staking_history.staking_fact {
            StakingFact::ValidatorRegistered { validator_id, .. }
            | StakingFact::StakeIncreased { validator_id, .. }
            | StakingFact::StakeDecreased { validator_id, .. }
            | StakingFact::ValidatorUnbonded { validator_id, .. }
            | StakingFact::ValidatorAutoUnbonded { validator_id, .. }
            | StakingFact::ValidatorDelegationEnabled { validator_id }
            | StakingFact::ValidatorDelegationDisabled { validator_id }
            | StakingFact::ValidatorIdInAppchainChanged { validator_id, .. } => validator_id,
            StakingFact::DelegatorRegistered { delegator_id, .. }
            | StakingFact::DelegationIncreased { delegator_id, .. }
            | StakingFact::DelegationDecreased { delegator_id, .. }
            | StakingFact::DelegatorUnbonded { delegator_id, .. }
            | StakingFact::DelegatorAutoUnbonded { delegator_id, .. } => delegator_id,
        };
        self.account_id_set.insert(account_id);
        let mut staking_histories_indexes = match self.staking_histories_map.get(account_id) {
            Some(indexes) => indexes,
            None => Vec::new(),
        };
        if !staking_histories_indexes.contains(&staking_history.index.0) {
            staking_histories_indexes.push(staking_history.index.0);
            self.staking_histories_map
                .insert(account_id, &staking_histories_indexes);
        }
    }
    ///
    pub fn get_staking_history_indexes_of(&self, account_id: &AccountId) -> Vec<u64> {
        match self.staking_histories_map.get(account_id) {
            Some(indexes) => indexes,
            None => Vec::new(),
        }
    }
    ///
    pub fn clear(&mut self) -> MultiTxsOperationProcessingResult {
        for account_id in self.account_id_set.to_vec() {
            self.staking_histories_map.remove(&account_id);
            self.account_id_set.remove(&account_id);
            if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
                return MultiTxsOperationProcessingResult::NeedMoreGas;
            }
        }
        MultiTxsOperationProcessingResult::Ok
    }
}
