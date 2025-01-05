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
    //
    pub fn len(&self) -> u64 {
        self.account_id_set.len()
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
            if env::used_gas() > Gas::ONE_TERA * T_GAS_CAP_FOR_MULTI_TXS_PROCESSING {
                return MultiTxsOperationProcessingResult::NeedMoreGas;
            }
        }
        MultiTxsOperationProcessingResult::Ok
    }
}
