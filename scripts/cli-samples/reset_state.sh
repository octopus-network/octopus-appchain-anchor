#!/bin/bash
set -e
#
export NEAR_ENV=testnet
export ANCHOR_ACCOUNT_ID=xxxx
export END_ERA=yy
#
#
#
for ((i=$END_ERA;i>=0;i--))
do
    param="'{\"era_number\":\"${i}\"}'"
    near call $ANCHOR_ACCOUNT_ID clear_reward_distribution_records $param --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
done
#
near call $ANCHOR_ACCOUNT_ID clear_unbonded_stakes '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
for ((i=$END_ERA;i>=0;i--))
do
    param="'{\"era_number\":\"${i}\"}'"
    near call $ANCHOR_ACCOUNT_ID clear_unwithdrawn_rewards $param --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
done
#
near call $ANCHOR_ACCOUNT_ID clear_appchain_notification_histories '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
for ((i=$END_ERA-1;i>=0;i--))
do
    param="'{\"era_number\":\"${i}\"}'"
    near call $ANCHOR_ACCOUNT_ID reset_validator_set_histories_to $param --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
done
#
near call $ANCHOR_ACCOUNT_ID reset_next_validator_set_to '{"era_number":"0"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID reset_staking_histories_to '{"era_number":"0"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID refresh_user_staking_histories '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID reset_validator_profiles_to '{"era_number":"0"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID clear_appchain_messages '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID set_latest_applied_appchain_message_nonce '{"nonce":0}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
