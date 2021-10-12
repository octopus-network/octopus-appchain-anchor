use crate::*;

impl AppchainSettings {
    ///
    pub fn all_fields_are_set(&self) -> bool {
        !(self.chain_spec.is_empty()
            || self.raw_chain_spec.is_empty()
            || self.boot_nodes.is_empty()
            || self.rpc_endpoint.is_empty()
            || self.era_reward.0 == 0)
    }
}

pub trait AppchainLifecycleManager {
    /// Verify and change the state of corresponding appchain to `booting`.
    fn go_booting(&mut self);
    /// Verify and change the state of corresponding appchain to `active`.
    fn go_live(&mut self);
}

#[near_bindgen]
impl AppchainLifecycleManager for AppchainAnchor {
    //
    fn go_booting(&mut self) {
        self.assert_owner();
        assert_eq!(
            self.appchain_state,
            AppchainState::Staging,
            "Appchain state is not 'staging'."
        );
        let appchain_settings = self.appchain_settings.get().unwrap();
        assert!(
            appchain_settings.all_fields_are_set(),
            "Missing appchain settings."
        );
        let protocol_settings = self.protocol_settings.get().unwrap();
        let validator_set = self.next_validator_set.get().unwrap();
        assert!(
            validator_set.validator_ids.len() >= protocol_settings.minimum_validator_count.0,
            "Not enough validators available."
        );
        let oct_token = self.oct_token.get().unwrap();
        assert!(
            validator_set.total_stake * oct_token.price_in_usd.0
                >= protocol_settings.minimum_total_stake_price_for_booting.0,
            "Not enough stake deposited in anchor."
        );
        self.appchain_state = AppchainState::Booting;
        self.start_switching_era(0);
    }
    //
    fn go_live(&mut self) {
        self.assert_owner();
        assert_eq!(
            self.appchain_state,
            AppchainState::Booting,
            "Appchain state is not 'booting'."
        );
        self.appchain_state = AppchainState::Active;
    }
}
