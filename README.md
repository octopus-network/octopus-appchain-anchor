# octopus-appchain-anchor

This contract provides an anchor for an appchain of [Octopus Network](https://oct.network). It is in charge of managing the necessary data of an appchain in NEAR protocol, , providing security and interoperability for the appchain.

Each appchain of Octopus Network will be bonded to an instance of this contract, which is deployed to a subaccount of [octopus-appchain-registry](https://github.com/octopus-network/octopus-appchain-registry).

Contents

* [Terminology](#terminology)
  * [Cross chain transfer in this contract](#cross-chain-transfer-in-this-contract)
* [Contract data design](#contract-data-design)
* [Contract initialization](#contract-initialization)
* [Manage bridge token](#manage-bridge-token)
  * [Register bridge token](#register-birdge-token)
  * [Set price of a bridge token](#set-price-of-a-bridge-token)
  * [Open bridging for a bridge token](#open-bridging-for-a-bridge-token)
  * [Close bridging for a bridge token](#close-bridging-for-a-bridge-token)
  * [Lock a certain amount of a bridge token](#lock-a-certain-amount-of-a-bridge-token)
  * [Unlock a certain amount of a bridge token](#unlock-a-certain-amount-of-a-bridge-token)
* [Manage appchain native token](#manage-appchain-native-token)
  * [Set metadata of appchain native token](#set-metadata-of-appchain-native-token)
  * [Set contract account of appchain native token](#set-contract-account-of-appchain-native-token)
  * [Stage code of appchain native token contract](#stage-code-of-appchain-native-token-contract)
  * [Set price of appchain native token](#set-price-of-appchain-native-token)
  * [Mint appchain native token](#mint-appchain-native-token)
  * [Burn appchain native token](#burn-appchain-native-token)
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
* [Manage appchain lifecycle](#manage-appchain-lifecycle)
  * [Go booting](#go-booting)
  * [Go live](#go-live)

## Terminology

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
    /// The state which an appchain is broken for some technical or governance reasons.
    Broken,
    /// The state which the lifecycle of an appchain is end.
    Dead,
}
```

* `staking state`: The staking state of a validator or delegator of the appchain. It is defined as:

```rust
pub enum StakingState {
    /// Active in staking on corresponding appchain.
    Active,
    /// Has been unbonded from staking on corresponding appchain.
    Unbonded,
}
```

* `validator`: A person who wants to act as a validator on the appchain corresponding to this contract. The person has to deposit a certain amount of OCT token in this contract. It is defined as:

```rust
pub struct AppchainValidator {
    /// Account id of appchain validator.
    pub validator_id: AccountId,
    /// Staked balance of the validator.
    pub deposit_amount: Balance,
    /// Staking state of the validator.
    pub staking_state: StakingState,
}
```

* `delegator`: A person who wants to act as a delegator on the appchain corresponding to this contract. The person has to deposit a certain amount of OCT token in this contract, to indicate that he/she wants to delegate his/her voting rights to a certain `validator` of the appchain. It is defined as:

```rust
pub struct AppchainDelegator {
    /// Account id of appchain delegator.
    pub delegator_id: AccountId,
    /// Account id of a validator which the delegator delegates his rights to.
    pub validator_id: AccountId,
    /// Delegated balance of the delegator.
    pub deposit_amount: Balance,
    /// Staking state of the delegator.
    pub staking_state: StakingState,
}
```

* `sender`: A NEAR transaction sender, that is the account which perform actions (call functions) on this contract.
* `bridge token`: A token which is lived in NEAR protocol. It should be a NEP-141 compatible contract. This contract can bridge the token to the corresponding appchain. It is defined as:

```rust
pub enum BridgingState {
    /// The state which this contract is bridging the bridge token to the appchain.
    Active,
    /// The state which this contract has stopped bridging the bridge token to the appchain.
    Closed,
}

pub struct BridgeTokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub price_decimals: u8,
}

pub struct BridgeToken {
    pub metadata: BridgeTokenMetadata,
    pub contract_account: AccountId,
    pub price: U64,
    pub price_decimals: u8,
    /// The total balance locked in this contract
    pub locked_balance: Balance,
    pub bridging_state: BridgingState,
}
```

* `appchain native token`: The token issued natively on the appchain. It is defined as:

```rust
pub struct AppchainNativeTokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub spec: String,
    pub icon: Option<Vec<u8>>,
    pub reference: Option<Vec<u8>>,
    pub reference_hash: Option<Vec<u8>>,
}

pub struct AppchainNativeToken {
    pub metadata: AppchainNativeTokenMetadata,
    pub contract_account: AccountId,
    pub price: U64,
    pub price_decimals: u8,
}
```

* `anchor fact`: The fact that happens in this contract. It is defined as:

```rust
pub enum AnchorFact {
    /// The fact that a certain amount of appchain native token is minted in its contract
    /// in NEAR protocol
    AppchainNativeTokenMinted {
        request_id: String,
        /// The account id of receiver in NEAR protocol
        receiver_id: AccountId,
        amount: U128,
    },
    /// The fact that a certain amount of appchain native token is burnt in its contract
    /// in NEAR protocol
    AppchainNativeTokenBurnt {
        sender_id: AccountId,
        /// The id of receiver on the appchain
        receiver_id: String,
        amount: U128,
    },
    /// The fact that a certain amount of bridge token has been locked in appchain anchor.
    BridgeTokenLocked {
        symbol: String,
        sender_id: AccountId,
        /// The id of receiver on the appchain
        receiver_id: String,
        amount: U128,
    },
    /// The fact that a certain amount of bridge token has been unlocked and
    /// transfered from this contract to the receiver.
    BridgeTokenUnlocked {
        request_id: String,
        symbol: String,
        /// The account id of receiver in NEAR protocol
        receiver_id: AccountId,
        amount: U128,
    },
    /// A new validator is registered in appchain anchor
    ValidatorAdded {
        validator_id: AccountId,
        amount: U128,
    },
    /// A validator increases his stake in appchain anchor
    StakeIncreased {
        validator_id: AccountId,
        amount: U128,
    },
    /// A validator decreases his stake in appchain anchor
    StakeDecreased {
        validator_id: AccountId,
        amount: U128,
    },
    /// A new delegator is registered in appchain anchor
    DelegatorAdded {
        delegator_id: AccountId,
        validator_id: AccountId,
        amount: U128,
    },
    /// A delegator increases his delegation for a validator in appchain anchor
    DelegationIncreased {
        delegator_id: AccountId,
        validator_id: AccountId,
        amount: U128,
    },
    /// A delegator decreases his delegation for a validator in appchain anchor
    DelegationDecreased {
        delegator_id: AccountId,
        validator_id: AccountId,
        amount: U128,
    },
}

pub struct AnchorFactRecord {
    pub anchor_fact: AnchorFact,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
}
```

* `appchain fact`: The fact that happens on the corresponding appchain. It is defined as:

```rust
pub enum AppchainFact {
    /// The fact that a certain amount of bridge token has been burnt on the appchain.
    BridgeTokenBurnt {
        symbol: String,
        amount: U128,
    },
    /// The fact that a certain amount of appchain native token has been locked on the appchain.
    NativeTokenLocked {
        amount: U128,
    },
    /// The fact that a validator has been unbonded on the appchain.
    ValidatorUnbonded {
        validator_id: AccountId,
        set_id: u32,
    },
    /// The fact that a delegator has been unbonded on the appchain.
    DelegatorUnbonded {
        delegator_id: AccountId,
        validator_id: AccountId,
        set_id: u32,
    }
}

pub struct AppchainFactRecord {
    pub appchain_fact: AppchainFact,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
}
```

* `octopus relayer`: A standalone service which will relay the `appchain fact` to this contract.
* `protocol settings`: A set of settings for Octopus Network protocol, maintained by the `owner`, which is defined as:

```rust
pub struct ProtocolSettings {
    /// A validator has to deposit a certain amount of OCT token to this contract for
    /// being validator of the appchain.
    pub minimum_validator_deposit: Balance,
    /// The minimum deposit amount for a delegator to delegate his voting weight to
    /// a certain validator.
    pub minimum_delegator_deposit: Balance,
    /// The minimum number of validator(s) registered in this contract for
    /// booting the corresponding appchain and keep it alive.
    pub minimum_validator_count: U64,
    /// The maximum number of validator(s) which a delegator can delegate to.
    pub maximum_validators_per_delegator: U64,
    /// The time duration for updating validator set based on recent deposit actions
    /// happened in this contract.
    pub validator_set_duration: U64,
    /// The unlock period for validator(s) can withdraw their deposit after they are removed from
    /// the corresponding appchain.
    pub unlock_period_of_validator_deposit: U64,
    /// The unlock period for delegator(s) can withdraw their deposit after they no longer
    /// delegates their stake to a certain validator on the corresponding appchain.
    pub unlock_period_of_delegator_deposit: U64,
}
```

### Cross chain transfer in this contract

There are 2 kinds of cross chain assets transfer in this contract:

* appchain native token transfer between appchain and NEAR
  * appchain:lock -> appchain-native-token-contract@near:mint
  * appchain-native-token-contract@near:burn -> appchain:unlock
* NEP141 asset (token) transfer between NEAR and appchain
  * bridge-token-contract@near:lock_asset -> appchain:mint_asset
  * appchain:burn_asset -> bridge-token-contract@near:unlock_asset

## Contract data design

The data fields of this contract is defined as:

```rust
pub struct AppchainAnchor {
    /// The id of corresponding appchain.
    pub appchain_id: AppchainId,
    /// The account id of appchain registry contract.
    pub appchain_registry_contract: AccountId,
    /// The account id of OCT token contract.
    pub oct_token_contract: AccountId,
    /// The native token of appchain.
    pub appchain_native_token: AppchainNativeToken,
    /// The bridge tokens data, mapped by the symbol of the token.
    pub bridge_tokens: UnorderedMap<String, BridgeToken>,
    /// The protocol settings for appchain anchor
    pub protocol_settings: ProtocolSettings,
    /// The state of the corresponding appchain
    pub appchain_state: AppchainState,
    /// The start index of anchor facts stored in the storage of this contract.
    pub anchor_fact_start_index: U64,
    /// The end index of anchor facts stored in the storage of this contract.
    pub anchor_fact_end_index: U64,
    /// The start index of appchain facts stored in the storage of this contract.
    pub appchain_fact_start_index: U64,
    /// The end index of appchain facts stored in the storage of this contract.
    pub appchain_fact_end_index: U64,
}
```

Considering the possible huge amount of history data for `anchor fact` and `appchain fact`, we do not define data fields for them in this contract struct. We'll use sdk function `env::storage_write` and `env::storage_read` to operate `AnchorFactRecord` and `AppchainFactRecord` in contract storage directly. By using this mechanism, we can only store the start index and end index for the facts data in contract struct. If we want to clear some history data, we can simply delete them by function `env::storage_remove` and update the start index in contract struct. The storage key used in these operations should be a specific string followed by a global sequence number. For example, use `acf_<number>` for `anchor fact` and `apf_<number>` for `appchain fact`.

## Contract initialization

This contract has to be initialized by the following parameters:

* `appchain_id`: The id of an appchain which is bound to this contract.
* `appchain_registry_contract`: The account id of `appchain registry`.
* `oct_token_contract`: The account id of OCT token contract.

Processing steps:

* Store the above parameters in this contract.
* Construct `protocol settings` by default values.
* The `anchor_fact_start_index`, `anchor_fact_end_index`, `appchain_fact_start_index` and `appchain_fact_end_index` are all set to `0`.
* The `appchain_state` is set to `staging`.

## Manage bridge token

### Register bridge token

This action needs the following parameters:

* `symbol`: The symbol of the `bridge token`.
* `name`: The name of the `bridge token`.
* `decimals`: The decimals of the `bridge token`.
* `contract_account`: The account id of the `bridge token` contract.
* `price`: The price of the `bridge token`.
* `price_decimals`: The decimals of `price`.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must not be registered.

Processing steps:

* Store these parameters as a `bridge token` to `bridge_tokens` in this contract, mapped by `symbol`.
* The default `bridging state` of the `bridge token` is `closed`.

### Set price of a bridge token

This action needs the following parameters:

* `symbol`: The symbol of the bridge token.
* `price`: The price of the `bridge token`.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must already be registered.

The price of the `bridge token` corresponding to `symbol` in this contract is set to `price`.

### Open bridging for a bridge token

This action needs the following parameters:

* `symbol`: The symbol of the bridge token.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must already be registered.

The `bridging state` of the given `bridge token` in this contract is set to `active`.

### Close bridging for a bridge token

This action needs the following parameters:

* `symbol`: The symbol of the bridge token.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must already be registered.

The `bridging state` of the given `bridge token` in this contract is set to `closed`.

### Lock a certain amount of a bridge token

This action needs the following parameters:

* `contract_account`: The account id of the contract of a bridge token.
* `sender_id`: The account id in NEAR protocol, which is the sender of the bridge token.
* `receiver_id`: The account id on the corresponding appchain for receiving the bridged token.
* `amount`: The amount of `bridge token` to lock.

Qualification of this action:

* This action can ONLY be performed inside this contract, or can ONLY be called by this contract.
* The `contract_account` must be equal to `contract_account` of a registered `bridge token`.

Processing steps:

* Add `amount` to `locked_balance` of the `bridge token`.
* Create a new `anchor fact` with type `BridgeTokenLocked`, and store it as an `AnchorFactRecord`.
* Generate log: `Token <symbol of bridge token> from <sender_id> locked. Receiver: <receiver_id>, Amount: <amount>`

### Unlock a certain amount of a bridge token

This action needs the following parameters:

* `request_id`: The request id generated by the `sender`, which is used to identify the unlocking action.
* `symbol`: The symbol of a bridge token.
* `receiver_id`: The account id of receiver in NEAR protocol for `bridge token` which will be unlocked.
* `amount`: The amount of `bridge token` to unlock.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `symbol` must be the symbol of a registered `bridge token`.
* The `amount` must be less or equal to the `locked_balance` of the `bridge token` corresponding to `symbol`.

Processing Steps:

* Reduce `amount` from `locked_balance` of the `bridge token`.
* Call function `ft_transfer` of `contract_account` of the `bridge token` with parameters `receiver_id` and `amount`:
  * If success:
    * Create a new `anchor fact` with type `BridgeTokenUnlocked`, and store it as an `AnchorFactRecord`.
    * Generate log: `Token <symbol> unlocked and transfered to <receiver_id>. Amount: <amount>`
  * If fail:
    * Generate log: `Failed to unlock and transfer token <symbol> to <receiver_id>. Amount: <amount>`

## Manage appchain native token

The contract of `appchain native token` in NEAR protocol can be applied by 2 ways:

* Deploy before this contract is deployed. In this case, the `owner` should set the contract account manually, before the appchain go `active`.
* Deploy by this contract automatically. In this case, the `owner` should stage code of contract of `appchain native token` before the appchain go `booting`.

### Set metadata of appchain native token

This action needs the following parameters:

* `name`: The name of `appchain native token`.
* `symbol`: The symbol of `appchain native token`.
* `decimals`: The decimals of `appchain native token`.
* `spec`: The specification of `appchain native token`.
* `icon`: (Optional) The data of icon file of `appchain native token`.
* `reference`: (Optional) The reference data of `appchain native token`.
* `reference_hash`: (Optional) The hash of reference data of `appchain native token`.

Qualification of this action:

* The `sender` must be the `owner`.
* The `appchain state` must be `staging`.

These parameters are stored to `appchain_native_token` of this contract. These are used when [Go booting](#go-booting).

### Set contract account of appchain native token

This action needs the following parameters:

* `contract_account`: The account id of native token contract of the appchain.

Qualification of this action:

* The `sender` must be the `owner`.
* The `appchain state` must be `staging` or `booting`.

Store the `contract_account` to `appchain_native_token` of this contract.

### Stage code of appchain native token contract

This action needs the following parameters:

* `code`: The wasm code of native token contract of the appchain.

Qualification of this action:

* The `sender` must be the `owner`.
* The `appchain state` must be `staging`.

The `code` is stored in this contract, it is used when [Go booting](#go-booting).

> Octopus Network provides [a standard implementation](https://github.com/octopus-network/appchain-native-token) of `appchain native token` contact.

### Set price of appchain native token

This action needs the following parameters:

* `price`: The price of the `appchain native token`.

Qualification of this action:

* The `sender` must be the `owner`.
* The `appchain state` must be `booting` or `active`.

The price of `appchain natvie token` in this contract is set to `price`.

### Mint appchain native token

This action needs the following parameters:

* `request_id`: The request id generated by the `sender`, which is used to identify the minting action.
* `receiver_id`: The account id of receiver of minting token in NEAR protocol.
* `amount`: The amount of appchain native token to mint.

Qualification of this action:

* This action can ONLY be performed inside this contract.

Processing steps:

* Call function `mint` of `contract_account` of `appchain_native_token` of this contract with `receiver_id` and `amount`:
  * If success:
    * Create a new `anchor fact` with the type `AppchainNativeTokenMinted`, and store it as an `AnchorFactRecord`.
    * Generate log: `<appchain_id> native token minted to <receiver_id>. Amount: <amount>`
  * If fail:
    * Generate log: `Failed to mint <appchain_id> native token to <receiver_id>. Amount: <amount>`

### Burn appchain native token

This action needs the following parameters:

* `receiver_id`: The account id of receiver on the appchain. The receiver should receive a certain amount (which is equals to `amount`) of appchain native token.
* `amount`: The amount of appchain native token to burn.

Processing steps:

* Call function `burn` of `contract_account` of `appchain_native_token` of this contract with `sender` and `amount`:
  * If success:
    * Create a new `anchor fact` with the type `AppchainNativeTokenBurnt`, and store it as an `AnchorFactRecord`.
    * Generate log: `<appchain_id> native token burnt by <sender_id>. Appchain receiver: <receiver_id>, Amount: <amount>`
  * If fail:
    * Generate log: `Failed to burn <appchain_id> native token from <sender_id>. Amount: <amount>`

## Manage protocol settings

This contract has a set of functions to manage the value of each field of `protocol settings`.

## Process fungible token deposit

This contract has a callback interface `FungibleTokenReceiver::ft_on_transfer` for contracts like `fungible_token` of `near-contract-standards`.

The callback function `ft_on_transfer` needs the following parameters:

* `sender_id`: The account id of sender of the transfer.
* `amount`: The amount of the transfer.
* `msg`: The message attached to the transfer, which indicates the purpose of the deposit.

If the caller of this callback (`env::predecessor_account_id()`) equals to `oct_token_contract` of this contract, match `msg` with the following patterns:

* `register_validator`: Perform [Register validator](#register-validator).
* `increase_stake`: Perform [Increase stake of a validator](#increase-stake-of-a-validator).
* `register_delegator,<account_id>`: Perform [Register delegator](#register-delegator).
* `increase_delegation,<account_id>`: Perform [Increase delegation](#increase-delegation).
* other cases:
  * The deposit will be considered as `invalid deposit`.

If the caller of this callback (`env::predecessor_account_id()`) equals to `contract_account` of a `bridge token` registered in this contract, match `msg` with the following patterns:

* `bridge_to,<receiver_id>`: Perform [Lock a certain amount of a bridge token](#lock-a-certain-amount-of-a-bridge-token).
* other cases:
  * The deposit will be considered as `invalid deposit`.

If the caller of this callback (`env::predecessor_account_id()`) is neither `oct_token_contract` nor `contract_account` of a `bridge token`, throws an error: `Invalid deposit of unknown NEP-141 asset`.

For `invalid deposit` case, throws an error: `Invalid deposit <amount> of OCT token from <sender_id>.`.

### Register validator

This action needs the following parameters:

* `sender_id`: The account id of the new `validator`.
* `amount`: The amount of the deposit.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `appchain state` must not be `broken` or `dead`.
* The amount of deposit must not be smaller than `minimum_validator_deposit` of `protocol settings`.

Processing steps:

* Create a new `validator` with following values:
  * `validator_id`: `sender_id`
  * `deposit_amount`: `amount`
  * `staking_state`: `StakingState::Active`
* Create a new `anchor fact` with type `ValidatorAdded`, and store it as an `AnchorFactRecord`.
* Generate log: `Validator <sender_id> is registered with stake <amount>.`

### Increase stake of a validator

This action needs the following parameters:

* `sender_id`: The account id of a certain `validator`.
* `amount`: The amount of the deposit.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `appchain state` must not be `broken` or `dead`.
* The `sender_id` must be equal to the `validator_id` of a registered `validator`.

Processing steps:

* Add `amount` to the `deposit_amount` of the given `validator`.
* Create a new `anchor fact` with type `StakeIncreased`, and store it as an `AnchorFactRecord`.
* Generate log: `Stake of validator <sender_id> raised by <amount>.`

### Register delegator

This action needs the following parameters:

* `sender_id`: The account id of the new `delegator`.
* `account_id`: The account id of a certain `validator`.
* `amount`: The amount of the deposit.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `appchain state` must not be `broken` or `dead`.
* The `account_id` must be equal to `validator_id` of a registered `validator`.
* The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must not have been registered in this contract.
* The amount of deposit must not be smaller than `minimum_delegator_deposit` of `protocol settings`.

Processing steps:

* Create a new `delegator` with following values:
  * `delegator_id`: `sender_id`
  * `validator_id`: `account_id`
  * `deposit_amount`: `amount`
  * `staking_state`: `StakingState::Active`
* Create a new `anchor fact` with type `DelegatorAdded`, and store it as an `AnchorFactRecord`.
* Generate log: `Delegator <sender_id> of validator <account_id> is registered with delegation <amount>.`

### Increase delegation of a delegator

This action needs the following parameters:

* `sender_id`: The account id of the new `delegator`.
* `account_id`: The account id of a certain `validator`.
* `amount`: The amount of the deposit.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `appchain state` must not be `broken` or `dead`.
* The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must have been registered in this contract.

Processing steps:

* Add `amount` to `deposit_amount` of the `delegator` corresponding to pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`).
* Create a new `anchor fact` with type `DelegationIncreased`, and store it as an `AnchorFactRecord`.
* Generate log: `The delegation of delegator <sender_id> of validator <account_id> raised by <amount>.`

## Handle relayed message

This action needs the following parameters:

* `encoded_messages`: The encoded fact data submitted by `octopus relayer`.
* `header_partial`: ?
* `leaf_proof`: ?
* `mmr_root`: ?

This action will verify the parameters by rule of light client of the appchain. If fail, throws an error.

Decode `encoded_messages`, the real message will be one of `appchain fact`:

* `NativeTokenLocked`: Which indicate that the appchain has locked a certain amount of `appchain native token`.
  * Store this `appchain fact` as a `AppchainFactRecord`.
  * Perform [Mint appchain native token](#mint-appchain-native-token).
* `BridgeTokenBurnt`: Which indicate that the appchain has burnt a certain amount of `bridge token`.
  * Store this `appchain fact` as a `AppchainFactRecord`.
  * Perform [Unlock a certain amount of a bridge token](#unlock-a-certain-amount-of-a-bridge-token).
* `ValidatorUnbonded`: Which indicate that a validator has been unbonded on the appchain.
  * Store this `appchain fact` as a `AppchainFactRecord`.
  * Perform [Unbond stake](#unbond-stake).
* `DelegatorUnbonded`: Which indicate that a delegator of a valicator has been unbonded on the appchain.
  * Store this `appchain fact` as a `AppchainFactRecord`.
  * Perform [Unbond delegation](#unbond-delegation).
* Other cases: throws an error.

## Manage appchain staking

### Unbond stake

This action needs the following parameters:

* `validator_id`: The account id of a certain `validator`.

Qualification of this action:

* This action can ONLY be performed inside this contract.

The `staking state` for the `validator` corresponding to `validator_id` is set to `unbonded`.

### Decrease stake

This action needs the following parameters:

* `amount`: The amount of stake to decrease.

Qualification of this action:

* The `sender` must be one of the registered `validator`.
* The `amount` must not be bigger than (`validator.deposit_amount` - `protocol_settings.minimum_validator_deposit`).

Processing steps:

* Reduce `amount` from `deposit_amount` of the given `validator`.
* Call function `ft_transfer` of `oct_token_contract` with parameters `sender` and `amount`:
  * If success:
    * Generate log: `Staking deposit of <sender> reduced by <amount>.`
  * If fail:
    * Generate log: `Failed to decrease staking deposit of <sender>. Amount: <amount>`

### Withdraw stake

Qualification of this action:

* The `sender` must be one of the registered `validator`.
* The `staking_state` of the given `validator` must be `unbonded`.

Processing steps:

* Call function `ft_transfer` of `oct_token_contract` with parameters `sender` and `validator.deposit_amount`:
  * If success:
    * Generate log: `Staking deposit of <sender> is withdrawed. Amount: <amount>`
  * If fail:
    * Generate log: `Failed to withdraw staking deposit of <sender>. Amount: <amount>`

### Unbond delegation

This action needs the following parameters:

* `delegator_id`: The account id of a certain `delegator`.
* `validator_id`: The account id of a certain `validator`.

Qualification of this action:

* This action can ONLY be performed inside this contract.

The `staking state` of the `delegator` corresponding to `delegator_id` and `validator_id` is set to `unbonded`.

### Decrease delegation

This action needs the following parameters:

* `validator_id`: The account id of a certain `validator`.
* `amount`: The amount of to withdraw.

Qualification of this action:

* The pair of (`sender`, `validator_id`) must be one of the registered `delegator`.
* The `amount` must not be bigger than (`delegator.deposit_amount` - `protocol_settings.minimum_delegator_deposit`).

Processing steps:

* Reduce `amount` from `deposit_amount` of the given `delegator`.
* Call function `ft_transfer` of `oct_token_contract` with parameters `sender` and `amount`:
  * If success:
    * Generate log: `Delegating deposit of <sender> for <validator_id> reduced by <amount>.`
  * If fail:
    * Generate log: `Failed to decrease delegating deposit of <sender> for <validator_id>. Amount: <amount>`

### Withdraw delegation

This action needs the following parameters:

* `validator_id`: The account id of a certain `validator`.

Qualification of this action:

* The pair of (`sender`, `validator_id`) must be one of the registered `delegator`.
* The `staking_state` of the `delegator` corresponding to (`sender`, `validator_id`) must be `unbonded`.

Processing steps:

* Call function `ft_transfer` of `oct_token_contract` with parameters `sender` and `delegator.deposit_amount`:
  * If success:
    * Generate log: `Delegating deposit of <sender> for <validator_id> is withdrawed. Amount: <deposit_amount>`
  * If fail:
    * Generate log: `Failed to withdraw delegating deposit of <sender> for <validator_id>. Amount: <deposit_amount>`

## Manage appchain lifecycle

### Go booting

Qualification of this action:

* The `sender` must be the `owner`.
* The `appchain state` must be `staging`.
* The metadata of `appchain native token` has already been set by [Set metadata of appchain native token](#set-metadata-of-appchain-native-token).
* If the `contract_account` of `appchain_native_token` is NOT set, the code of contract of `appchain native token` must have already been staged by [Stage code of appchain natvie token contract](#stage-code-of-appchain-native-token-contract).

Processing steps:

* The `appchain state` is set to `booting`.
* If the `contract_account` of `appchain_native_token` is NOT set, deploy and initialize the contract of `appchain native token`:
  * Create subaccount `token.<account id of this contract>`.
  * Transfer a certain amount of NEAR token to account `token.<account id of this contract>` for storage deposit.
  * Set `contract_account` of `appchain_native_token` to `token.<account id of this contract>`.
  * Deploy the code of contract of `appchain native token` to account `token.<account id of this contract>`.
  * Create a new full access key of the deployed contract for this contract.
  * Call function `new` of the deployed contract with the metadata of `appchain_native_token` of this contract.
* Sync `appchain state` to `appchain registry`.

### Go live

Qualification of this action:

* The `sender` must be the `owner`.
* The `appchain state` must be `booting`.

Processing steps:

* The `appchain state` is set to `active`.
* Store currently registered validators and delegators as `appchain fact` with type `update validator set` in this contract.
* Sync `appchain state` to `appchain registry`.
