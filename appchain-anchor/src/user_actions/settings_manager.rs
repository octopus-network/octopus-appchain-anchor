use crate::{
    interfaces::{AnchorSettingsManager, AppchainSettingsManager, ProtocolSettingsManager},
    *,
};
use core::convert::From;

impl Default for ProtocolSettings {
    fn default() -> Self {
        Self {
            minimum_validator_deposit: U128::from(5_000 * OCT_DECIMALS_VALUE),
            minimum_validator_deposit_changing_amount: U128::from(1000 * OCT_DECIMALS_VALUE),
            maximum_validator_stake_percent: 25,
            minimum_delegator_deposit: U128::from(200 * OCT_DECIMALS_VALUE),
            minimum_delegator_deposit_changing_amount: U128::from(100 * OCT_DECIMALS_VALUE),
            minimum_total_stake_price_for_booting: U128::from(100_000 * USD_DECIMALS_VALUE),
            maximum_market_value_percent_of_near_fungible_tokens: 33,
            maximum_market_value_percent_of_wrapped_appchain_token: 67,
            minimum_validator_count: U64::from(4),
            maximum_validator_count: U64::from(60),
            maximum_validators_per_delegator: U64::from(16),
            unlock_period_of_validator_deposit: U64::from(21),
            unlock_period_of_delegator_deposit: U64::from(21),
            maximum_era_count_of_unwithdrawn_reward: U64::from(84),
            maximum_era_count_of_valid_appchain_message: U64::from(7),
            validator_commission_percent: 20,
            maximum_allowed_unprofitable_era_count: 3,
        }
    }
}

impl Default for AnchorSettings {
    fn default() -> Self {
        Self {
            token_price_maintainer_account: None,
            relayer_account: None,
            beefy_light_client_witness_mode: false,
        }
    }
}

impl Default for AppchainSettings {
    fn default() -> Self {
        Self {
            rpc_endpoint: String::new(),
            subql_endpoint: String::new(),
            era_reward: U128::from(0),
            bonus_for_new_validator: U128::from(0),
        }
    }
}

#[near_bindgen]
impl ProtocolSettingsManager for AppchainAnchor {
    //
    fn change_minimum_validator_deposit(&mut self, value: U128) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value.0 != protocol_settings.minimum_validator_deposit.0,
            "The value is not changed."
        );
        assert!(
            value.0
                > protocol_settings
                    .minimum_validator_deposit_changing_amount
                    .0,
            "The value should be greater than `minimum_validator_deposit_changing_amount`."
        );
        protocol_settings.minimum_validator_deposit = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_minimum_validator_deposit_changing_amount(&mut self, value: U128) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value.0
                != protocol_settings
                    .minimum_validator_deposit_changing_amount
                    .0,
            "The value is not changed."
        );
        assert!(
            value.0 < protocol_settings.minimum_validator_deposit.0,
            "The value should be less than `minimum_validator_deposit`."
        );
        protocol_settings.minimum_validator_deposit_changing_amount = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_maximum_validator_stake_percent(&mut self, value: u16) {
        self.assert_owner();
        assert!(value < 100, "Invalid percent value.");
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value != protocol_settings.maximum_validator_stake_percent,
            "The value is not changed."
        );
        protocol_settings.maximum_validator_stake_percent = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_minimum_delegator_deposit(&mut self, value: U128) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value.0 != protocol_settings.minimum_delegator_deposit.0,
            "The value is not changed."
        );
        assert!(
            value.0
                > protocol_settings
                    .minimum_delegator_deposit_changing_amount
                    .0,
            "The value should be greater than `minimum_delegator_deposit_changing_amount`."
        );
        protocol_settings.minimum_delegator_deposit = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_minimum_delegator_deposit_changing_amount(&mut self, value: U128) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value.0
                != protocol_settings
                    .minimum_delegator_deposit_changing_amount
                    .0,
            "The value is not changed."
        );
        assert!(
            value.0 < protocol_settings.minimum_delegator_deposit.0,
            "The value should be less than `minimum_delegator_deposit`."
        );
        protocol_settings.minimum_delegator_deposit_changing_amount = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_minimum_total_stake_price_for_booting(&mut self, value: U128) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value.0 != protocol_settings.minimum_total_stake_price_for_booting.0,
            "The value is not changed."
        );
        protocol_settings.minimum_total_stake_price_for_booting = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_maximum_market_value_percent_of_near_fungible_tokens(&mut self, value: u16) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value != protocol_settings.maximum_market_value_percent_of_near_fungible_tokens,
            "The value is not changed."
        );
        protocol_settings.maximum_market_value_percent_of_near_fungible_tokens = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_maximum_market_value_percent_of_wrapped_appchain_token(&mut self, value: u16) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value != protocol_settings.maximum_market_value_percent_of_wrapped_appchain_token,
            "The value is not changed."
        );
        protocol_settings.maximum_market_value_percent_of_wrapped_appchain_token = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_minimum_validator_count(&mut self, value: U64) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value.0 != protocol_settings.minimum_validator_count.0,
            "The value is not changed."
        );
        assert!(
            value.0 < protocol_settings.maximum_validator_count.0,
            "The value should be less than `maximum_validator_count`."
        );
        protocol_settings.minimum_validator_count = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_maximum_validator_count(&mut self, value: U64) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value.0 != protocol_settings.maximum_validator_count.0,
            "The value is not changed."
        );
        assert!(
            value.0 > protocol_settings.minimum_validator_count.0,
            "The value should be greater than `minimum_validator_count`."
        );
        protocol_settings.maximum_validator_count = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_maximum_validators_per_delegator(&mut self, value: U64) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value.0 != protocol_settings.maximum_validators_per_delegator.0,
            "The value is not changed."
        );
        protocol_settings.maximum_validators_per_delegator = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_unlock_period_of_validator_deposit(&mut self, value: U64) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value.0 != protocol_settings.unlock_period_of_validator_deposit.0,
            "The value is not changed."
        );
        protocol_settings.unlock_period_of_validator_deposit = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_unlock_period_of_delegator_deposit(&mut self, value: U64) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value.0 != protocol_settings.unlock_period_of_delegator_deposit.0,
            "The value is not changed."
        );
        protocol_settings.unlock_period_of_delegator_deposit = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_maximum_era_count_of_unwithdrawn_reward(&mut self, value: U64) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value.0 != protocol_settings.maximum_era_count_of_unwithdrawn_reward.0,
            "The value is not changed."
        );
        protocol_settings.maximum_era_count_of_unwithdrawn_reward = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_maximum_era_count_of_valid_appchain_message(&mut self, value: U64) {
        self.assert_owner();
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value.0
                != protocol_settings
                    .maximum_era_count_of_valid_appchain_message
                    .0,
            "The value is not changed."
        );
        protocol_settings.maximum_era_count_of_valid_appchain_message = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_validator_commission_percent(&mut self, value: u16) {
        self.assert_owner();
        assert!(value < 100, "Invalid percent value.");
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value != protocol_settings.validator_commission_percent,
            "The value is not changed."
        );
        protocol_settings.validator_commission_percent = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_maximum_allowed_unprofitable_era_count(&mut self, value: u16) {
        self.assert_owner();
        assert!(
            value < 10,
            "Invalid value for maximum allowed unprofitable era count."
        );
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            value != protocol_settings.maximum_allowed_unprofitable_era_count,
            "The value is not changed."
        );
        protocol_settings.maximum_allowed_unprofitable_era_count = value;
        self.protocol_settings.set(&protocol_settings);
    }
}

