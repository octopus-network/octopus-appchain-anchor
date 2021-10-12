use crate::*;
use core::convert::From;

impl Default for ProtocolSettings {
    fn default() -> Self {
        Self {
            minimum_validator_deposit: U128::from(10_000 * OCT_DECIMALS_VALUE),
            minimum_delegator_deposit: U128::from(1000 * OCT_DECIMALS_VALUE),
            minimum_total_stake_price_for_booting: U128::from(
                500_000 * USD_DECIMALS_VALUE * OCT_DECIMALS_VALUE,
            ),
            maximum_market_value_percent_of_near_fungible_tokens: 33,
            maximum_market_value_percent_of_wrapped_appchain_token: 67,
            minimum_validator_count: U64::from(13),
            maximum_validators_per_delegator: U64::from(16),
            unlock_period_of_validator_deposit: U64::from(21),
            unlock_period_of_delegator_deposit: U64::from(7),
            maximum_era_count_of_unwithdrawn_reward: U64::from(84),
            maximum_era_count_of_valid_appchain_message: U64::from(7),
            delegation_fee_percent: 20,
        }
    }
}

pub trait ProtocolSettingsManager {
    ///
    fn change_minimum_validator_deposit(&mut self, value: U128);
    ///
    fn change_minimum_delegator_deposit(&mut self, value: U128);
    ///
    fn change_minimum_total_stake_price_for_booting(&mut self, value: U128);
    ///
    fn change_maximum_market_value_percent_of_near_fungible_tokens(&mut self, value: u16);
    ///
    fn change_maximum_market_value_percent_of_wrapped_appchain_token(&mut self, value: u16);
    ///
    fn change_minimum_validator_count(&mut self, value: U64);
    ///
    fn change_maximum_validators_per_delegator(&mut self, value: U64);
    ///
    fn change_unlock_period_of_validator_deposit(&mut self, value: U64);
    ///
    fn change_unlock_period_of_delegator_deposit(&mut self, value: U64);
    ///
    fn change_maximum_era_count_of_unwithdrawn_reward(&mut self, value: U64);
    ///
    fn change_maximum_era_count_of_valid_appchain_message(&mut self, value: U64);
    ///
    fn change_delegation_fee_percent(&mut self, value: u16);
}

pub trait AppchainSettingsManager {
    ///
    fn set_chain_spec(&mut self, chain_spec: String);
    ///
    fn set_raw_chain_spec(&mut self, raw_chain_spec: String);
    ///
    fn set_boot_nodes(&mut self, boot_nodes: String);
    ///
    fn set_rpc_endpoint(&mut self, rpc_endpoint: String);
    ///
    fn set_era_reward(&mut self, era_reward: U128);
}

pub trait AnchorSettingsManager {
    ///
    fn set_token_price_maintainer_account(&mut self, account_id: AccountId);
}

#[near_bindgen]
impl ProtocolSettingsManager for AppchainAnchor {
    //
    fn change_minimum_validator_deposit(&mut self, value: U128) {
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        protocol_settings.minimum_validator_deposit = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_minimum_delegator_deposit(&mut self, value: U128) {
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        protocol_settings.minimum_delegator_deposit = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_minimum_total_stake_price_for_booting(&mut self, value: U128) {
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        protocol_settings.minimum_total_stake_price_for_booting = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_maximum_market_value_percent_of_near_fungible_tokens(&mut self, value: u16) {
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        protocol_settings.maximum_market_value_percent_of_near_fungible_tokens = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_maximum_market_value_percent_of_wrapped_appchain_token(&mut self, value: u16) {
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        protocol_settings.maximum_market_value_percent_of_wrapped_appchain_token = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_minimum_validator_count(&mut self, value: U64) {
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        protocol_settings.minimum_validator_count = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_maximum_validators_per_delegator(&mut self, value: U64) {
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        protocol_settings.maximum_validators_per_delegator = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_unlock_period_of_validator_deposit(&mut self, value: U64) {
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        protocol_settings.unlock_period_of_validator_deposit = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_unlock_period_of_delegator_deposit(&mut self, value: U64) {
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        protocol_settings.unlock_period_of_delegator_deposit = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_maximum_era_count_of_unwithdrawn_reward(&mut self, value: U64) {
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        protocol_settings.maximum_era_count_of_unwithdrawn_reward = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_maximum_era_count_of_valid_appchain_message(&mut self, value: U64) {
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        protocol_settings.maximum_era_count_of_valid_appchain_message = value;
        self.protocol_settings.set(&protocol_settings);
    }
    //
    fn change_delegation_fee_percent(&mut self, value: u16) {
        let mut protocol_settings = self.protocol_settings.get().unwrap();
        protocol_settings.delegation_fee_percent = value;
        self.protocol_settings.set(&protocol_settings);
    }
}

#[near_bindgen]
impl AppchainSettingsManager for AppchainAnchor {
    //
    fn set_chain_spec(&mut self, chain_spec: String) {
        let mut appchain_settings = self.appchain_settings.get().unwrap();
        appchain_settings.chain_spec.clear();
        appchain_settings.chain_spec.push_str(&chain_spec);
        self.appchain_settings.set(&appchain_settings);
    }
    //
    fn set_raw_chain_spec(&mut self, raw_chain_spec: String) {
        let mut appchain_settings = self.appchain_settings.get().unwrap();
        appchain_settings.raw_chain_spec.clear();
        appchain_settings.raw_chain_spec.push_str(&raw_chain_spec);
        self.appchain_settings.set(&appchain_settings);
    }
    //
    fn set_boot_nodes(&mut self, boot_nodes: String) {
        let mut appchain_settings = self.appchain_settings.get().unwrap();
        appchain_settings.boot_nodes.clear();
        appchain_settings.boot_nodes.push_str(&boot_nodes);
        self.appchain_settings.set(&appchain_settings);
    }
    //
    fn set_rpc_endpoint(&mut self, rpc_endpoint: String) {
        let mut appchain_settings = self.appchain_settings.get().unwrap();
        appchain_settings.rpc_endpoint.clear();
        appchain_settings.rpc_endpoint.push_str(&rpc_endpoint);
        self.appchain_settings.set(&appchain_settings);
    }
    //
    fn set_era_reward(&mut self, era_reward: U128) {
        let mut appchain_settings = self.appchain_settings.get().unwrap();
        appchain_settings.era_reward = era_reward;
        self.appchain_settings.set(&appchain_settings);
    }
}

#[near_bindgen]
impl AnchorSettingsManager for AppchainAnchor {
    //
    fn set_token_price_maintainer_account(&mut self, account_id: AccountId) {
        let mut anchor_settings = self.anchor_settings.get().unwrap();
        anchor_settings.token_price_maintainer_account = account_id;
        self.anchor_settings.set(&anchor_settings);
    }
}
