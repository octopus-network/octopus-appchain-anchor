#
export NEAR_ENV=mainnet
#
#
#
near deploy --accountId debionetwork.octopus-registry.near --wasmFile res/appchain_anchor.wasm
near call debionetwork.octopus-registry.near new '{"appchain_id":"debionetwork","appchain_registry":"octopus-registry.near","oct_token":"f5cfbc74057c610c8ef151a439252680ac68c6dc.factory.bridge.near"}' --accountId octopus-registry.near --gas 200000000000000
near call f5cfbc74057c610c8ef151a439252680ac68c6dc.factory.bridge.near storage_deposit '{"account_id":"debionetwork.octopus-registry.near","registration_only":null}' --accountId octopus-registry.near --deposit 1
#
near call debionetwork.octopus-registry.near migrate_state '' --accountId octopus-registry.near --gas 200000000000000
#
near call debionetwork.octopus-registry.near set_token_price_maintainer_account '{"account_id":"octopus-registry.near"}' --accountId octopus-registry.near
near call debionetwork.octopus-registry.near set_price_of_oct_token '{"price":"2740000"}' --accountId octopus-registry.near
#
