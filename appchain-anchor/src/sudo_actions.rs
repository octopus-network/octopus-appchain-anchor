use crate::*;

pub trait SudoActions {
    /// Apply a certain `AppchainMessage`
    fn apply_appchain_message(&mut self, appchain_message: AppchainMessage);
}

#[near_bindgen]
impl SudoActions for AppchainAnchor {
    //
    fn apply_appchain_message(&mut self, appchain_message: AppchainMessage) {
        self.assert_owner();
        self.internal_apply_appchain_message(appchain_message);
    }
}
