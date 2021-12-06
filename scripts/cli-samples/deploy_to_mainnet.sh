#
export NEAR_ENV=mainnet
#
export ANCHOR_ACCOUNT_ID=debionetwork.octopus-registry.near
export REGISTRY_ACCOUNT_ID=octopus-registry.near
#
near deploy --accountId $ANCHOR_ACCOUNT_ID --wasmFile res/appchain_anchor.wasm
near call $ANCHOR_ACCOUNT_ID new '{"appchain_id":"debionetwork","appchain_registry":"octopus-registry.near","oct_token":"f5cfbc74057c610c8ef151a439252680ac68c6dc.factory.bridge.near"}' --accountId $REGISTRY_ACCOUNT_ID --gas 200000000000000
near call f5cfbc74057c610c8ef151a439252680ac68c6dc.factory.bridge.near storage_deposit '{"account_id":"debionetwork.octopus-registry.near","registration_only":null}' --accountId $REGISTRY_ACCOUNT_ID --deposit 1
#
near call $ANCHOR_ACCOUNT_ID migrate_state '' --accountId $REGISTRY_ACCOUNT_ID --gas 200000000000000
#
near call $ANCHOR_ACCOUNT_ID set_token_price_maintainer_account '{"account_id":"octopus-registry.near"}' --accountId $REGISTRY_ACCOUNT_ID
near call $ANCHOR_ACCOUNT_ID set_price_of_oct_token '{"price":"2740000"}' --accountId $REGISTRY_ACCOUNT_ID
#
near call $ANCHOR_ACCOUNT_ID initialize_beefy_light_client '{"initial_public_keys": ["0x02ccfa1aef0c9f012f055e216ae7546556cadd09896215edbf837b07cb6362b1f8","0x0338478804f8f14bcaab2b7c3ec85481ac4c59994ff5f92125901465b6b7c177e0","0x02c55dee371aa4ad6c2eb7b030354d38e2be227f8f221c440a83b2b0a71d42f8f9","0x03e7446dcfb7d23bbb4a98a7fe93ea9e4be5ae1734e3d7d96645c9432db5dedb37"]}' --accountId $REGISTRY_ACCOUNT_ID
#
near call $ANCHOR_ACCOUNT_ID set_relayer_account '{"account_id": "octopus-counter.near"}' --accountId $REGISTRY_ACCOUNT_ID
near call $ANCHOR_ACCOUNT_ID turn_on_beefy_light_client_witness_mode --accountId $REGISTRY_ACCOUNT_ID
near call $ANCHOR_ACCOUNT_ID set_rpc_endpoint '{"rpc_endpoint": "wss://gateway.mainnet.octopus.network/debionetwork/ae48005a0c7ecb4053394559a7f4069e"}' --accountId $REGISTRY_ACCOUNT_ID
#
near call $ANCHOR_ACCOUNT_ID set_price_of_wrapped_appchain_token '{"price": "88451"}' --accountId $REGISTRY_ACCOUNT_ID
#
near call $ANCHOR_ACCOUNT_ID go_live --accountId $REGISTRY_ACCOUNT_ID
