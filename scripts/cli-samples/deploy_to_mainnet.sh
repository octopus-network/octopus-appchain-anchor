#!/bin/bash
set -e
#
export NEAR_ENV=mainnet
export APPCHAIN_ID=xxxx
export REGISTRY_ACCOUNT_ID=octopus-registry.near
export ANCHOR_ACCOUNT_ID=$APPCHAIN_ID'.'$REGISTRY_ACCOUNT_ID
#
#
#
near deploy --accountId $ANCHOR_ACCOUNT_ID --wasmFile res/appchain_anchor.wasm
#
ARGS='{"appchain_id":"'$APPCHAIN_ID'","appchain_registry":"'$REGISTRY_ACCOUNT_ID'","oct_token":"f5cfbc74057c610c8ef151a439252680ac68c6dc.factory.bridge.near"}'
near call $ANCHOR_ACCOUNT_ID new $ARGS --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
ARGS='{"account_id":"'$ANCHOR_ACCOUNT_ID'","registration_only":null}'
near call f5cfbc74057c610c8ef151a439252680ac68c6dc.factory.bridge.near storage_deposit $ARGS --accountId $ANCHOR_ACCOUNT_ID --deposit 1
#
near call $ANCHOR_ACCOUNT_ID migrate_state '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID set_token_price_maintainer_account '{"account_id":"octopus-registry.near"}' --accountId $ANCHOR_ACCOUNT_ID
near call $ANCHOR_ACCOUNT_ID set_price_of_oct_token '{"price":"2740000"}' --accountId $ANCHOR_ACCOUNT_ID
#
near call $ANCHOR_ACCOUNT_ID change_minimum_delegator_deposit '{"value":"200000000000000000000"}' --accountId $ANCHOR_ACCOUNT_ID
