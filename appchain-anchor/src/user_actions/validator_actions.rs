use crate::{interfaces::ValidatorActions, *};

use borsh::maybestd::collections::HashMap;

#[near_bindgen]
impl ValidatorActions for AppchainAnchor {
    //
    fn set_validator_id_in_appchain(&mut self, account_id_in_appchain: String) {
        let validator_id = env::predecessor_account_id();
        self.internal_change_account_id_in_appchain_of_validator(
            &validator_id,
            &account_id_in_appchain,
        );
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

impl AppchainAnchor {
    ///
    pub fn internal_change_account_id_in_appchain_of_validator(
        &mut self,
        validator_id: &AccountId,
        account_id_in_appchain: &String,
    ) {
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        self.assert_validator_id(validator_id, &next_validator_set);
        let validator_id_in_appchain = AccountIdInAppchain::new(
            Some(account_id_in_appchain.clone()),
            &self.appchain_template_type,
        );
        validator_id_in_appchain.assert_valid();
        //
        let mut validator_profiles = self.validator_profiles.get().unwrap();
        let mut validator_profile = validator_profiles.get(validator_id).unwrap();
        validator_profile.validator_id_in_appchain = validator_id_in_appchain.to_string();
        validator_profiles.insert(validator_profile);
        self.validator_profiles.set(&validator_profiles);
        //
        let staking_history = self.record_staking_fact(StakingFact::ValidatorIdInAppchainChanged {
            validator_id: validator_id.clone(),
            validator_id_in_appchain: account_id_in_appchain.clone(),
        });
        //
        next_validator_set.apply_staking_fact(&staking_history.staking_fact);
        self.next_validator_set.set(&next_validator_set);
        //
        self.sync_state_to_registry();
    }
}
