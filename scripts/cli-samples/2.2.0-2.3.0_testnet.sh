#!/bin/bash
set -e
#
# export NEAR_CLI_TESTNET_RPC_SERVER_URL=https://near-testnet.infura.io/v3/4f80a04e6eb2437a9ed20cb874e10d55
# export NEAR_CLI_TESTNET_RPC_SERVER_URL=https://public-rpc.blockpi.io/http/near-testnet
export NEAR_ENV=testnet
export APPCHAIN_ID=$1
export REGISTRY_ACCOUNT_ID=registry.test_oct.testnet
export ANCHOR_ACCOUNT_ID=$APPCHAIN_ID'.'$REGISTRY_ACCOUNT_ID
export NEAR_VAULT_ACCOUNT_ID=near-vault.$ANCHOR_ACCOUNT_ID
#
#
#
cp ~/.near-credentials/testnet/$ANCHOR_ACCOUNT_ID.json ~/.near-credentials/testnet/$NEAR_VAULT_ACCOUNT_ID.json
sed -i '' "s/$ANCHOR_ACCOUNT_ID/$NEAR_VAULT_ACCOUNT_ID/" ~/.near-credentials/testnet/$NEAR_VAULT_ACCOUNT_ID.json
#
near deploy --accountId $ANCHOR_ACCOUNT_ID --wasmFile res/appchain_anchor.wasm
#
near call $ANCHOR_ACCOUNT_ID migrate_state '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near view $ANCHOR_ACCOUNT_ID get_native_near_token
#
WASM_BYTES='cat res/near_vault.wasm | base64'
near call $ANCHOR_ACCOUNT_ID store_wasm_of_near_vault_contract $(eval "$WASM_BYTES") --base64 --accountId $ANCHOR_ACCOUNT_ID --deposit 3 --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID deploy_near_vault_contract '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
