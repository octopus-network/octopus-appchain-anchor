use crate::*;

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
        let protocol_settings = self.protocol_settings.get().unwrap();
        let validator_set = self.next_validator_set.get().unwrap();
        assert!(
            validator_set.validator_id_set.len() >= protocol_settings.minimum_validator_count.0,
            "Not enough validators available."
        );
        let oct_token = self.oct_token.get().unwrap();
        assert!(
            validator_set.total_stake / OCT_DECIMALS_VALUE * oct_token.price_in_usd.0
                >= protocol_settings.minimum_total_stake_price_for_booting.0,
            "Not enough stake deposited in anchor."
        );
        self.appchain_state = AppchainState::Booting;
        self.internal_start_switching_era(0, 0);
        self.sync_state_to_registry();
    }
    //
    fn go_live(&mut self) {
        self.assert_owner();
        assert_eq!(
            self.appchain_state,
            AppchainState::Booting,
            "Appchain state is not 'booting'."
        );
        let wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
        assert!(
            !(wrapped_appchain_token.contract_account.trim().is_empty()
                || wrapped_appchain_token
                    .premined_beneficiary
                    .trim()
                    .is_empty()
                || wrapped_appchain_token.metadata.symbol.is_empty()
                || wrapped_appchain_token.metadata.name.is_empty()
                || wrapped_appchain_token.metadata.decimals == 0),
            "Missing settings of wrapped appchain token."
        );
        let appchain_settings = self.appchain_settings.get().unwrap();
        assert!(
            !(appchain_settings.boot_nodes.trim().is_empty()
                || appchain_settings.rpc_endpoint.trim().is_empty()
                || appchain_settings.era_reward.0 == 0),
            "Missing appchain settings."
        );
        self.appchain_state = AppchainState::Active;
        self.sync_state_to_registry();
    }
}
