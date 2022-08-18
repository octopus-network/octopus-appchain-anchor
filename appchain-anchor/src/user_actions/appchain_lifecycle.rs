use crate::{
    interfaces::AppchainLifecycleManager,
    permissionless_actions::AppchainMessagesProcessingContext, *,
};

#[near_bindgen]
impl AppchainLifecycleManager for AppchainAnchor {
    //
    fn go_booting(&mut self) {
        self.assert_owner();
        assert_eq!(
            self.appchain_state,
            AppchainState::Staging,
            "Appchain state must be 'staging'."
        );
        let protocol_settings = self.protocol_settings.get().unwrap();
        let next_validator_set = self.next_validator_set.get().unwrap();
        assert!(
            next_validator_set.validator_count() >= protocol_settings.minimum_validator_count.0,
            "Not enough validators available."
        );
        let oct_token = self.oct_token.get().unwrap();
        assert!(
            next_validator_set.total_stake() / OCT_DECIMALS_VALUE * oct_token.price_in_usd.0
                >= protocol_settings.minimum_total_stake_price_for_booting.0,
            "Not enough stake deposited in anchor."
        );
        self.appchain_state = AppchainState::Booting;
        let mut processing_context = AppchainMessagesProcessingContext::new(
            self.permissionless_actions_status.get().unwrap(),
        );
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        self.internal_start_switching_era(&mut processing_context, &mut validator_set_histories, 0);
        loop {
            match self.complete_switching_era(
                &mut processing_context,
                &mut validator_set_histories,
                0,
            ) {
                MultiTxsOperationProcessingResult::Ok => break,
                MultiTxsOperationProcessingResult::NeedMoreGas => (),
                MultiTxsOperationProcessingResult::Error(message) => {
                    panic!("Failed to generate validator set 0: '{}'", &message)
                }
            }
        }
        self.validator_set_histories.set(&validator_set_histories);
        self.sync_state_to_registry();
    }
    //
    fn go_live(&mut self) {
        self.assert_owner();
        assert_eq!(
            self.appchain_state,
            AppchainState::Booting,
            "Appchain state must be 'booting'."
        );
        let wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
        assert!(
            !(wrapped_appchain_token.contract_account.is_none()
                || wrapped_appchain_token.premined_beneficiary.is_none()
                || wrapped_appchain_token.metadata.symbol.is_empty()
                || wrapped_appchain_token.metadata.name.is_empty()
                || wrapped_appchain_token.metadata.decimals == 0),
            "Missing settings of wrapped appchain token."
        );
        let appchain_settings = self.appchain_settings.get().unwrap();
        assert!(
            !(appchain_settings.rpc_endpoint.trim().is_empty()
                || appchain_settings.era_reward.0 == 0),
            "Missing appchain settings."
        );
        self.assert_light_client_initialized();
        self.appchain_state = AppchainState::Active;
        self.sync_state_to_registry();
    }
    //
    fn initialize_beefy_light_client(&mut self, initial_public_keys: Vec<String>) {
        self.assert_owner();
        assert_eq!(
            self.appchain_state,
            AppchainState::Booting,
            "Appchain state must be 'booting'."
        );
        assert!(
            self.beefy_light_client_state.is_none(),
            "Beefy light client has already been initialized."
        );
        self.beefy_light_client_state
            .set(&beefy_light_client::new(initial_public_keys));
    }
}
