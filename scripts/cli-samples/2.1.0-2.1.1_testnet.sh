#!/bin/bash
set -e
#
export NEAR_CLI_TESTNET_RPC_SERVER_URL=https://near-testnet.infura.io/v3/4f80a04e6eb2437a9ed20cb874e10d55
# export NEAR_CLI_TESTNET_RPC_SERVER_URL=https://public-rpc.blockpi.io/http/near-testnet
export NEAR_ENV=testnet
export APPCHAIN_ID=$1
export REGISTRY_ACCOUNT_ID=registry.test_oct.testnet
export ANCHOR_ACCOUNT_ID=$APPCHAIN_ID'.'$REGISTRY_ACCOUNT_ID
export FAUCET_ACCOUNT_ID=wat-faucet.$ANCHOR_ACCOUNT_ID
#
#
#
cp ~/.near-credentials/testnet/$ANCHOR_ACCOUNT_ID.json ~/.near-credentials/testnet/$FAUCET_ACCOUNT_ID.json
sed -i '' "s/$ANCHOR_ACCOUNT_ID/$FAUCET_ACCOUNT_ID/" ~/.near-credentials/testnet/$FAUCET_ACCOUNT_ID.json
#
near create-account $FAUCET_ACCOUNT_ID --masterAccount $ANCHOR_ACCOUNT_ID --publicKey ed25519:99dtM6c33a1NCoRrhM7cwfDaJy123JLdsT9i4M9wYk3f --initialBalance 2
near deploy --accountId $FAUCET_ACCOUNT_ID --wasmFile res/wat_faucet.wasm
near call $FAUCET_ACCOUNT_ID new '' --accountId $FAUCET_ACCOUNT_ID --gas 200000000000000
#
near deploy --accountId $ANCHOR_ACCOUNT_ID --wasmFile res/appchain_anchor.wasm
#
near call $ANCHOR_ACCOUNT_ID migrate_state '' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID set_bonus_for_new_validator '{"bonus_amount":"1000000000000000000"}' --accountId $ANCHOR_ACCOUNT_ID --gas 200000000000000
#
near view $ANCHOR_ACCOUNT_ID get_appchain_settings
