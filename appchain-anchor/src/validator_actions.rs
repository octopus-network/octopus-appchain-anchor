use crate::*;

use borsh::maybestd::collections::HashMap;

pub trait ValidatorActions {
    ///
    fn set_validator_id_in_appchain(&mut self, account_id_in_appchain: String);
    ///
    fn set_validator_profile(&mut self, profile: HashMap<String, String>);
}

#[near_bindgen]
impl ValidatorActions for AppchainAnchor {
    //
    fn set_validator_id_in_appchain(&mut self, account_id_in_appchain: String) {
        let validator_id = env::predecessor_account_id();
        let next_validator_set = self.next_validator_set.get().unwrap();
        self.assert_validator_id(&validator_id, &next_validator_set);
        let validator_id_in_appchain = AccountIdInAppchain::new(Some(account_id_in_appchain));
        validator_id_in_appchain.assert_valid();
        let mut validator_profiles = self.validator_profiles.get().unwrap();
        let mut validator_profile = validator_profiles.get(&validator_id).unwrap();
        validator_profile.validator_id_in_appchain = validator_id_in_appchain.to_string();
        validator_profiles.insert(validator_profile);
        self.validator_profiles.set(&validator_profiles);
    }
    //
    fn set_validator_profile(&mut self, profile: HashMap<String, String>) {
        let validator_id = env::predecessor_account_id();
        let next_validator_set = self.next_validator_set.get().unwrap();
        self.assert_validator_id(&validator_id, &next_validator_set);
        let mut validator_profiles = self.validator_profiles.get().unwrap();
        let mut validator_profile = validator_profiles.get(&validator_id).unwrap();
        validator_profile.profile = profile;
        validator_profiles.insert(validator_profile);
        self.validator_profiles.set(&validator_profiles);
    }
}
