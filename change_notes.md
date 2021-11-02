# Change notes

## 20211025

* Add view function `get_delegations_of` for querying all delegations of a certain delegator in a certain era.
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

## 20211102

* Change `StakingDepositMessage::RegisterValidator` as the following:

```rust
    RegisterValidator {
        validator_id_in_appchain: Option<String>,
        can_be_delegated_to: bool,
        profile: HashMap<String, String>,
    },
```

* Add trait `ValidatorActions` and implement it for `AppchainAnchor`:

```rust
pub trait ValidatorActions {
    ///
    fn set_validator_id_in_appchain(&mut self, account_id_in_appchain: String);
    ///
    fn set_validator_profile(&mut self, profile: HashMap<String, String>);
}
```

* Add the following view functions:

```rust
    /// Get profile of a certain validator.
    fn get_validator_profile(&self, validator_id: AccountId) -> Option<ValidatorProfile>;
    /// Get validator profile by his/her account id in appchain.
    fn get_validator_profile_by_id_in_appchain(
        &self,
        validator_id_in_appchain: String,
    ) -> Option<ValidatorProfile>;
```
