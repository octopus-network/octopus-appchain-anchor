use crate::{interfaces::OwnerActions, *};

#[near_bindgen]
impl OwnerActions for AppchainAnchor {
    //
    fn remove_validator_set_before(&mut self, era_number: U64) {
        self.assert_owner();
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        validator_set_histories.remove_before(&era_number.0);
        self.validator_set_histories.set(&validator_set_histories);
    }
    //
    fn remove_staking_history_before(&mut self, index: U64) {
        self.assert_owner();
        let mut staking_histories = self.staking_histories.get().unwrap();
        staking_histories.remove_before(&index.0);
        self.staking_histories.set(&staking_histories);
    }
    //
    fn remove_appchain_notification_history_before(&mut self, index: U64) {
        self.assert_owner();
        let mut appchain_notification_histories =
            self.appchain_notification_histories.get().unwrap();
        appchain_notification_histories.remove_before(&index.0);
        self.appchain_notification_histories
            .set(&appchain_notification_histories);
    }
}
