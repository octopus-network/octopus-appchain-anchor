use crate::*;

pub trait SudoActions {
    /// Apply a certain `AppchainMessage`
    fn apply_appchain_message_by_owner(&mut self, appchain_message: AppchainMessage);
}

#[near_bindgen]
impl SudoActions for AppchainAnchor {
    //
    fn apply_appchain_message_by_owner(&mut self, appchain_message: AppchainMessage) {
        self.assert_owner();
        self.apply_appchain_message(appchain_message);
    }
}
