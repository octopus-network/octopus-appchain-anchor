#
export NEAR_ENV=testnet
#
#
#
near deploy --accountId easydeal.registry.test_oct.testnet --wasmFile res/appchain_anchor.wasm
near call easydeal.registry.test_oct.testnet new '{"appchain_id":"easydeal","appchain_registry":"registry.test_oct.testnet","oct_token":"oct.beta_oct_relay.testnet"}' --accountId registry.test_oct.testnet --gas 200000000000000
near call oct.beta_oct_relay.testnet storage_deposit '{"account_id":"easydeal.registry.test_oct.testnet","registration_only":null}' --accountId dev-oct-registry.testnet --deposit 1
near call easydeal.registry.test_oct.testnet migrate_state '' --accountId registry.test_oct.testnet --gas 200000000000000
#
near deploy --accountId debionetwork.registry.test_oct.testnet --wasmFile res/appchain_anchor.wasm
near call debionetwork.registry.test_oct.testnet migrate_state '' --accountId debionetwork.registry.test_oct.testnet --gas 200000000000000
