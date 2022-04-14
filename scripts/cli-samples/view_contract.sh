#!/bin/bash
set -e
#
export NEAR_ENV=$1
#
near view $2 get_owner ''
near view $2 get_anchor_settings ''
near view $2 get_appchain_settings ''
near view $2 get_protocol_settings ''
near view $2 get_oct_token ''
near view $2 get_wrapped_appchain_token ''
near view $2 get_near_fungible_tokens ''
near view $2 get_appchain_state ''
near view $2 get_anchor_status ''
near view $2 get_validator_set_info_of '{"era_number":"0"}'
near view $2 get_processing_status_of '{"era_number":"0"}'
near view $2 get_staking_history '{"index":"0"}'
near view $2 get_appchain_notification_history '{"index":"0"}'
near view $2 get_validator_list_of '{"era_number":null}'
near view $2 get_appchain_message_of '{"nonce":1}'
near view $2 get_appchain_message_processing_result_of '{"nonce":1}'
near view $2 get_storage_balance ''
near view $2 get_beefy_light_client_status ''
near view $2 get_latest_commitment_of_appchain ''
