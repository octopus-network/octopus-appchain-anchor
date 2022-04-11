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
near deploy --accountId myriad.registry.test_oct.testnet --wasmFile res/appchain_anchor.wasm
near call myriad.registry.test_oct.testnet migrate_state '' --accountId myriad.registry.test_oct.testnet --gas 200000000000000
#
near deploy --accountId fusotao.registry.test_oct.testnet --wasmFile res/appchain_anchor.wasm
near call fusotao.registry.test_oct.testnet migrate_state '' --accountId fusotao.registry.test_oct.testnet --gas 200000000000000
#
near deploy --accountId barnacle-evm.registry.test_oct.testnet --wasmFile res/appchain_anchor.wasm
near call barnacle-evm.registry.test_oct.testnet migrate_state '' --accountId barnacle-evm.registry.test_oct.testnet --gas 200000000000000
#
export ANCHOR_ACCOUNT_ID=barnacle-evm.registry.test_oct.testnet
near view $ANCHOR_ACCOUNT_ID get_anchor_version
near view $ANCHOR_ACCOUNT_ID get_anchor_status
near view $ANCHOR_ACCOUNT_ID get_anchor_settings
near view $ANCHOR_ACCOUNT_ID get_protocol_settings
near view $ANCHOR_ACCOUNT_ID get_wrapped_appchain_token
near view $ANCHOR_ACCOUNT_ID get_near_fungible_tokens
