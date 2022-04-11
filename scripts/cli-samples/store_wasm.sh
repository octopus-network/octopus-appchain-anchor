#!/bin/bash
set -e
#
export NEAR_ENV=testnet
#
export ANCHOR_ACCOUNT_ID=barnacle-evm.registry.test_oct.testnet
#
WASM_BYTES='cat ../../res/wrapped_appchain_nft.wasm | base64'
near call $ANCHOR_ACCOUNT_ID store_wasm_of_wrapped_appchain_nft_contract $(eval "$WASM_BYTES") --base64 --accountId $ANCHOR_ACCOUNT_ID --deposit 3 --gas 200000000000000
