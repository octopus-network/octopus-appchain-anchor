#
export NEAR_ENV=testnet
export ANCHOR_ACCOUNT_ID=debionetwork.registry.test_oct.testnet
#
#
#
near call $ANCHOR_ACCOUNT_ID clear_reward_distribution_records '{"era_number":"4"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID clear_reward_distribution_records '{"era_number":"3"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID clear_reward_distribution_records '{"era_number":"2"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID clear_reward_distribution_records '{"era_number":"1"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID clear_reward_distribution_records '{"era_number":"0"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID clear_unbonded_stakes '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID clear_unwithdrawn_rewards '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID clear_anchor_event_histories '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID clear_appchain_notification_histories '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID reset_validator_set_histories_to '{"era_number":"3"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID reset_validator_set_histories_to '{"era_number":"2"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID reset_validator_set_histories_to '{"era_number":"1"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID reset_validator_set_histories_to '{"era_number":"0"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID reset_validator_profiles_to '{"era_number":"0"}' --accountId $ANCHOR_ACCOUNT_ID
