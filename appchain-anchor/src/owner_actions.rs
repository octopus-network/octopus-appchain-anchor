use crate::*;

pub trait OwnerActions {
    ///
    fn remove_validator_set_before(&mut self, era_number: U64);
    ///
    fn remove_staking_history_before(&mut self, index: U64);
    ///
    fn remove_anchor_event_history_before(&mut self, index: U64);
}

#[near_bindgen]
impl OwnerActions for AppchainAnchor {
    //
    fn remove_validator_set_before(&mut self, era_number: U64) {
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        validator_set_histories.remove_before(&era_number.0);
        self.validator_set_histories.set(&validator_set_histories);
    }
    //
    fn remove_staking_history_before(&mut self, index: U64) {
        let mut staking_histories = self.staking_histories.get().unwrap();
        staking_histories.remove_before(&index.0);
        self.staking_histories.set(&staking_histories);
    }
    //
    fn remove_anchor_event_history_before(&mut self, index: U64) {
        let mut anchor_event_histories = self.anchor_event_histories.get().unwrap();
        anchor_event_histories.remove_before(&index.0);
        self.anchor_event_histories.set(&anchor_event_histories);
    }
}
