#!/bin/bash
set -e
#
export NEAR_ENV=testnet
export APPCHAIN_ID=$1
export REGISTRY_ACCOUNT_ID=registry.test_oct.testnet
export ANCHOR_ACCOUNT_ID=$APPCHAIN_ID'.'$REGISTRY_ACCOUNT_ID
#
near deploy --accountId $ANCHOR_ACCOUNT_ID --initFunction 'migrate_state' --initArgs '{}' --wasmFile res/appchain_anchor.wasm --force
#
near call $ANCHOR_ACCOUNT_ID migrate_appchain_messages '{"start_nonce":0}' --accountId $ANCHOR_ACCOUNT_ID --gas 300000000000000
