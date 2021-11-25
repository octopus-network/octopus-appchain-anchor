#
export NEAR_ENV=testnet
#
#
#
near deploy --accountId ychain.dev-oct-registry.testnet --wasmFile res/appchain_anchor.wasm
near call ychain.dev-oct-registry.testnet new '{"appchain_id":"ychain","appchain_registry":"dev-oct-registry.testnet","oct_token":"oct.beta_oct_relay.testnet"}' --accountId dev-oct-registry.testnet --gas 200000000000000
near call oct.beta_oct_relay.testnet storage_deposit '{"account_id":"ychain.dev-oct-registry.testnet","registration_only":null}' --accountId dev-oct-registry.testnet --deposit 1
near call ychain.dev-oct-registry.testnet migrate_state '' --accountId dev-oct-registry.testnet --gas 200000000000000
#
#
#
near deploy --accountId wchain.dev-oct-registry.testnet --wasmFile res/appchain_anchor.wasm
near call wchain.dev-oct-registry.testnet new '{"appchain_id":"wchain","appchain_registry":"dev-oct-registry.testnet","oct_token":"oct.beta_oct_relay.testnet"}' --accountId dev-oct-registry.testnet --gas 200000000000000
near call oct.beta_oct_relay.testnet storage_deposit '{"account_id":"wchain.dev-oct-registry.testnet","registration_only":null}' --accountId dev-oct-registry.testnet --deposit 1
near call wchain.dev-oct-registry.testnet migrate_state '' --accountId dev-oct-registry.testnet --gas 200000000000000
#
#
#
near deploy --accountId tchain.dev-oct-registry.testnet --wasmFile res/appchain_anchor.wasm
near call tchain.dev-oct-registry.testnet new '{"appchain_id":"tchain","appchain_registry":"dev-oct-registry.testnet","oct_token":"oct.beta_oct_relay.testnet"}' --accountId dev-oct-registry.testnet --gas 200000000000000
near call oct.beta_oct_relay.testnet storage_deposit '{"account_id":"tchain.dev-oct-registry.testnet","registration_only":null}' --accountId dev-oct-registry.testnet --deposit 1
near call tchain.dev-oct-registry.testnet migrate_state '' --accountId dev-oct-registry.testnet --gas 200000000000000
