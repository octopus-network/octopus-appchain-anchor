module.exports = {
  changeMethods: [
    'new',
    'set_owner',
    // protocol settings
    'change_minimum_validator_deposit',
    'change_minimum_delegator_deposit',
    'change_minimum_total_stake_price_for_booting',
    'change_maximum_market_value_percent_of_near_fungible_tokens',
    'change_maximum_market_value_percent_of_wrapped_appchain_token',
    'change_minimum_validator_count',
    'change_maximum_validators_per_delegator',
    'change_unlock_period_of_validator_deposit',
    'change_unlock_period_of_delegator_deposit',
    'change_maximum_era_count_of_unwithdrawn_reward',
    'change_maximum_era_count_of_valid_appchain_message',
    'change_delegation_fee_percent',
    // appchain settings
    'set_chain_spec',
    'set_raw_chain_spec',
    'set_boot_nodes',
    'set_rpc_endpoint',
    'set_era_reward',
    // anchor settings
    'set_token_price_maintainer_account',
  ],
  viewMethods: [
    'get_owner',
    'get_protocol_settings',
    'get_appchain_settings',
    'get_anchor_settings',
    'get_appchain_state',
    'get_anchor_status',
  ],
};
