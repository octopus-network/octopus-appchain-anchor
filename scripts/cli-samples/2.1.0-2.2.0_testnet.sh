#!/bin/bash
#!/bin/bash
set -e
#
export NEAR_ENV=testnet
export APPCHAIN_ID=$1
export REGISTRY_ACCOUNT_ID=registry.test_oct.testnet
export ANCHOR_ACCOUNT_ID=$APPCHAIN_ID'.'$REGISTRY_ACCOUNT_ID
#
near deploy --accountId $ANCHOR_ACCOUNT_ID --wasmFile res/appchain_anchor.wasm
#
near call $ANCHOR_ACCOUNT_ID migrate_state '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID migrate_staking_histories '{"start_index":"0"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID migrate_appchain_notification_histories '{"start_index":"0"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
near call $ANCHOR_ACCOUNT_ID migrate_appchain_messages '{"start_nonce":0}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
