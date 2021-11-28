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
    record_set: LookupSet<(u32, u64, AccountId, AccountId)>,
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
    pub fn contains_record(
        &self,
        appchain_message_nonce: u32,
        era_number: u64,
        delegator_id: &AccountId,
        validator_id: &AccountId,
    ) -> bool {
        self.record_set.contains(&(
            appchain_message_nonce,
            era_number,
            delegator_id.clone(),
            validator_id.clone(),
        ))
    }
    ///
    pub fn insert(
        &mut self,
        appchain_message_nonce: u32,
        era_number: u64,
        delegator_id: &AccountId,
        validator_id: &AccountId,
    ) {
        self.era_number_set.insert(&era_number);
        let mut nonces = match self.era_number_to_nonces_map.get(&era_number) {
            Some(nonces) => nonces,
            None => Vec::new(),
        };
        nonces.push(appchain_message_nonce);
        self.era_number_to_nonces_map.insert(&era_number, &nonces);
        self.record_set.insert(&(
            appchain_message_nonce,
            era_number,
            delegator_id.clone(),
            validator_id.clone(),
        ));
    }
}
