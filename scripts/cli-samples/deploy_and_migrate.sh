#
export NEAR_ENV=mainnet
#
export ANCHOR_ACCOUNT_ID=debionetwork.octopus-registry.near
export OWNER_ACCOUNT_ID=octopus-registry.near
#
near deploy --accountId $ANCHOR_ACCOUNT_ID --wasmFile res/appchain_anchor.wasm
#
near call $ANCHOR_ACCOUNT_ID migrate_state '' --accountId $OWNER_ACCOUNT_ID --gas 200000000000000
