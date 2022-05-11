#!/bin/bash
set -e
#
export NEAR_ENV=testnet
export APPCHAIN_ID=xxxx
export REGISTRY_ACCOUNT_ID=registry.test_oct.testnet
export ANCHOR_ACCOUNT_ID=$APPCHAIN_ID'.'$REGISTRY_ACCOUNT_ID
#
#
#
near deploy --accountId $ANCHOR_ACCOUNT_ID --wasmFile res/appchain_anchor.wasm
#
ARGS='{"appchain_id":"'$APPCHAIN_ID'","appchain_registry":"'$REGISTRY_ACCOUNT_ID'","oct_token":"oct.beta_oct_relay.testnet"}'
near call $ANCHOR_ACCOUNT_ID new $ARGS --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
ARGS='{"account_id":"'$ANCHOR_ACCOUNT_ID'","registration_only":null}'
near call oct.beta_oct_relay.testnet storage_deposit $ARGS --accountId $ANCHOR_ACCOUNT_ID --deposit 0.00125
#
near call $ANCHOR_ACCOUNT_ID migrate_state '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near view $ANCHOR_ACCOUNT_ID get_anchor_version
near view $ANCHOR_ACCOUNT_ID get_anchor_status
near view $ANCHOR_ACCOUNT_ID get_anchor_settings
near view $ANCHOR_ACCOUNT_ID get_protocol_settings
near view $ANCHOR_ACCOUNT_ID get_wrapped_appchain_token
near view $ANCHOR_ACCOUNT_ID get_near_fungible_tokens
