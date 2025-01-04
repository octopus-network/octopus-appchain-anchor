use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorProfiles {
    /// The set of all validator id in NEAR protocol
    validator_id_set: UnorderedSet<AccountId>,
    /// The mapping for validator profiles, from account id in NEAR protocol to his/her profile
    profiles: LookupMap<AccountId, ValidatorProfile>,
    /// The mapping for validators' accounts, from account id in the appchain to
    /// account id in NEAR protocol.
    map_by_id_in_appchain: LookupMap<String, AccountId>,
}

impl ValidatorProfiles {
    ///
    pub fn new() -> Self {
        Self {
            validator_id_set: UnorderedSet::new(StorageKey::ValidatorProfilesIdSet.into_bytes()),
            profiles: LookupMap::new(StorageKey::ValidatorProfilesMap.into_bytes()),
            map_by_id_in_appchain: LookupMap::new(StorageKey::ValidatorProfilesIdMap.into_bytes()),
        }
    }
    ///
    pub fn get(&self, validator_id: &AccountId) -> Option<ValidatorProfile> {
        self.profiles.get(validator_id)
    }
    ///
    pub fn get_by_id_in_appchain(
        &self,
        validator_id_in_appchain: &String,
    ) -> Option<ValidatorProfile> {
        match self.map_by_id_in_appchain.get(validator_id_in_appchain) {
            Some(validator_id) => self.profiles.get(&validator_id),
            None => None,
        }
    }
    //
    pub fn clear(&mut self) {
        for validator_id in self.validator_id_set.iter() {
            if let Some(profile) = self.profiles.get(&validator_id) {
                self.map_by_id_in_appchain
                    .remove(&profile.validator_id_in_appchain);
            }
            self.profiles.remove(&validator_id);
        }
    }
}
