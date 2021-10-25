# Change notes

## 20211025

* Add view function `get_delegations_of` for querying all delegations of a certain validator in a certain era.
* Add field `validator_id_in_appchain` to `types::AppchainValidator`.
* Add field `validator_id` to `types::AppchainDelegator`.
* Move function `set_metadata_of_wrapped_appchain_token` and `set_premined_balance_of_wrapped_appchain_token` to module `sudo_actions`.
* Add function `sync_basedata_of_wrapped_appchain_token` for the contract of wrapped appchain token to sync basedata back to this contract. Refer to [Initial deployment](https://github.com/octopus-network/octopus-appchain-anchor#initial-deployment).
* Add restriction to staking actions as follow:

Staking action | AppchainState: Staging | AppchainState: Booting | AppchainState: Active | AppchainState: Frozen | AppchainState: Broken
---|---|---|---|---|---
register_validator | allowed |  | allowed |  |
increase_stake | allowed |  | allowed |  |
register_delegator | allowed |  | allowed |  |
increase_delegation | allowed |  | allowed |  |
decrease_stake |  |  | allowed |  |
decrease_delegation |  |  | allowed |  |
unbond_stake |  |  | allowed |  | allowed
unbond_delegation |  |  | allowed |  | allowed
