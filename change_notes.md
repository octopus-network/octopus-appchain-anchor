# Change notes

## 20211128

* Add field `appchain_message_nonce` to `ValidatorSetProcessingStatus::DistributingReward`:

```rust
pub enum ValidatorSetProcessingStatus {
    CopyingFromLastEra {
        copying_validator_index: U64,
        copying_delegator_index: U64,
    },
    ApplyingStakingHistory {
        applying_index: U64,
    },
    ReadyForDistributingReward,
    DistributingReward {
        appchain_message_nonce: u32,
        distributing_validator_index: U64,
        distributing_delegator_index: U64,
    },
    Completed,
}
```

* Change the mechanism of processing the appchain message `EraRewardConcluded`. Now, if the `ValidatorSetProcessingStatus` of target validator set is `ReadyForDistributingReward` or `Completed`, this appchain message can be applied normally.

## 20211126

* Add data fields to `AnchorSettings`:

```rust
pub struct AnchorSettings {
    pub token_price_maintainer_account: AccountId,
    pub relayer_account: AccountId,
    pub beefy_light_client_witness_mode: bool,
}
```

* Add the following functions to trait `AnchorSettingsManager`:

```rust
    ///
    fn set_relayer_account(&mut self, account_id: AccountId);
    ///
    fn turn_on_beefy_light_client_witness_mode(&mut self);
    ///
    fn turn_off_beefy_light_client_witness_mode(&mut self);
```

* Change the mechanism of permissionless functions `start_updating_state_of_beefy_light_client`, `try_complete_updating_state_of_beefy_light_client` and `verify_and_apply_appchain_messages`:
  * If the `beefy_light_client_witness_mode` of `AnchorSettings` is set to `true`:
    * The function `start_updating_state_of_beefy_light_client` and `try_complete_updating_state_of_beefy_light_client` will fail directly.
    * The function `verify_and_apply_appchain_messages` will only assert that the function caller is exactly the account of `relayer_account` of `AnchorSettings`, and apply the appchain messages without verifying the proof passed by the parameters of the function.
  * If the `beefy_light_client_witness_mode` of `AnchorSettings` is set to `false`:
    * The proofs passed by the parameters of these functions will be verified before applying state changes to contract.

## 20211125

* Add sudo function `reset_beefy_light_client`:

```rust
    ///
    fn reset_beefy_light_client(&mut self, initial_public_keys: Vec<String>);
```

## 20211123

* Add interface enum in module `types`:

```rust
pub enum BeefyLightClientStatus {
    Uninitialized,
    UpdatingState,
    Ready,
}
```

* Add view function `get_beefy_light_client_status`:

```rust
    ///
    fn get_beefy_light_client_status(&self) -> BeefyLightClientStatus;
```

## 20211121

* Add the following interface data structs in module `types`:

```rust
pub enum MultiTxsOperationProcessingResult {
    NeedMoreGas,
    Ok,
    Error(String),
}

pub struct ValidatorMerkleProof {
    /// Root hash of generated merkle tree.
    pub root: Hash,
    /// Proof items (does not contain the leaf hash, nor the root obviously).
    ///
    /// This vec contains all inner node hashes necessary to reconstruct the root hash given the
    /// leaf hash.
    pub proof: Vec<Hash>,
    /// Number of leaves in the original tree.
    ///
    /// This is needed to detect a case where we have an odd number of leaves that "get promoted"
    /// to upper layers.
    pub number_of_leaves: u32,
    /// Index of the leaf the proof is for (0-based).
    pub leaf_index: u32,
    /// Leaf content.
    pub leaf: Vec<u8>,
}

pub struct AppchainCommitment {
    pub payload: Hash,
    pub block_number: U64,
    pub validator_set_id: u32,
}
```

* Change permissionless function `update_state_of_beefy_light_client` to the following permissionless functions:

```rust
    ///
    fn start_updating_state_of_beefy_light_client(
        &mut self,
        signed_commitment: Vec<u8>,
        validator_proofs: Vec<ValidatorMerkleProof>,
        mmr_leaf: Vec<u8>,
        mmr_proof: Vec<u8>,
    );
    ///
    fn try_complete_updating_state_of_beefy_light_client(
        &mut self,
    ) -> MultiTxsOperationProcessingResult;
```

* Add view function `get_latest_commitment_of_appchain`:

```rust
    /// Get the latest commitment data of appchain state
    fn get_latest_commitment_of_appchain(&self) -> Option<AppchainCommitment>;
```

* Change data type of return value of function `try_complete_switching_era` and `try_complete_distributing_reward`:

```rust
    ///
    fn try_complete_switching_era(&mut self) -> MultiTxsOperationProcessingResult;
    ///
    fn try_complete_distributing_reward(&mut self) -> MultiTxsOperationProcessingResult;
```

## 20211116

* Integrated implementation of beefy light client (by a crate in workspace).
* Add function `initialize_beefy_light_client`. It can only be called by owner account while the appchain is in `booting` state.

```rust
    fn initialize_beefy_light_client(&mut self, initial_public_keys: Vec<String>);
```

* Add permissionless function `update_state_of_beefy_light_client`.

```rust
    fn update_state_of_beefy_light_client(
        &mut self,
        payload: Vec<u8>,
        mmr_leaf: Vec<u8>,
        mmr_proof: Vec<u8>,
    );
```

* Change param names of function `verify_and_apply_appchain_messages`.

```rust
    fn verify_and_apply_appchain_messages(
        &mut self,
        encoded_messages: Vec<u8>,
        header: Vec<u8>,
        mmr_leaf: Vec<u8>,
        mmr_proof: Vec<u8>,
    ) -> Vec<AppchainMessageProcessingResult>;
```

## 20211107

* Remove `boot_nodes`, `chain_spec`, `raw_chain_spec` from `AppchainSettings`.
* Remove function `set_boot_nodes`, `set_chain_spec`, `set_raw_chain_spec`.
* Add `subql_endpoint` to `AppchainSettings`.
* Add function `set_subql_endpoint`.
* Add field `should_distribute_rewards` to `AppchainEvent::EraRewardConcluded`.
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
