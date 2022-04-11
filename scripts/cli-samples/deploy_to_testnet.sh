#!/bin/bash
set -e
#
export NEAR_ENV=testnet
export APPCHAIN_ID=xxxx
export REGISTRY_ACCOUNT_ID=registry.test_oct.testnet
export ANCHOR_ACCOUNT_ID=$APPCHAIN_ID'.'$REGISTRY_ACCOUNT_ID
#
export ANCHOR_ACCOUNT_ID=barnacle-evm.registry.test_oct.testnet
#
near call easydeal.registry.test_oct.testnet new '{"appchain_id":"easydeal","appchain_registry":"registry.test_oct.testnet","oct_token":"oct.beta_oct_relay.testnet"}' --accountId registry.test_oct.testnet --gas 200000000000000
near call oct.beta_oct_relay.testnet storage_deposit '{"account_id":"easydeal.registry.test_oct.testnet","registration_only":null}' --accountId dev-oct-registry.testnet --deposit 1
#
near deploy --accountId $ANCHOR_ACCOUNT_ID --wasmFile res/appchain_anchor.wasm
near call $ANCHOR_ACCOUNT_ID migrate_state '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near view $ANCHOR_ACCOUNT_ID get_anchor_version
near view $ANCHOR_ACCOUNT_ID get_anchor_status
near view $ANCHOR_ACCOUNT_ID get_anchor_settings
near view $ANCHOR_ACCOUNT_ID get_protocol_settings
near view $ANCHOR_ACCOUNT_ID get_wrapped_appchain_token
near view $ANCHOR_ACCOUNT_ID get_near_fungible_tokens
