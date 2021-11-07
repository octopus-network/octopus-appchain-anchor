# Change notes

## 20211106

* Remove `chain_spec`, `raw_chain_spec` from `AppchainSettings`.
* Remove function `set_chain_spec`, `set_raw_chain_spec`.
* Add `subql_endpoint` to `AppchainSettings`.
* Add function `set_subql_endpoint`.
* Add data type `AppchainMessageProcessingResult`:

```rust
pub enum AppchainMessageProcessingResult {
    Ok { nonce: u32, message: Option<String> },
    Error { nonce: u32, message: String },
}
```

* Add return value `Vec<AppchainMessageProcessingResult>` to function `verify_and_apply_appchain_messages`. As this function need to process multiple appchain messages, the process of single `AppchainMessage` will return a `AppchainMessageProcessingResult` now rather than making the whole function call fail.

## 20211103

* Add data type for appchain notification:

```rust
pub enum AppchainNotification {
    /// A certain amount of a NEAR fungible token has been locked in appchain anchor.
    NearFungibleTokenLocked {
        symbol: String,
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: U128,
    },
    /// A certain amount of wrapped appchain token is burnt in its contract
    /// in NEAR protocol.
    WrappedAppchainTokenBurnt {
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: U128,
    },
}

pub struct AppchainNotificationHistory {
    pub appchain_notification: AppchainNotification,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub index: U64,
}
```

* Add a function to `OwnerActions`:

```rust
    ///
    fn remove_appchain_notification_history_before(&mut self, index: U64);
```

* Add a function to `SudoActions`:

```rust
    ///
    fn reset_appchain_notification_histories(&mut self);
```

* Add the following view functions:

```rust
    /// Get the index range of appchain notification histories stored in anchor.
    fn get_index_range_of_appchain_notification_history(&self) -> IndexRange;
    /// Get appchain notification by index.
    /// If the param `index `is omitted, the latest notification will be returned.
    /// If the paran `index` is smaller than the start index, or bigger than the end index
    /// stored in anchor, or there is no event in anchor yet, `Option::None` will be returned.
    fn get_appchain_notification_history(
        &self,
        index: Option<U64>,
    ) -> Option<AppchainNotificationHistory>;
    /// Get appchain notification history by start index and quantity.
    /// If the param `quantity` is omitted, up to 50 events will be returned.
    fn get_appchain_notification_histories(
        &self,
        start_index: U64,
        quantity: Option<U64>,
    ) -> Vec<AppchainNotificationHistory>;
```

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

* Add trait `OwnerActions` and implement it for `AppchainAnchor`:

```rust
pub trait OwnerActions {
    ///
    fn remove_validator_set_before(&mut self, era_number: U64);
    ///
    fn remove_staking_history_before(&mut self, index: U64);
    ///
    fn remove_anchor_event_history_before(&mut self, index: U64);
}
```

* Add functions to trait `SudoActions`:

```rust
    ///
    fn remove_validator_set_of(&mut self, era_number: U64);
    ///
    fn reset_validator_set_histories(&mut self);
    ///
    fn reset_staking_histories(&mut self);
    ///
    fn reset_anchor_event_histories(&mut self);
```

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