#[near_bindgen]
impl AppchainSettingsManager for AppchainAnchor {
    //
    fn set_rpc_endpoint(&mut self, rpc_endpoint: String) {
        self.assert_owner();
        let mut appchain_settings = self.appchain_settings.get().unwrap();
        appchain_settings.rpc_endpoint = rpc_endpoint;
        self.appchain_settings.set(&appchain_settings);
    }
    //
    fn set_subql_endpoint(&mut self, subql_endpoint: String) {
        self.assert_owner();
        let mut appchain_settings = self.appchain_settings.get().unwrap();
        appchain_settings.subql_endpoint = subql_endpoint;
        self.appchain_settings.set(&appchain_settings);
    }
    //
    fn set_era_reward(&mut self, era_reward: U128) {
        self.assert_owner();
        let mut appchain_settings = self.appchain_settings.get().unwrap();
        appchain_settings.era_reward = era_reward;
        self.appchain_settings.set(&appchain_settings);
    }
    //
    fn set_bonus_for_new_validator(&mut self, bonus_amount: U128) {
        self.assert_owner();
        let mut appchain_settings = self.appchain_settings.get().unwrap();
        appchain_settings.bonus_for_new_validator = bonus_amount;
        self.appchain_settings.set(&appchain_settings);
    }
}

#[near_bindgen]
impl AnchorSettingsManager for AppchainAnchor {
    //
    fn set_token_price_maintainer_account(&mut self, account_id: AccountId) {
        self.assert_owner();
        assert!(
            !account_id.eq(&self.owner),
            "This account should not be the same as the owner account."
        );
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        anchor_settings.token_price_maintainer_account = Some(account_id);
        self.anchor_settings.set(&anchor_settings);
    }
    //
    fn set_relayer_account(&mut self, account_id: AccountId) {
        self.assert_owner();
        assert!(
            !account_id.eq(&self.owner),
            "This account should not be the same as the owner account."
        );
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        anchor_settings.relayer_account = Some(account_id);
        self.anchor_settings.set(&anchor_settings);
    }
    //
    fn turn_on_beefy_light_client_witness_mode(&mut self) {
        self.assert_owner();
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            !anchor_settings.beefy_light_client_witness_mode,
            "Witness mode is already turned on."
        );
        anchor_settings.beefy_light_client_witness_mode = true;
        self.anchor_settings.set(&anchor_settings);
    }
    //
    fn turn_off_beefy_light_client_witness_mode(&mut self) {
        self.assert_owner();
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            anchor_settings.beefy_light_client_witness_mode,
            "Witness mode is already turned off."
        );
        anchor_settings.beefy_light_client_witness_mode = false;
        self.anchor_settings.set(&anchor_settings);
    }
}
