# Implementation detail of appchain anchor

This contract provides an anchor for an appchain of [Octopus Network](https://oct.network). It is in charge of managing the necessary data of an appchain in NEAR protocol, , providing security and interoperability for the appchain.

Each appchain of Octopus Network will be bonded to an instance of this contract, which is deployed to a subaccount of [octopus-appchain-registry](https://github.com/octopus-network/octopus-appchain-registry).

Contents

* [Terminology](#terminology)
  * [Cross chain transfer in this contract](#cross-chain-transfer-in-this-contract)
* [Contract data design](#contract-data-design)
* [Contract initialization](#contract-initialization)
* [Manage NEAR fungible token](#manage-near-fungible-token)
  * [Register NEAR fungible token](#register-near-fungible-token)
  * [Set price of a NEAR fungible token](#set-price-of-a-near-fungible-token)
  * [Open bridging for a NEAR fungible token](#open-bridging-for-a-near-fungible-token)
  * [Close bridging for a NEAR fungible token](#close-bridging-for-a-near-fungible-token)
  * [Lock a certain amount of a NEAR fungible token](#lock-a-certain-amount-of-a-near-fungible-token)
  * [Unlock a certain amount of a NEAR fungible token](#unlock-a-certain-amount-of-a-near-fungible-token)
* [Manage wrapped appchain token](#manage-wrapped-appchain-token)
  * [Set metadata of wrapped appchain token](#set-metadata-of-wrapped-appchain-token)
  * [Set contract account of wrapped appchain token](#set-contract-account-of-wrapped-appchain-token)
  * [Set initial balance of wrapped appchain token](#set-initial-balance-of-wrapped-appchain-token)
  * [Set exchange rate of wrapped appchain token](#set-exchange-rate-of-wrapped-appchain-token)
  * [Mint wrapped appchain token](#mint-wrapped-appchain-token)
  * [Burn wrapped appchain token](#burn-wrapped-appchain-token)
* [Manage appchain settings](#manage-appchain-settings)
* [Manage anchor settings](#manage-anchor-settings)
* [Manage protocol settings](#manage-protocol-settings)
* [Process fungible token deposit](#process-fungible-token-deposit)
  * [Register validator](#register-validator)
  * [Increase stake of a validator](#increase-stake-of-a-validator)
  * [Register delegator](#register-delegator)
  * [Increase delegation of a delegator](#increase-delegation-of-a-delegator)
* [Handle relayed message](#handle-relayed-message)
* [Manage appchain staking](#manage-appchain-staking)
  * [Unbond stake](#unbond-stake)
  * [Decrease stake](#decrease-stake)
  * [Withdraw stake](#withdraw-stake)
  * [Unbond delegation](#unbond-delegation)
  * [Decrease delegation](#decrease-delegation)
  * [Withdraw delegation](#withdraw-delegation)
  * [Start applying staking history](#start-applying-staking-history)
  * [Apply staking history](#apply-staking-history)
* [Manage appchain lifecycle](#manage-appchain-lifecycle)
  * [Go booting](#go-booting)
  * [Go live](#go-live)

## Terminology and data design

* `owner`: The owner of this contract, that is the Octopus Network.
* `appchain registry`: A NEAR contract which manage the lifecycle of appchains of Octopus Network, controlled by Octopus Network.
* `appchain owner`: The owner of an appchain.
* `appchain state`: The state of an appchain, which is defined as:

```rust
pub enum AppchainState {
    /// The initial state of an appchain, after it is successfully registered.
    /// This state is managed by appchain registry.
    Registered,
    /// The state while the appchain is under auditing by Octopus Network.
    /// This state is managed by appchain registry.
    Auditing,
    /// The state while voter can upvote or downvote an appchain.
    /// This state is managed by appchain registry.
    InQueue,
    /// The state while validator and delegator can deposit OCT tokens to this contract
    /// to indicate their willing of staking for an appchain.
    Staging,
    /// The state while an appchain is booting.
    Booting,
    /// The state while an appchain is active normally.
    Active,
    /// The state while an appchain is under challenging, which all deposit and withdraw actions
    /// are frozen.
    Frozen,
    /// The state which an appchain is broken for some technical or governance reasons.
    Broken,
    /// The state which the lifecycle of an appchain is end.
    Dead,
}
```

* `account id in appchain`: The account id in the appchain, which is usually the public key of an account in the appchain. The id is bonded to an account id in NEAR protocol in this contract.

```rust
pub type AccountIdInAppchain = String;
```

* `validator`: A person who wants to act as a validator on the appchain corresponding to this contract. The person has to deposit a certain amount of OCT token in this contract. It is defined as:

```rust
pub struct Validator {
    /// The validator's id in NEAR protocol.
    pub validator_id: AccountId,
    /// The validator's id in the appchain.
    pub validator_id_in_appchain: AccountIdInAppchain,
    /// The account id in the appchain for receiving income of the validator in appchain.
    pub payee_id_in_appchain: AccountIdInAppchain,
    /// The block height when the validator is registered.
    pub registered_block_height: BlockHeight,
    /// The timestamp when the validator is registered.
    pub registered_timestamp: Timestamp,
}
```

* `delegator`: A person who wants to act as a delegator in the corresponding appchain. The person has to deposit a certain amount of OCT token in this contract, to indicate that he/she wants to delegate his/her voting rights to a certain `validator` of the appchain. It is defined as:

```rust
pub struct Delegator {
    /// The delegator's id in NEAR protocol.
    pub delegator_id: AccountId,
    /// The validator's id in NEAR protocol, which the delegator delegates his rights to.
    pub validator_id: AccountId,
    /// The block height when the delegator is registered.
    pub registered_block_height: BlockHeight,
    /// The timestamp when the delegator is registered.
    pub registered_timestamp: Timestamp,
}
```

* `validator set`: A set of validators and delegators of the corresponding appchain. It is defined as:

```rust
pub struct ValidatorSet<V, D> {
    /// The set of account id of validators.
    pub validator_ids: LazyOption<UnorderedSet<AccountId>>,
    /// The set of account id of delegators.
    pub delegator_ids: LazyOption<UnorderedSet<AccountId>>,
    /// The map from validator id to its delegators' ids.
    pub validator_id_to_delegator_ids: LookupMap<AccountId, UnorderedSet<AccountId>>,
    /// The map from delegator id to the validators' ids that
    /// the delegator delegates his/her voting rights to.
    pub delegator_id_to_validator_ids: LookupMap<AccountId, UnorderedSet<AccountId>>,
    /// The validators data, mapped by their account id in NEAR protocol.
    pub validators: LookupMap<AccountId, V>,
    /// The delegators data, mapped by the tuple of their delegator account id and
    /// validator account id in NEAR protocol.
    pub delegators: LookupMap<(AccountId, AccountId), D>,
}
```

* `OCT token`: The OCT token is used to stake for the validators of corresponding appchain. It is defined as:

```rust
pub struct OctToken {
    pub contract_account: AccountId,
    pub price_in_usd: U64,
    pub total_stake: Balance,
}
```

* `NEAR fungible token`: A token which is lived in NEAR protocol. It should be a NEP-141 compatible contract. This contract can bridge the token to the corresponding appchain. It is defined as:

```rust
pub enum BridgingState {
    /// The state which this contract is bridging the bridge token to the appchain.
    Active,
    /// The state which this contract has stopped bridging the bridge token to the appchain.
    Closed,
}

pub struct NearFungibleTokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
}

pub struct NearFungibleToken {
    pub metadata: NearFungibleTokenMetadata,
    pub contract_account: AccountId,
    pub price_in_usd: U64,
    /// The total balance locked in this contract
    pub locked_balance: Balance,
    pub bridging_state: BridgingState,
}
```

* `wrapped appchain token`: The wrapped token of the appchain native token, which is managed by a contract in NEAR protocol. It is defined as:

```rust
pub struct WrappedAppchainTokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub spec: String,
    pub icon: Option<Vec<u8>>,
    pub reference: Option<Vec<u8>>,
    pub reference_hash: Option<Vec<u8>>,
}

pub struct WrappedAppchainToken {
    pub metadata: WrappedAppchainTokenMetadata,
    pub contract_account: AccountId,
    pub initial_balance: Balance,
    pub changed_balance: I128,
    pub price_in_usd: U64,
}
```

* `token bridging history`: The token bridging fact happens in this contract. It is defined as:

```rust
pub enum TokenBridgingFact {
    /// The fact that a certain amount of wrapped appchain token is minted in its contract
    /// in NEAR protocol
    WrappedAppchainTokenMinted {
        request_id: String,
        /// The account id of receiver in NEAR protocol
        receiver_id: AccountId,
        amount: U128,
    },
    /// The fact that a certain amount of wrapped appchain token is burnt in its contract
    /// in NEAR protocol
    WrappedAppchainTokenBurnt {
        sender_id: AccountId,
        /// The id of receiver on the appchain
        receiver_id: String,
        amount: U128,
    },
    /// The fact that a certain amount of NEAR fungible token has been locked in appchain anchor.
    NearFungibleTokenLocked {
        symbol: String,
        /// The account id of sender in NEAR protocol
        sender_id: AccountId,
        /// The id of receiver on the appchain
        receiver_id: String,
        amount: U128,
    },
    /// The fact that a certain amount of NEAR fungible token has been unlocked and
    /// transfered from this contract to the receiver.
    NearFungibleTokenUnlocked {
        request_id: String,
        symbol: String,
        /// The account id of receiver in NEAR protocol
        receiver_id: AccountId,
        amount: U128,
    },
}

pub struct TokenBridgingHistory {
    pub token_bridging_fact: TokenBridgingFact,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub index: U64,
}
```

* `staking history`: The staking fact happens in this contract. It is defined as:

```rust
pub enum StakingFact {
    /// A new validator is registered in appchain anchor
    ValidatorAdded {
        /// The validator's id in NEAR protocol.
        validator_id_in_near: AccountId,
        /// The validator's id in the appchain.
        validator_id_in_appchain: AccountIdInAppchain,
        amount: U128,
    },
    /// A validator increases his stake in appchain anchor
    StakeIncreased {
        /// The validator's id in NEAR protocol.
        validator_id_in_near: AccountId,
        amount: U128,
    },
    /// A validator decreases his stake in appchain anchor
    StakeDecreased {
        /// The validator's id in NEAR protocol.
        validator_id_in_near: AccountId,
        amount: U128,
    },
    /// A new delegator is registered in appchain anchor
    DelegatorAdded {
        /// The delegator's id in NEAR protocol.
        delegator_id_in_near: AccountId,
        /// The delegator's id in the appchain.
        delegator_id_in_appchain: AccountIdInAppchain,
        /// The validator's id in NEAR protocol.
        validator_id_in_near: AccountId,
        amount: U128,
    },
    /// A delegator increases his delegation for a validator in appchain anchor
    DelegationIncreased {
        /// The delegator's id in NEAR protocol.
        delegator_id_in_near: AccountId,
        /// The validator's id in NEAR protocol.
        validator_id_in_near: AccountId,
        amount: U128,
    },
    /// A delegator decreases his delegation for a validator in appchain anchor
    DelegationDecreased {
        /// The delegator's id in NEAR protocol.
        delegator_id_in_near: AccountId,
        /// The validator's id in NEAR protocol.
        validator_id_in_near: AccountId,
        amount: U128,
    },
}

pub struct StakingHistory {
    pub staking_fact: StakingFact,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub index: U64,
}
```

* `appchain message`: The fact that happens on the corresponding appchain. It is defined as:

```rust
pub enum AppchainFact {
    /// The fact that a certain amount of bridge token has been burnt on the appchain.
    NearFungibleTokenBurnt { symbol: String, amount: U128 },
    /// The fact that a certain amount of appchain native token has been locked on the appchain.
    NativeTokenLocked { amount: U128 },
    /// The fact that a validator has been unbonded on the appchain.
    ValidatorUnbonded {
        validator_id: AccountIdInAppchain,
        block_height: BlockHeight,
        timestamp: Timestamp,
    },
    /// The fact that a delegator has been unbonded on the appchain.
    DelegatorUnbonded {
        delegator_id: AccountIdInAppchain,
        validator_id: AccountIdInAppchain,
        block_height: BlockHeight,
        timestamp: Timestamp,
    },
    /// The fact that the era is switched in the appchain
    EraSwitched {
        appchain_era_number: U64
    }
}

pub struct AppchainMessage {
    pub appchain_fact: AppchainFact,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub nonce: u32,
}
```

* `octopus relayer`: A standalone service which will relay the `appchain message` to this contract.
* `appchain settings`: A set of settings for booting corresponding appchain, which is defined as:

```rust
pub struct AppchainSettings {
    pub chain_spec: String,
    pub raw_chain_spec: String,
    pub boot_node: String,
    pub rpc_endpoint: String,
}
```

* `anchor settings`: A set of settings for current appchain anchor, which is defined as:

```rust
pub struct AnchorSettings {
    pub token_price_maintainer_account: AccountId,
}
```

* `protocol settings`: A set of settings for Octopus Network protocol, maintained by the `owner`, which is defined as:

```rust
pub struct ProtocolSettings {
    /// A validator has to deposit a certain amount of OCT token to this contract for
    /// being validator of the appchain.
    pub minimum_validator_deposit: Balance,
    /// The minimum deposit amount for a delegator to delegate his voting weight to
    /// a certain validator.
    pub minimum_delegator_deposit: Balance,
    /// The minimum value of total stake in this contract for booting corresponding appchain
    pub minimum_total_stake_for_booting: Balance,
    /// The maximum percentage of the total market value of all NEP-141 tokens to the total
    /// market value of OCT token staked in this contract
    pub maximum_market_value_percent_of_near_fungible_tokens: u16,
    /// The maximum percentage of the total market value of wrapped appchain token to the total
    /// market value of OCT token staked in this contract
    pub maximum_market_value_percent_of_wrapped_appchain_token: u16,
    /// The minimum number of validator(s) registered in this contract for
    /// booting the corresponding appchain and keep it alive.
    pub minimum_validator_count: u16,
    /// The maximum number of validator(s) which a delegator can delegate to.
    pub maximum_validators_per_delegator: u16,
    /// The unlock period (in days) for validator(s) can withdraw their deposit after
    /// they are removed from the corresponding appchain.
    pub unlock_period_of_validator_deposit: u16,
    /// The unlock period (in days) for delegator(s) can withdraw their deposit after
    /// they no longer delegates their stake to a certain validator on the corresponding appchain.
    pub unlock_period_of_delegator_deposit: u16,
    /// The maximum number of historical eras that the validators or delegators are allowed to
    /// withdraw their benefit
    pub maximum_era_count_of_delayed_benefit: u16,
}
```

* `sender`: A NEAR transaction sender, that is the account which perform actions (call functions) on this contract.

### Cross chain transfer in this contract

There are 2 kinds of cross chain assets transfer in this contract:

* wrapped appchain token transfer between appchain and NEAR
  * appchain:lock -> wrapped-appchain-token-contract@near:mint
  * wrapped-appchain-token-contract@near:burn -> appchain:unlock
* NEP141 asset (token) transfer between NEAR and appchain
  * near-fungible-token-contract@near:lock_asset -> appchain:mint_asset
  * appchain:burn_asset -> near-fungible-token-contract@near:unlock_asset

## Contract data design

The data fields of this contract is defined as:

```rust
pub struct AppchainAnchor {
    /// The id of corresponding appchain.
    pub appchain_id: AppchainId,
    /// The account id of appchain registry contract.
    pub appchain_registry_contract: AccountId,
    /// The info of OCT token.
    pub oct_token: LazyOption<OctToken>,
    /// The info of wrapped appchain token in NEAR protocol.
    pub wrapped_appchain_token: LazyOption<WrappedAppchainToken>,
    /// The set of symbols of NEP-141 tokens.
    pub near_fungible_token_symbols: UnorderedSet<String>,
    /// The NEP-141 tokens data, mapped by the symbol of the token.
    pub near_fungible_tokens: LookupMap<String, NearFungibleToken>,
    /// The history version of validator set, mapped by era number in appchain.
    pub validator_set_histories: LookupMap<u64, ValidatorSetOfEra>,
    /// The validator set of the next era in appchain.
    pub next_validator_set: ValidatorSetOfEra,
    /// The validator set for unbonded validators and delegators.
    pub unbonded_validator_set: ValidatorSet<UnbondedValidator, UnbondedDelegator>,
    /// The mapping for validators' accounts, from account id in the appchain to
    /// account id in NEAR protocol.
    pub validator_account_id_mapping: LookupMap<AccountIdInAppchain, AccountId>,
    /// The custom settings for appchain.
    pub appchain_settings: LazyOption<AppchainSettings>,
    /// The anchor settings for appchain.
    pub anchor_settings: LazyOption<AnchorSettings>,
    /// The protocol settings for appchain anchor.
    pub protocol_settings: LazyOption<ProtocolSettings>,
    /// The state of the corresponding appchain.
    pub appchain_state: AppchainState,
    /// The staking history data happened in this contract.
    pub staking_histories: LookupMap<u64, StakingHistory>,
    /// The start index of valid staking history in `staking_histories`.
    pub staking_history_start_index: u64,
    /// The end index of valid staking history in `staking_histories`.
    pub staking_history_end_index: u64,
    /// The token bridging history data happened in this contract.
    pub token_bridging_histories: LookupMap<u64, TokenBridgingHistory>,
    /// The start index of valid token bridging history in `token_bridging_histories`.
    pub token_bridging_history_start_index: u64,
    /// The end index of valid token bridging history in `token_bridging_histories`.
    pub token_bridging_history_end_index: u64,
}
```

Due to the relatively large amount of data volume in this contract, we use `LookupMap`, `LazyOption` to store data that will become larger. By doing this, we can reduce the gas consumption of deserialization of the struct in each function call.

Considering the possible huge amount of history data for `token bridging history` and `staking history`, we use `LookupMap` to store them. Then we can only store the start index and end index for valid history data in contract struct. If we want to clear some history data, we can simply specify a value of start index, and then delete all records with smaller index in the map of history data.

Due to the gas limit of the transaction (single function call) in NEAR protocol, we may not be able to complete the copy of the entire validator set in one transaction, so we choose to recover the complete validator set at a certain moment by applying all the staking history in the past. This may take multiple transactions to complete the entire process. To store history version of validator set and support the recover process in multiple transactions, we define `ValidatorSetOfEra` as:

```rust
pub struct ValidatorSetOfEra {
    /// The number of era in appchain.
    pub appchain_era_number: u64,
    /// The index of the latest staking history happened in the era of corresponding appchain
    pub staking_history_index: u64,
    /// The index of latest applied staking history
    pub applied_staking_history_index: u64,
    /// Total benefit of this era (which will be distributed to validators and delegators),
    /// in wrapped appchain token on NEAR protocol
    pub total_benefit: Balance,
    /// Total stake of this era
    pub total_stake: Balance,
    /// The set of validator id which will not be profited
    pub unprofitable_validator_ids: UnorderedSet<AccountId>,
    /// The validator set of this era
    pub validator_set: ValidatorSet<ValidatorOfEra, DelegatorOfEra>,
}
```

All changes to validator set which are caused by external users will be recorded in `self.staking_histories` and will be applied to `self.next_validator_set`.

When this contract receives `AppchainMessage` with `AppchainFact::EraSwitched`, the contract will create a history version of validator set at the time using `staking history`, and insert it into `self.validator_set_histories`. This process will cost at least 2 transactions (function calls), which are triggered by `octopus relayer`.

## Contract initialization

This contract has to be initialized by the following parameters:

* `appchain_id`: The id of an appchain which is bound to this contract.
* `appchain_registry_contract`: The account id of `appchain registry`.
* `oct_token_contract`: The account id of OCT token contract.

Processing steps:

* The `self.appchain_id` is set to `appchain_id`.
* The `self.appchain_registry_contract` is set to `appchain_registry_contract`.
* The `self.oct_token.contract_account` is set to `oct_token_contract`.
* Initialize `self.protocol_settings` by default values.
* Initialize other fields of this contract by default (empty) values.
* The `self.appchain_state` is set to `staging`.

## Manage NEAR fungible token

### Register NEAR fungible token

This action needs the following parameters:

* `symbol`: The symbol of the `NEAR fungible token`.
* `name`: The name of the `NEAR fungible token`.
* `decimals`: The decimals of the `NEAR fungible token`.
* `contract_account`: The account id of the `NEAR fungible token` contract.
* `price`: The price of the `NEAR fungible token`.
* `price_decimals`: The decimals of `price`.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must not be registered.

Processing steps:

* Create a new `NEAR fungible token` with input parameters.
* Insert the new `NEAR fungible token` into `self.near_fungible_tokens` using `symbol` as key.
* The default `bridging state` of the `NEAR fungible token` is `closed`.

### Set price of a NEAR fungible token

This action needs the following parameters:

* `symbol`: The symbol of the `NEAR fungible token`.
* `price`: The price of the `NEAR fungible token`.

Qualification of this action:

* The `sender` must be the `self.anchor_settings.token_price_maintainer_account`.
* The `symbol` must already be registered.

Processing steps:

* Get the `NEAR fungible token` from `self.near_fungible_tokens` by key `symbol`.
* The price of the `NEAR fungible token` is set to `price`.

### Open bridging for a NEAR fungible token

This action needs the following parameters:

* `symbol`: The symbol of the NEAR fungible token.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must already be registered.

Processing steps:

* Get the `NEAR fungible token` from `self.near_fungible_tokens` by key `symbol`.
* The `bridging state` of the `NEAR fungible token` is set to `active`.

### Close bridging for a NEAR fungible token

This action needs the following parameters:

* `symbol`: The symbol of the NEAR fungible token.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must already be registered.

Processing steps:

* Get the `NEAR fungible token` from `self.near_fungible_tokens` by key `symbol`.
* The `bridging state` of the `NEAR fungible token` is set to `closed`.

### Lock a certain amount of a NEAR fungible token

This action is performed in [Process fungible token deposit](#process-fungible-token-deposit).

This action needs the following parameters:

* `contract_account`: The account id of the contract of a NEAR fungible token.
* `sender_id`: The account id in NEAR protocol, which is the sender of the NEAR fungible token.
* `receiver_id`: The account id on the corresponding appchain for receiving the bridged token.
* `amount`: The amount of `NEAR fungible token` to lock.

Qualification of this action:

* This action can ONLY be performed inside this contract, or can ONLY be called by this contract.
* The `contract_account` must be equal to `contract_account` of a registered `NEAR fungible token`.
* The `bridging state` of the `NEAR fungible token` must be `open`.

Processing steps:

* Get the `NEAR fungible token` from `self.near_fungible_tokens` by `contract_account`.
* Add `amount` to `locked_balance` of the `NEAR fungible token`.
* Check the total market value of all `NEAR fungible token` locked in this contract:
  * Calculate the market value of each `NEAR fungible token` by `near_fungible_token.locked_balance * near_fungible_token.price_in_usd`.
  * If the total market value of all `NEAR fungible token` is bigger than `self.oct_token.total_stake * self.oct_token.price_in_usd * self.protocol_settings.maximum_market_value_percent_of_near_fungible_tokens / 100`, throws an error.
* Create a new `token bridging history` with fact `BridgeTokenLocked`, and insert it to `self.token_bridging_histories` by key `self.token_bridging_history_end_index + 1`.
* Add `1` to `self.token_bridging_history_end_index`.
* Generate log: `Token <symbol of NEAR fungible token> from <sender_id> locked. Receiver: <receiver_id>, Amount: <amount>`

### Unlock a certain amount of a NEAR fungible token

This action is performed when `AppchainFact::NearFungibleTokenBurnt` is received in [Handle relayed message](#handle-relayed-message).

This action needs the following parameters:

* `request_id`: The request id generated by the `sender`, which is used to identify the unlocking action.
* `symbol`: The symbol of a NEAR fungible token.
* `receiver_id`: The account id of receiver in NEAR protocol for `NEAR fungible token` which will be unlocked.
* `amount`: The amount of `NEAR fungible token` to unlock.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `symbol` must be the symbol of a registered `NEAR fungible token`.
* The `amount` must be less or equal to the `locked_balance` of the `NEAR fungible token` corresponding to `symbol`.

Processing Steps:

* Get the `NEAR fungible token` from `self.near_fungible_tokens` by key `symbol`.
* Reduce `amount` from `locked_balance` of the `NEAR fungible token`.
* Call function `ft_transfer` of `contract_account` of the `NEAR fungible token` with parameters `receiver_id` and `amount`:
  * If success:
    * Create a new `token bridging history` with fact `BridgeTokenUnlocked` and insert it into `self.token_bridging_histories` with key `self.token_bridging_history_end_index + 1`.
    * Add `1` to `self.token_bridging_history_end_index`.
    * Generate log: `Token <symbol> unlocked and transfered to <receiver_id>. Amount: <amount>`
  * If fail:
    * Generate log: `Failed to unlock and transfer token <symbol> to <receiver_id>. Amount: <amount>`

## Manage wrapped appchain token

The contract of `wrapped appchain token` in NEAR protocol should be deployed before the appchain go `active`. The owner of the token contract should be set to the owner of this contract. The initial total supply of `wrapped appchain token` should be minted to an account belongs to the appchain team.

### Set metadata of wrapped appchain token

This action needs the following parameters:

* `name`: The name of `wrapped appchain token`.
* `symbol`: The symbol of `wrapped appchain token`.
* `decimals`: The decimals of `wrapped appchain token`.
* `spec`: The specification of `wrapped appchain token`.
* `icon`: (Optional) The data of icon file of `wrapped appchain token`.
* `reference`: (Optional) The reference data of `wrapped appchain token`.
* `reference_hash`: (Optional) The hash of reference data of `wrapped appchain token`.

Qualification of this action:

* The `sender` must be the `owner`.
* The `self.appchain_state` must be `staging`.

These parameters are stored to `self.wrapped_appchain_token`.

### Set contract account of wrapped appchain token

This action needs the following parameters:

* `contract_account`: The account id of native token contract of the appchain.

Qualification of this action:

* The `sender` must be the `owner`.
* The `self.appchain_state` must be `staging` or `booting`.

The `self.wrapped_appchain_token.contract_account` is set to `contract_account`.

### Set initial balance of wrapped appchain token

This action needs the following parameters:

* `value`: The initial balance of wrapped appchain token contract in NEAR protocol.

Qualification of this action:

* The `sender` must be the `owner`.

The `self.wrapped_appchain_token.initial_balance` is set to `value`.

### Set price of wrapped appchain token

This action needs the following parameters:

* `price`: The price of the `wrapped appchain token`.

Qualification of this action:

* The `sender` must be the `self.anchor_settings.token_price_maintainer_account`.

Processing steps:

* The `self.wrapped_appchain_token.price_in_usd` is set to `price`.

### Mint wrapped appchain token

This action is performed when `AppchainFact::NativeTokenLocked` is received in [Handle relayed message](#handle-relayed-message).

This action needs the following parameters:

* `request_id`: The request id generated by the `sender`, which is used to identify the minting action.
* `receiver_id`: The account id of receiver of minting token in NEAR protocol.
* `amount`: The amount of wrapped appchain token to mint.

Qualification of this action:

* This action can ONLY be performed inside this contract.

Processing steps:

* Calculate the amount of OCT token which is equivalent to the total market value of total balance of wrapped appchain token as `equivalent amount`:

```rust
(self.wrapped_appchain_token.initial_balance + self.wrapped_appchain_token.changed_balance + amount) * self.wrapped_appchain_token.exchange_rate_to_oct_token / 10 ** self.wrapped_appchain_token.exchange_rate_decimals_to_oct_token
```

* Calculate the maximum amount of OCT token that is allowed to be minted in the contract of `wrapped appchain token` as `maximum amount to be minted`:

```rust
self.oct_token.total_stake * self.protocol_settings.maximum_market_value_percent_of_wrapped_appchain_token / 100
```

* If the `equivalent amount` is bigger than `maximum amount to be minted`, throws an error.
* Call function `mint` of `contract_account` of `self.wrapped_appchain_token` with params `receiver_id` and `amount`:
  * If success:
    * Create a new `token bridging history` with fact `AppchainNativeTokenMinted` and insert it into `self.token_bridging_histories` with key `self.token_bridging_history_end_index + 1`.
    * Add `1` to `self.token_bridging_history_end_index`.
    * Add `amount` to `self.wrapped_appchain_token.changed_balance`.
    * Generate log: `<appchain_id> native token minted to <receiver_id>. Amount: <amount>`
  * If fail:
    * Generate log: `Failed to mint <appchain_id> native token to <receiver_id>. Amount: <amount>`

### Burn wrapped appchain token

A owner of `wrapped appchain token` in NEAR protocol can perform this action to burn a certain amount of `wrapped appchain token`, and specify an account id to receive native token in appchain.

This action needs the following parameters:

* `receiver_id_in_appchain`: The account id of the receiver in appchain. The receiver will receive `amount` of native token in appchain after the `amount` of `wrapped appchain token` is burnt.
* `amount`: The amount of wrapped appchain token to burn.

Processing steps:

* Call function `burn` of `contract_account` of `self.wrapped_appchain_token` with params `sender` and `amount`:
  * If success:
    * Create a new `token bridging history` with fact `AppchainNativeTokenBurnt` and insert it into `self.token_bridging_histories` with key `self.token_bridging_history_end_index + 1`.
    * Add `1` to `self.token_bridging_history_end_index`.
    * Reduce `amount` from `self.wrapped_appchain_token.changed_balance`.
    * Generate log: `Wrapped <appchain_id> token burnt by <sender>. Appchain receiver: <receiver_id_in_appchain>, Amount: <amount>`
  * If fail:
    * Generate log: `Failed to burn <appchain_id> native token from <sender_id>. Amount: <amount>`

## Manage appchain settings

This contract has a set of functions to manage the value of each field of `appchain settings`.

## Manage anchor settings

This contract has a set of functions to manage the value of each field of `anchor settings`.

## Manage protocol settings

This contract has a set of functions to manage the value of each field of `protocol settings`.

## Process fungible token deposit

This contract has a callback interface `FungibleTokenReceiver::ft_on_transfer` for contracts like `fungible_token` of `near-contract-standards`.

The callback function `ft_on_transfer` needs the following parameters:

* `sender_id`: The account id of sender of the transfer.
* `amount`: The amount of the transfer.
* `msg`: The message attached to the transfer, which indicates the purpose of the deposit.

If the caller of this callback (`env::predecessor_account_id()`) equals to `self.oct_token.contract_account` of this contract, match `msg` with the following patterns:

* `register_reserved_validator,<validator_account_id_in_appchain>`: Perform [Register reserved validator](#register-reserved-validator).
* `register_validator,<validator_account_id_in_appchain>,<payee_account_id_in_appchain>`: Perform [Register validator](#register-validator).
* `increase_stake`: Perform [Increase stake of a validator](#increase-stake-of-a-validator).
* `register_delegator,<delegator_account_id_in_appchain>,<validator_account_id_in_near>`: Perform [Register delegator](#register-delegator).
* `increase_delegation,<validator_account_id_in_near>`: Perform [Increase delegation of a delegator](#increase-delegation-of-a-delegator).
* other cases:
  * The deposit will be considered as `invalid deposit`.

If the caller of this callback (`env::predecessor_account_id()`) equals to `contract_account` of a `NEAR fungible token` registered in this contract, match `msg` with the following patterns:

* `bridge_to,<receiver_id>`: Perform [Lock a certain amount of a NEAR fungible token](#lock-a-certain-amount-of-a-near-fungible-token).
* other cases:
  * The deposit will be considered as `invalid deposit`.

If the caller of this callback (`env::predecessor_account_id()`) is neither `self.oct_token_contract` nor `contract_account` of a `NEAR fungible token`, throws an error: `Invalid deposit of unknown NEP-141 asset`.

For `invalid deposit` case, throws an error: `Invalid deposit <amount> of OCT token from <sender_id>.`.

### Register validator

This action is performed in [Process fungible token deposit](#process-fungible-token-deposit).

This action needs the following parameters:

* `sender_id`: The new `validator`'s account id in NEAR protocol.
* `validator_id_in_appchain`: The `validator`'s account id in the corresponding appchain.
* `payee_id_in_appchain`: The account id in corresponding appchain for receiving the income of the validator in appchain.
* `amount`: The amount of the deposit.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `self.appchain_state` must be `staging`, `booting` or `active`.
* The `sender_id` must not be existed in `self.next_validator_set` as `validator_id_in_near`.
* The `sender_id` must not be existed in `self.unbonded_validator_set` as `validator_id_in_near`.
* The amount of deposit must not be smaller than `self.protocol_settings.minimum_validator_deposit`.

Processing steps:

* Create a new `validator` with following values:
  * `validator_id_in_near`: `sender_id`
  * `validator_id_in_appchain`: `validator_account_id_in_appchain`
  * `payee_id_in_appchain`: `payee_account_id_in_appchain`
  * `deposit_amount`: `amount`
  * `staking_state`: `StakingState::Active`
  * `staking_unlock_time`: `u64::MAX`
  * `can_accept_delegation`: `true`
* Add the new `validator` to `self.next_validator_set`.
* Create a new `staking history` with fact `ValidatorAdded` and insert it into `self.staking_histories` with key `self.staking_history_end_index + 1`.
* Add `1` to `self.staking_history_end_index`.
* Add `amount` to `self.oct_token.total_stake`.
* Generate log: `Validator <sender_id> is registered with stake <amount>.`

### Increase stake of a validator

This action is performed in [Process fungible token deposit](#process-fungible-token-deposit).

This action needs the following parameters:

* `sender_id`: The account id of a certain `validator` in NEAR protocol.
* `amount`: The amount of the deposit.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `self.appchain_state` must be `staging`, `booting` or `active`.
* The `sender_id` must be existed in `self.next_validator_set` as `validator_id_in_near`.
* The `sender_id` must not be existed in `self.unbonded_validator_set` as `validator_id_in_near`.

Processing steps:

* Add `amount` to the `deposit_amount` of the given `validator` in `self.next_validator_set`.
* Create a new `staking history` with fact `StakeIncreased` and insert it into `self.staking_histories` with key `self.staking_history_end_index + 1`.
* Add 1 to `self.staking_history_end_index`.
* Add `amount` to `self.oct_token.total_stake`.
* Generate log: `Stake of validator <sender_id> raised by <amount>.`

### Register delegator

This action is performed in [Process fungible token deposit](#process-fungible-token-deposit).

This action needs the following parameters:

* `sender_id`: The account id of the new `delegator` in NEAR protocol.
* `account_id_in_appchain`: The `delegator`'s account id in the corresponding appchain.
* `account_id`: The account id of a certain `validator` in NEAR protocol.
* `amount`: The amount of the deposit.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `self.appchain_state` must be `staging`, `booting` or `active`.
* The `account_id` as `validator_id_in_near` must be existed in `self.next_validator_set`.
* The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must not be existed in `self.next_validator_set`.
* The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must not be existed in `self.unbonded_validator_set`.
* The amount of deposit must not be smaller than `self.protocol_settings.minimum_delegator_deposit`.
* The value of `can_accept_delegation` of the `validator` corresponding to `account_id` in `self.next_validator_set` must be `true`.
* The count of `validator` of the `delegator` corresponding to `sender_id` in `self.next_validator_set` must be smaller than `self.protocol_settings.maximum_validators_per_delegator`.

Processing steps:

* Create a new `delegator` with following values:
  * `delegator_id_in_near`: `sender_id`
  * `delegator_id_in_appchain`: `account_id_in_appchain`
  * `validator_id_in_near`: `account_id`
  * `validator_id_in_appchain`: validator account id in appchain, get from corresponding validator set (depends on `self.appchain_state`)
  * `deposit_amount`: `amount`
  * `staking_state`: `StakingState::Active`
  * `staking_unlock_time`: `u64::MAX`
* Add the new `delegator` to `self.next_validator_set`.
* Create a new `staking history` with fact `DelegatorAdded` and insert it into `self.staking_histories` with key `self.staking_history_end_index + 1`.
* Add 1 to `self.staking_history_end_index`.
* Add `amount` to `self.oct_token.total_stake`.
* Generate log: `Delegator <sender_id> of validator <account_id> is registered with delegation <amount>.`

### Increase delegation of a delegator

This action is performed in [Process fungible token deposit](#process-fungible-token-deposit).

This action needs the following parameters:

* `sender_id`: The account id of the new `delegator` in NEAR protocol.
* `account_id`: The account id of a certain `validator` in NEAR protocol.
* `amount`: The amount of the deposit.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `self.appchain_state` must be `staging`, `booting` or `active`.
* The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must be existed in `self.next_validator_set`.
* The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must not be existed in `self.unbonded_validator_set`.

Processing steps:

* Add `amount` to `deposit_amount` of the `delegator` corresponding to pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) in `self.next_validator_set`.
* Create a new `staking history` with fact `DelegationIncreased` and insert it into `self.staking_histories` with key `self.staking_history_end_index + 1`.
* Add 1 to `self.staking_history_end_index`.
* Add `amount` to `self.oct_token.total_stake`.
* Generate log: `The delegation of delegator <sender_id> of validator <account_id> raised by <amount>.`

## Handle relayed message

This action needs the following parameters:

* `encoded_message`: The encoded fact data submitted by `octopus relayer`.
* `header_partial`: ?
* `leaf_proof`: ?
* `mmr_root`: ?

This action will verify the parameters by rule of light client of the appchain. If fail, throws an error.

Decode `encoded_message`, the real message will be one of `appchain message`:

* `NativeTokenLocked`: Which indicate that the appchain has locked a certain amount of `wrapped appchain token`.
  * Perform [Mint wrapped appchain token](#mint-wrapped-appchain-token).
* `NearFungibleTokenBurnt`: Which indicate that the appchain has burnt a certain amount of `NEAR fungible token`.
  * Perform [Unlock a certain amount of a NEAR fungible token](#unlock-a-certain-amount-of-a-near-fungible-token).
* `ValidatorUnbonded`: Which indicate that a validator has been unbonded on the appchain.
  * Perform [Unbond stake](#unbond-stake).
* `DelegatorUnbonded`: Which indicate that a delegator of a valicator has been unbonded on the appchain.
  * Perform [Unbond delegation](#unbond-delegation).
* `EraSwitched`: Which indicate that the era in the appchain has been switched.
  * Perform [Start applying staking history](#start-applying-staking-history).
* Other cases: throws an error.

## Manage appchain staking

### Unbond stake

This action is performed when `AppchainFact::ValidatorUnbonded` is received in [Handle relayed message](#handle-relayed-message).

This action needs the following parameters:

* `validator_id_in_appchain`: The account id of a certain `validator` in the appchain.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `validator_id_in_appchain` must be existed in `self.validator_account_id_mapping` as a key.

Processing steps:

* Get `validator_id_in_near` from `self.validator_account_id_mapping` using `validator_id_in_appchain` as key.
* Get `validator` data from `self.next_validator_set`.
* Remove `validator_id_in_near` from `self.next_validator_set`.
* The `staking state` of the `validator` is set to `unbonded`.
* Add the `validator` to `self.unbonded_validator_set`.
* Reduce the value of `deposit_amount` of the `validator` and all its `delegators` from `self.oct_token.total_stake`.
* The `staking_unlock_time` of the `validator` is set to `StakingState::Unbonded.timestamp + self.protocol_settings.unlock_period_of_validator_deposit * SECONDS_OF_A_DAY * NANO_SECONDS_MULTIPLE`.

### Decrease stake

This action needs the following parameters:

* `amount`: The amount of stake to decrease.

Qualification of this action:

* The `sender` must be one of the registered `validator`.
* The `self.appchain_state` must be `staging`, `booting` or `active`.
* The `sender_id` must be existed in `self.next_validator_set` as key.
* The `amount` must not be bigger than (`validator.deposit_amount` - `self.protocol_settings.minimum_validator_deposit`).

Processing steps:

* Reduce `amount` from `deposit_amount` of the given `validator` in `self.next_validator_set`.
* Create a new `staking history` with fact `StakeDecreased` and insert it into `self.staking_histories` with key `self.staking_history_end_index + 1`.
* Add 1 to `self.staking_history_end_index`.
* Reduce `amount` from `self.oct_token.total_stake`.
* Call function `ft_transfer` of `self.oct_token.contract_account` with parameters `sender` and `amount`:
  * If success:
    * Generate log: `Staking deposit of <sender> reduced by <amount>.`
  * If fail:
    * Generate log: `Failed to decrease staking deposit of <sender>. Amount: <amount>`

### Withdraw stake

Qualification of this action:

* The `sender` must be existed in `self.unbonded_validator_set` as `validator_id_in_near`.
* The days passed from `StakingState::Unbonded.timestamp` is bigger than `self.protocol_settings.unlock_period_of_validator_deposit`.

Processing steps:

* Get `validator` from `self.unbonded_validator_set` using `sender` as `validator_id_in_near`.
* Call function `ft_transfer` of `self.oct_token.contract_account` with parameters `sender` and `validator.deposit_amount`:
  * If success:
    * Generate log: `Staking deposit of <sender> is withdrawn. Amount: <amount>`
  * If fail:
    * Generate log: `Failed to withdraw staking deposit of <sender>. Amount: <amount>`
* Remove the `validator` from `self.unbonded_validator_set`.

### Unbond delegation

This action is performed when `AppchainFact::DelegatorUnbonded` is received in [Handle relayed message](#handle-relayed-message).

This action needs the following parameters:

* `delegator_id_in_appchain`: The account id of a certain `delegator` in the appchain.
* `validator_id_in_appchain`: The account id of a certain `validator` in the appchain.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `delegator_id_in_appchain` must be existed in `self.validator_account_id_mapping` as a key.
* The `validator_id_in_appchain` must be existed in `self.validator_account_id_mapping` as a key.

Processing steps:

* Get `delegator_id_in_near` from `self.validator_account_id_mapping` using `validator_id_in_appchain` as key.
* Get `validator_id_in_near` from `self.validator_account_id_mapping` using `validator_id_in_appchain` as key.
* Get the `delegator` data from `self.next_validator_set`.
* Remove the `delegator` from `self.next_validator_set`.
* The `staking state` of the `delegator` is set to `unbonded`.
* Add the `delegator` to `self.unbonded_validator_set`.
* Reduce the value of `deposit_amount` of the `delegator` from `self.oct_token.total_stake`.
* The `staking_unlock_time` of the `delegator` is set to `StakingState::Unbonded.timestamp + self.protocol_settings.unlock_period_of_delegator_deposit * SECONDS_OF_A_DAY * NANO_SECONDS_MULTIPLE`.

### Decrease delegation

This action needs the following parameters:

* `validator_id`: The account id of a certain `validator` in NEAR protocol.
* `amount`: The amount of to withdraw.

Qualification of this action:

* The pair of (`sender`, `validator_id`) must be one of the registered `delegator`.
* The `self.appchain_state` must be `staging`, `booting` or `active`.
* The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must be existed in `self.next_validator_set`.
* The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must not be existed in `self.unbonded_validator_set`.
* The `amount` must not be bigger than (`delegator.deposit_amount` - `self.protocol_settings.minimum_delegator_deposit`).

Processing steps:

* Reduce `amount` from `deposit_amount` of the `delegator` corresponding to pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) in `self.next_validator_set`.
* Create a new `staking history` with fact `DelegationIncreased` and insert it into `self.staking_histories` with key `self.staking_history_end_index + 1`.
* Add 1 to `self.staking_history_end_index`.
* Reduce `amount` from `self.oct_token.total_stake`.
* Call function `ft_transfer` of `self.oct_token.contract_account` with parameters `sender` and `amount`:
  * If success:
    * Generate log: `Delegating deposit of <sender> for <validator_id> reduced by <amount>.`
  * If fail:
    * Generate log: `Failed to decrease delegating deposit of <sender> for <validator_id>. Amount: <amount>`

### Withdraw delegation

This action needs the following parameters:

* `validator_id`: The account id of a certain `validator` in NEAR protocol.

Qualification of this action:

* The pair of (`sender`, `validator_id`) as (`delegator_id`, `validator_id`) must be existed in `self.unbonded_validator_set`.
* The days passed from `StakingState::Unbonded.timestamp` of the `delegator` is bigger than `self.protocol_settings.unlock_period_of_delegator_deposit`.

Processing steps:

* Get the `delegator` data from `self.unbonded_validator_set` using pair (`sender`, `validator_id`) as the key.
* Call function `ft_transfer` of `self.oct_token.contract_account` with parameters `sender` and `delegator.deposit_amount`:
  * If success:
    * Generate log: `Delegating deposit of <sender> for <validator_id> is withdrawn. Amount: <deposit_amount>`
  * If fail:
    * Generate log: `Failed to withdraw delegating deposit of <sender> for <validator_id>. Amount: <deposit_amount>`
* Remove the `delegator` from `self.unbonded_validator_set`.

### Start applying staking history

This action is performed when `AppchainFact::EraSwitched` is received in [Handle relayed message](#handle-relayed-message).

This action needs the following parameters:

* `appchain_era_number`: The era number in appchain.

Qualification of this action:

* This action can ONLY be performed inside this contract.

Processing steps:

* Create a new `ValidatorSetOfEra` with following values:
  * `appchain_era_number`: `appchain_era_number`
  * `staking_history_index`: `self.staking_history_end_index`
  * `applied_staking_history_index`: `0`
  * `validator_set`: a new (empty) `AppchainValidatorSet`
* Insert the new `ValidatorSetOfEra` into `self.validator_set_histories` using `appchain_era_number` as key.
* The `self.applying_appchain_era_number` is set to `appchain_era_number`.
* Perform [Apply staking history](#apply-staking-history).

### Apply staking history

Anyone can perform this action.

Processing steps:

* Get `ValidatorSetOfEra` from `self.validator_set_histories` by key `self.applying_appchain_era_number` as `validator_set`.
* Repeat the following process until `validator_set.applied_staking_history_index` is equal to `validator_set.staking_history_index` or the burnt gas reaches 180T (90% of gas limit per contract in a transaction of NEAR protocol):
  * Get `StakingHistory` from `self.staking_histories` by key `validator_set.applied_staking_history_index + 1` as `staking_history`.
  * Apply `staking_history` to `validator_set`.
  * Add `1` to `validator_set.applied_staking_history_index`.
* If `validator_set.applied_staking_history_index` is equal to `validator_set.staking_history_index` return `true`, else return `false`.

## Manage appchain lifecycle

### Go booting

Qualification of this action:

* The `sender` must be the `owner`.
* The `self.appchain_state` must be `staging`.
* The metadata of `wrapped appchain token` has already been set by [Set metadata of wrapped appchain token](#set-metadata-of-wrapped-appchain-token).
* If the `contract_account` of `warpped appchain token` must be set.
* The count of `validator` in `self.next_validator_set` must be not smaller than `self.protocol_settings.minimum_validator_count`.
* The `self.oct_token.total_stake` must be not smaller than `self.protocol_settings.minimum_total_stake_for_booting`.
* The `chain_spec`, `raw_chain_spec` and `boot_node` fields of `self.appchain_settings` must be set.

Processing steps:

* The `self.appchain_state` is set to `booting`.
* Sync `self.appchain_state` to `appchain registry`.

### Go live

Qualification of this action:

* The `sender` must be the `owner`.
* The `self.appchain_state` must be `booting`.

Processing steps:

* The `self.appchain_state` is set to `active`.
* Sync `self.appchain_state` to `appchain registry`.
