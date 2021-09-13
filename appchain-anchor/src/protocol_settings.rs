use crate::*;

impl Default for ProtocolSettings {
    fn default() -> Self {
        Self {
            minimum_validator_deposit: 10_000 * OCT_DECIMALS_VALUE,
            minimum_delegator_deposit: 1000 * OCT_DECIMALS_VALUE,
            minimum_total_stake_for_booting: 500_000 * OCT_DECIMALS_VALUE,
            maximum_market_value_percent_of_nep141_tokens: 33,
            maximum_market_value_percent_of_wrapped_appchain_token: 67,
            minimum_validator_count: 13,
            maximum_validators_per_delegator: 16,
            unlock_period_of_validator_deposit: 21,
            unlock_period_of_delegator_deposit: 7,
        }
    }
}

pub trait ProtocolSettingsManager {
    ///
    fn get_protocol_settings(&self) -> ProtocolSettings;
    ///
    fn change_minimum_validator_deposit(&mut self, value: Balance);
    ///
    fn change_minimum_delegator_deposit(&mut self, value: Balance);
    ///
    fn change_minimum_total_stake_for_booting(&mut self, value: Balance);
    ///
    fn change_maximum_market_value_percent_of_nep141_tokens(&mut self, value: u16);
    ///
    fn change_maximum_market_value_percent_of_wrapped_appchain_token(&mut self, value: u16);
    ///
    fn change_minimum_validator_count(&mut self, value: u16);
    ///
    fn change_maximum_validators_per_delegator(&mut self, value: u16);
    ///
    fn change_unlock_period_of_validator_deposit(&mut self, value: u16);
    ///
    fn change_unlock_period_of_delegator_deposit(&mut self, value: u16);
}
