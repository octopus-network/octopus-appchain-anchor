#!/bin/bash
set -e
#
export NEAR_ENV=testnet
export ANCHOR_ACCOUNT_ID=$1
export END_ERA=$2
#
#
#
for ((i=$END_ERA;i>=0;i--))
do
    param="'{\"era_number\":\"${i}\"}'"
    near call $ANCHOR_ACCOUNT_ID clear_reward_distribution_records $param --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
    near call $ANCHOR_ACCOUNT_ID clear_unwithdrawn_rewards $param --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
    near call $ANCHOR_ACCOUNT_ID remove_validator_set_history_of $param --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
done
#
near call $ANCHOR_ACCOUNT_ID clear_unbonded_stakes '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID clear_next_validator_set '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID clear_user_staking_histories '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID clear_staking_histories '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID clear_validator_profiles '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID clear_appchain_messages '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID clear_appchain_notification_histories '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID clear_external_assets_registration '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID clear_contract_level_lazy_option_values '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
#
#
near view-state $ANCHOR_ACCOUNT_ID
