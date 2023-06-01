use near_sdk::collections::LookupSet;

use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct RewardDistributionRecords {
    ///
    era_number_set: UnorderedSet<u64>,
    /// The map from `era_number` to `Vec<appchain_message_nonce>`.
    era_number_to_nonces_map: LookupMap<u64, Vec<u32>>,
    /// The record set for reward distribution
    /// The element in set is `(appchain_message_nonce, era_number, account_id_of_delegator, account_id_of_validator)`
    record_set: LookupSet<(u32, u64, String, AccountId)>,
}

impl RewardDistributionRecords {
    ///
    pub fn new() -> Self {
        Self {
            era_number_set: UnorderedSet::new(
                StorageKey::RewardDistributionEraNumberSet.into_bytes(),
            ),
            era_number_to_nonces_map: LookupMap::new(
                StorageKey::RewardDistributionEraNumberToNoncesMap.into_bytes(),
            ),
            record_set: LookupSet::new(StorageKey::RewardDistributionRecordSet.into_bytes()),
        }
    }
    ///
    pub fn get_message_nonces_of_era(&self, era_number: &u64) -> Vec<u32> {
        match self.era_number_to_nonces_map.get(era_number) {
            Some(nonces) => nonces.clone(),
            None => Vec::new(),
        }
    }
    ///
    pub fn contains_record(
        &self,
        appchain_message_nonce: u32,
        era_number: u64,
        delegator_id: &Option<AccountId>,
        validator_id: &AccountId,
    ) -> bool {
        self.record_set.contains(&(
            appchain_message_nonce,
            era_number,
            match delegator_id {
                Some(delegator_id) => delegator_id.to_string(),
                None => String::new(),
            },
            validator_id.clone(),
        ))
    }
    ///
    pub fn insert(
        &mut self,
        appchain_message_nonce: u32,
        era_number: u64,
        delegator_id: &Option<AccountId>,
        validator_id: &AccountId,
    ) {
        self.era_number_set.insert(&era_number);
        let mut nonces = match self.era_number_to_nonces_map.get(&era_number) {
            Some(nonces) => nonces,
            None => Vec::new(),
        };
        if !nonces.contains(&appchain_message_nonce) {
            nonces.push(appchain_message_nonce);
        }
        self.era_number_to_nonces_map.insert(&era_number, &nonces);
        self.record_set.insert(&(
            appchain_message_nonce,
            era_number,
            match delegator_id {
                Some(delegator_id) => delegator_id.to_string(),
                None => String::new(),
            },
            validator_id.clone(),
        ));
    }
    ///
    pub fn clear<V: ValidatorSetViewer>(
        &mut self,
        validator_set: &V,
        era_number: &u64,
        nonce_index_start: u32,
        validator_index_start: u64,
        delegator_index_start: u64,
        max_gas: Gas,
    ) -> MultiTxsOperationProcessingResult {
        if self.era_number_set.contains(era_number) {
            if let Some(nonce_array) = self.era_number_to_nonces_map.get(era_number) {
                let mut nonce_index: usize = 0;
                let mut validator_index = 0;
                let mut delegator_index = 0;
                while nonce_index < nonce_array.len() {
                    if nonce_index < nonce_index_start as usize {
                        nonce_index += 1;
                        continue;
                    }
                    if let Some(nonce) = nonce_array.get(nonce_index) {
                        let validator_ids = validator_set.get_validator_ids();
                        for validator_id in validator_ids {
                            if nonce_index == nonce_index_start as usize
                                && validator_index < validator_index_start
                            {
                                validator_index += 1;
                                continue;
                            }
                            for delegator_id in validator_set.get_delegator_ids_of(&validator_id) {
                                if nonce_index == nonce_index_start as usize
                                    && validator_index == validator_index_start
                                    && delegator_index < delegator_index_start
                                {
                                    delegator_index += 1;
                                    continue;
                                }
                                self.record_set.remove(&(
                                    *nonce,
                                    *era_number,
                                    delegator_id.to_string(),
                                    validator_id.clone(),
                                ));
                                delegator_index += 1;
                                if env::used_gas() >= max_gas {
                                    RemovingValidatorSetSteps::ClearingRewardDistributionRecords {
                                        appchain_message_nonce_index: nonce_index as u32,
                                        validator_index,
                                        delegator_index,
                                    }
                                    .save();
                                    return MultiTxsOperationProcessingResult::NeedMoreGas;
                                }
                            }
                            self.record_set.remove(&(
                                *nonce,
                                *era_number,
                                String::new(),
                                validator_id.clone(),
                            ));
                            validator_index += 1;
                            delegator_index = 0;
                            if env::used_gas() >= max_gas {
                                RemovingValidatorSetSteps::ClearingRewardDistributionRecords {
                                    appchain_message_nonce_index: nonce_index as u32,
                                    validator_index,
                                    delegator_index,
                                }
                                .save();
                                return MultiTxsOperationProcessingResult::NeedMoreGas;
                            }
                        }
                    }
                    nonce_index += 1;
                    validator_index = 0;
                    delegator_index = 0;
                    if env::used_gas() >= max_gas {
                        RemovingValidatorSetSteps::ClearingRewardDistributionRecords {
                            appchain_message_nonce_index: nonce_index as u32,
                            validator_index,
                            delegator_index,
                        }
                        .save();
                        return MultiTxsOperationProcessingResult::NeedMoreGas;
                    }
                }
                self.era_number_to_nonces_map.remove(era_number);
            }
            self.era_number_set.remove(era_number);
        }
        MultiTxsOperationProcessingResult::Ok
    }
    /// This function is for fixing wrong history data
    pub fn remove_duplicated_message_nonces(&mut self, era_number: u64) {
        if self.era_number_set.contains(&era_number) {
            if let Some(nonce_array) = self.era_number_to_nonces_map.get(&era_number) {
                let mut nonces = Vec::<u32>::new();
                nonce_array.iter().for_each(|nonce| {
                    if !nonces.contains(nonce) {
                        nonces.push(*nonce);
                    }
                });
                self.era_number_to_nonces_map.insert(&era_number, &nonces);
            }
        }
    }
}
