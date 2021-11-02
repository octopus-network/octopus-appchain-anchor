use crate::*;

pub trait OwnerActions {
    ///
    fn remove_staking_history_before(&mut self, index: U64);
    ///
    fn remove_anchor_event_history_before(&mut self, index: U64);
}

#[near_bindgen]
impl OwnerActions for AppchainAnchor {
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
