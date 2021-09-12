# octopus-appchain-anchor

This contract provides an anchor for an appchain of [Octopus Network](https://oct.network). It is in charge of managing the necessary data of an appchain in NEAR protocol, , providing security and interoperability for the appchain.

Each appchain of Octopus Network will be bonded to an instance of this contract, which is deployed to a subaccount of [octopus-appchain-registry](https://github.com/octopus-network/octopus-appchain-registry).

Contents

* [Terminology](#terminology)
  * [Cross chain transfer in this contract](#cross-chain-transfer-in-this-contract)
* [Contract data design](#contract-data-design)
* [Contract initialization](#contract-initialization)
* [Manage NEP-141 token](#manage-nep-141-token)
  * [Register NEP-141 token](#register-nep-141-token)
  * [Set price of a NEP-141 token](#set-price-of-a-nep-141-token)
  * [Open bridging for a NEP-141 token](#open-bridging-for-a-nep-141-token)
  * [Close bridging for a NEP-141 token](#close-bridging-for-a-nep-141-token)
  * [Lock a certain amount of a NEP-141 token](#lock-a-certain-amount-of-a-nep-141-token)
  * [Unlock a certain amount of a NEP-141 token](#unlock-a-certain-amount-of-a-nep-141-token)
* [Manage wrapped appchain token](#manage-wrapped-appchain-token)
  * [Set metadata of wrapped appchain token](#set-metadata-of-wrapped-appchain-token)
  * [Set contract account of wrapped appchain token](#set-contract-account-of-wrapped-appchain-token)
  * [Set price of wrapped appchain token](#set-price-of-wrapped-appchain-token)
  * [Mint wrapped appchain token](#mint-wrapped-appchain-token)
  * [Burn wrapped appchain token](#burn-wrapped-appchain-token)
* [Manage protocol settings](#manage-protocol-settings)
* [Process fungible token deposit](#process-fungible-token-deposit)
  * [Register reserved validator](#register-reserved-validator)
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
    /// The state while an appchain is under challenging, which all deposit and withdraw actions
    /// are frozen.
    Frozen,
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
    Active {
        block_height: BlockHeight,
        timestamp: Timestamp,
    },
    /// Has been unbonded from staking on corresponding appchain.
    Unbonded {
        block_height: BlockHeight,
        timestamp: Timestamp,
    },
}
```

* `account id in appchain`: The account id in the appchain, which is usually the public key of an account in the appchain. The id is bonded to an account id in NEAR protocol in this contract.

```rust
pub type AccountIdInAppchain = String;
```

* `validator`: A person who wants to act as a validator on the appchain corresponding to this contract. The person has to deposit a certain amount of OCT token in this contract. It is defined as:

```rust
pub struct AppchainValidator {
    /// The validator's id in NEAR protocol.
    pub validator_id_in_near: AccountId,
    /// The validator's id in the appchain.
    pub validator_id_in_appchain: AccountIdInAppchain,
    /// The account id in the appchain for receiving income of the validator in appchain.
    pub payee_id_in_appchain: AccountIdInAppchain,
    /// Staked balance of the validator.
    pub deposit_amount: Balance,
    /// Staking state of the validator.
    pub staking_state: StakingState,
    /// Whether the validator is reserved.
    /// The reserved validator can NOT be delegated to.
    pub is_reserved: bool,
}
```

* `delegator`: A person who wants to act as a delegator in the corresponding appchain. The person has to deposit a certain amount of OCT token in this contract, to indicate that he/she wants to delegate his/her voting rights to a certain `validator` of the appchain. It is defined as:

```rust
pub struct AppchainDelegator {
    /// The delegator's id in NEAR protocol.
    pub delegator_id_in_near: AccountId,
    /// The delegator's id in the appchain.
    pub delegator_id_in_appchain: AccountIdInAppchain,
    /// The validator's id in NEAR protocol, which the delegator delegates his rights to.
    pub validator_id_in_near: AccountId,
    /// The validator's id in the appchain, which the delegator delegates his rights to.
    pub validator_id_in_appchain: AccountIdInAppchain,
    /// Delegated balance of the delegator.
    pub deposit_amount: Balance,
    /// Staking state of the delegator.
    pub staking_state: StakingState,
}
```

* `validator set`: A set of validators and delegators of the corresponding appchain. It is defined as:

```rust
pub struct AppchainValidatorSet {
    /// The set of account id of validators.
    pub validator_ids: LazyOption<UnorderedSet<AccountId>>,
    /// The set of account id of delegators.
    pub delegator_ids: LazyOption<UnorderedSet<AccountId>>,
    /// The validators that a delegator delegates his/her voting rights to.
    pub validator_ids_of_delegator_id: LookupMap<AccountId, UnorderedSet<AccountId>>,
    /// The validators data, mapped by their account id in NEAR protocol.
    pub validators: LookupMap<AccountId, AppchainValidator>,
    /// The delegators data, mapped by the tuple of their delegator account id and
    /// validator account id in NEAR protocol.
    pub delegators: LookupMap<(AccountId, AccountId), AppchainDelegator>,
}
```

* `sender`: A NEAR transaction sender, that is the account which perform actions (call functions) on this contract.
* `NEP-141 token`: A token which is lived in NEAR protocol. It should be a NEP-141 compatible contract. This contract can bridge the token to the corresponding appchain. It is defined as:

```rust
pub enum BridgingState {
    /// The state which this contract is bridging the NEP-141 token to the appchain.
    Active,
    /// The state which this contract has stopped bridging the NEP-141 token to the appchain.
    Closed,
}

pub struct Nep141TokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
}

pub struct Nep141Token {
    pub metadata: Nep141TokenMetadata,
    pub contract_account: AccountId,
    pub price: U64,
    pub price_decimals: u8,
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
    pub price: U64,
    pub price_decimals: u8,
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
    /// The fact that a certain amount of NEP-141 token has been locked in appchain anchor.
    Nep141TokenLocked {
        symbol: String,
        /// The account id of sender in NEAR protocol
        sender_id: AccountId,
        /// The id of receiver on the appchain
        receiver_id: String,
        amount: U128,
    },
    /// The fact that a certain amount of NEP-141 token has been unlocked and
    /// transfered from this contract to the receiver.
    Nep141TokenUnlocked {
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
    Nep141TokenBurnt { symbol: String, amount: U128 },
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
    /// The minimum number of validator(s) registered in this contract for
    /// booting the corresponding appchain and keep it alive.
    pub minimum_validator_count: U64,
    /// The maximum number of validator(s) which a delegator can delegate to.
    pub maximum_validators_per_delegator: U64,
    /// The unlock period (in days) for validator(s) can withdraw their deposit after
    /// they are removed from the corresponding appchain.
    pub unlock_period_of_validator_deposit: u16,
    /// The unlock period (in days) for delegator(s) can withdraw their deposit after
    /// they no longer delegates their stake to a certain validator on the corresponding appchain.
    pub unlock_period_of_delegator_deposit: u16,
}
```

### Cross chain transfer in this contract

There are 2 kinds of cross chain assets transfer in this contract:

* wrapped appchain token transfer between appchain and NEAR
  * appchain:lock -> wrapped-appchain-token-contract@near:mint
  * wrapped-appchain-token-contract@near:burn -> appchain:unlock
* NEP141 asset (token) transfer between NEAR and appchain
  * nep-141-token-contract@near:lock_asset -> appchain:mint_asset
  * appchain:burn_asset -> nep-141-token-contract@near:unlock_asset

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
    /// The wrapped appchain token in NEAR protocol.
    pub wrapped_appchain_token: WrappedAppchainToken,
    /// The set of symbols of NEP-141 tokens.
    pub nep141_token_symbols: UnorderedSet<String>,
    /// The NEP-141 tokens data, mapped by the symbol of the token.
    pub nep141_tokens: LookupMap<String, Nep141Token>,
    /// The currently used validator set in appchain
    pub current_validator_set: TaggedAppchainValidatorSet,
    /// The validator set of the next era in appchain
    pub next_validator_set: TaggedAppchainValidatorSet,
    /// The validator set for unbonded validators and delegators
    pub unbonded_validator_set: AppchainValidatorSet,
    /// The mapping for validators' accounts, from account id in the appchain to
    /// account id in NEAR protocol
    pub validator_account_id_mapping: LookupMap<AccountIdInAppchain, AccountId>,
    /// The protocol settings for appchain anchor
    pub protocol_settings: LazyOption<ProtocolSettings>,
    /// The state of the corresponding appchain
    pub appchain_state: AppchainState,
    /// The current total stake of all validators and delegators in this contract.
    pub total_stake: Balance,
    /// The staking history data happened in this contract
    pub staking_histories: LookupMap<u64, StakingHistory>,
    /// The start index of valid staking history in `staking_histories`.
    pub staking_history_start_index: u64,
    /// The end index of valid staking history in `staking_histories`.
    pub staking_history_end_index: u64,
    /// The token bridging history data happened in this contract
    pub token_bridging_histories: LookupMap<u64, TokenBridgingHistory>,
    /// The start index of valid token bridging history in `token_bridging_histories`.
    pub token_bridging_history_start_index: u64,
    /// The end index of valid token bridging history in `token_bridging_histories`.
    pub token_bridging_history_end_index: u64,
}
```

Due to the relatively large amount of data volume in this contract, we use `LookupMap`, `LazyOption` to store data that will become larger. By doing this, we can reduce the gas consumption of deserialization of the struct in each function call.

Considering the possible huge amount of history data for `token bridging history` and `staking history`, we use `LookupMap` to store them. Then we can only store the start index and end index for valid history data in contract struct. If we want to clear some history data, we can simply specify a value of start index, and then delete all records with smaller index in the map of history data.

As the event of switching era is happening in the appchain, we have to store 2 versions of validator set. One for current era in the appchain, another is for the next era in the appchain. And for tracing the changes between the 2 adjacent eras in appchain, we define `TaggedAppchainValidatorSet` for these 2 validator set:

```rust
pub struct TaggedAppchainValidatorSet {
    /// The number of era in appchain.
    pub appchain_era_number: u64,
    /// The index of the latest staking history happened in the era of corresponding appchain
    pub staking_history_index: u64,
    /// The index of latest applied staking history
    pub applied_staking_history_index: u64,
    /// The validator set for tagging
    pub validator_set: AppchainValidatorSet,
}
```

All changes to validator set which are caused by external users will be recorded in `self.staking_histories`.

While the `self.appchain_state` is `staging` or `booting`, all changes to validator set will be applied to `self.current_validator_set` directly. After the appchain goes to `active`, all changes to validator set will be applied to `self.next_validator_set`.

When this contract receives `AppchainMessage` with `AppchainFact::EraSwitched`, the contract will start applying the records of staking histories with the index between `current_validator_set.applied_staking_history_index` and `current_validator_set.staking_history_index` to `self.current_validator_set`. (This is to make `self.current_validator_set` to be equal to `self.next_validator_set`.) This process will cost 2 transactions (function calls) at least, which are triggered by `octopus relayer`.

## Contract initialization

This contract has to be initialized by the following parameters:

* `appchain_id`: The id of an appchain which is bound to this contract.
* `appchain_registry_contract`: The account id of `appchain registry`.
* `oct_token_contract`: The account id of OCT token contract.

Processing steps:

* Store the above parameters in this contract.
* Initialize `self.protocol_settings` by default values.
* The `self.appchain_state` is set to `staging`.

## Manage NEP-141 token

### Register NEP-141 token

This action needs the following parameters:

* `symbol`: The symbol of the `NEP-141 token`.
* `name`: The name of the `NEP-141 token`.
* `decimals`: The decimals of the `NEP-141 token`.
* `contract_account`: The account id of the `NEP-141 token` contract.
* `price`: The price of the `NEP-141 token`.
* `price_decimals`: The decimals of `price`.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must not be registered.

Processing steps:

* Store these parameters as a `NEP-141 token` to `self.nep141_tokens` using `symbol` as key.
* The default `bridging state` of the `NEP-141 token` is `closed`.

### Set price of a NEP-141 token

This action needs the following parameters:

* `symbol`: The symbol of the `NEP-141 token`.
* `price`: The price of the `NEP-141 token`.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must already be registered.

Processing steps:

* Get the `NEP-141 token` from `self.nep141_tokens` by key `symbol`.
* The price of the `NEP-141 token` is set to `price`.

### Open bridging for a NEP-141 token

This action needs the following parameters:

* `symbol`: The symbol of the NEP-141 token.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must already be registered.

Processing steps:

* Get the `NEP-141 token` from `self.nep141_tokens` by key `symbol`.
* The `bridging state` of the `NEP-141 token` is set to `active`.

### Close bridging for a NEP-141 token

This action needs the following parameters:

* `symbol`: The symbol of the NEP-141 token.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must already be registered.

Processing steps:

* Get the `NEP-141 token` from `self.nep141_tokens` by key `symbol`.
* The `bridging state` of the `NEP-141 token` is set to `closed`.

### Lock a certain amount of a NEP-141 token

This action needs the following parameters:

* `contract_account`: The account id of the contract of a NEP-141 token.
* `sender_id`: The account id in NEAR protocol, which is the sender of the NEP-141 token.
* `receiver_id`: The account id on the corresponding appchain for receiving the bridged token.
* `amount`: The amount of `NEP-141 token` to lock.

Qualification of this action:

* This action can ONLY be performed inside this contract, or can ONLY be called by this contract.
* The `contract_account` must be equal to `contract_account` of a registered `NEP-141 token`.

Processing steps:

* Get the `NEP-141 token` from `self.nep141_tokens` by `contract_account`.
* Add `amount` to `locked_balance` of the `NEP-141 token`.
* Create a new `token bridging history` with fact `BridgeTokenLocked`, and insert it to `self.token_bridging_histories` by key `self.token_bridging_history_end_index + 1`.
* Add `1` to `self.token_bridging_history_end_index`.
* Generate log: `Token <symbol of NEP-141 token> from <sender_id> locked. Receiver: <receiver_id>, Amount: <amount>`

### Unlock a certain amount of a NEP-141 token

This action needs the following parameters:

* `request_id`: The request id generated by the `sender`, which is used to identify the unlocking action.
* `symbol`: The symbol of a NEP-141 token.
* `receiver_id`: The account id of receiver in NEAR protocol for `NEP-141 token` which will be unlocked.
* `amount`: The amount of `NEP-141 token` to unlock.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `symbol` must be the symbol of a registered `NEP-141 token`.
* The `amount` must be less or equal to the `locked_balance` of the `NEP-141 token` corresponding to `symbol`.

Processing Steps:

* Get the `NEP-141 token` from `self.nep141_tokens` by key `symbol`.
* Reduce `amount` from `locked_balance` of the `NEP-141 token`.
* Call function `ft_transfer` of `contract_account` of the `NEP-141 token` with parameters `receiver_id` and `amount`:
  * If success:
    * Create a new `token bridging history` with fact `BridgeTokenUnlocked` and insert it into `self.token_bridging_histories` with key `self.token_bridging_history_end_index + 1`.
    * Add `1` to `self.token_bridging_history_end_index`.
    * Generate log: `Token <symbol> unlocked and transfered to <receiver_id>. Amount: <amount>`
  * If fail:
    * Generate log: `Failed to unlock and transfer token <symbol> to <receiver_id>. Amount: <amount>`

## Manage wrapped appchain token

The contract of `wrapped appchain token` in NEAR protocol should be deployed before the appchain go `active`. And the owner of the token contract should be set to the owner of this contract.

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

Store the `contract_account` to `self.wrapped_appchain_token`.

### Set price of wrapped appchain token

This action needs the following parameters:

* `price`: The price of the `wrapped appchain token`.

Qualification of this action:

* The `sender` must be the `owner`.
* The `self.appchain_state` must be `booting` or `active`.

The price of `self.wrapped_appchain_token` is set to `price`.

### Mint wrapped appchain token

This action needs the following parameters:

* `request_id`: The request id generated by the `sender`, which is used to identify the minting action.
* `receiver_id`: The account id of receiver of minting token in NEAR protocol.
* `amount`: The amount of wrapped appchain token to mint.

Qualification of this action:

* This action can ONLY be performed inside this contract.

Processing steps:

* Call function `mint` of `contract_account` of `self.wrapped_appchain_token` with params `receiver_id` and `amount`:
  * If success:
    * Create a new `token bridging history` with fact `AppchainNativeTokenMinted` and insert it into `self.token_bridging_histories` with key `self.token_bridging_history_end_index + 1`.
    * Add `1` to `self.token_bridging_history_end_index`.
    * Generate log: `<appchain_id> native token minted to <receiver_id>. Amount: <amount>`
  * If fail:
    * Generate log: `Failed to mint <appchain_id> native token to <receiver_id>. Amount: <amount>`

### Burn wrapped appchain token

This action needs the following parameters:

* `receiver_id`: The account id of receiver on the appchain. The receiver should receive a certain amount (which is equals to `amount`) of wrapped appchain token.
* `amount`: The amount of wrapped appchain token to burn.

Processing steps:

* Call function `burn` of `contract_account` of `self.wrapped_appchain_token` with params `sender` and `amount`:
  * If success:
    * Create a new `token bridging history` with fact `AppchainNativeTokenBurnt` and insert it into `self.token_bridging_histories` with key `self.token_bridging_history_end_index + 1`.
    * Add `1` to `self.token_bridging_history_end_index`.
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

* `register_reserved_validator,<validator_account_id_in_appchain>`: Perform [Register reserved validator](#register-reserved-validator).
* `register_validator,<validator_account_id_in_appchain>,<payee_account_id_in_appchain>`: Perform [Register validator](#register-validator).
* `increase_stake`: Perform [Increase stake of a validator](#increase-stake-of-a-validator).
* `register_delegator,<delegator_account_id_in_appchain>,<validator_account_id_in_near>`: Perform [Register delegator](#register-delegator).
* `increase_delegation,<validator_account_id_in_near>`: Perform [Increase delegation](#increase-delegation).
* other cases:
  * The deposit will be considered as `invalid deposit`.

If the caller of this callback (`env::predecessor_account_id()`) equals to `contract_account` of a `NEP-141 token` registered in this contract, match `msg` with the following patterns:

* `bridge_to,<receiver_id>`: Perform [Lock a certain amount of a NEP-141 token](#lock-a-certain-amount-of-a-nep-141-token).
* other cases:
  * The deposit will be considered as `invalid deposit`.

If the caller of this callback (`env::predecessor_account_id()`) is neither `self.oct_token_contract` nor `contract_account` of a `NEP-141 token`, throws an error: `Invalid deposit of unknown NEP-141 asset`.

For `invalid deposit` case, throws an error: `Invalid deposit <amount> of OCT token from <sender_id>.`.

### Register reserved validator

This action needs the following parameters:

* `sender_id`: The new `validator`'s account id in NEAR protocol.
* `validator_id_in_appchain`: The `validator`'s account id in the corresponding appchain.
* `amount`: The amount of the deposit.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `self.appchain_state` must be `staging`, `booting`.
* The `sender_id` must not be existed in `self.current_validator_set` as `validator_id_in_near`.
* The amount of deposit must not be smaller than `self.protocol_settings.minimum_validator_deposit`.

Processing steps:

* Create a new `validator` with following values:
  * `validator_id_in_near`: `sender_id`
  * `validator_id_in_appchain`: `account_id_in_appchain`
  * `payee_id_in_appchain`: empty string
  * `deposit_amount`: `amount`
  * `staking_state`: `StakingState::Active`
  * `is_reserved`: true
* Add the new `validator` to `self.current_validator_set`.
* Create a new `staking history` with fact `ValidatorAdded` and insert it into `self.staking_histories` with key `self.staking_history_end_index + 1`.
* Add `1` to `self.staking_history_end_index`.
* Add `amount` to `self.total_stake`.
* Generate log: `Validator <sender_id> is registered with stake <amount>.`

### Register validator

This action needs the following parameters:

* `sender_id`: The new `validator`'s account id in NEAR protocol.
* `validator_id_in_appchain`: The `validator`'s account id in the corresponding appchain.
* `payee_id_in_appchain`: The account id in corresponding appchain for receiving the income of the validator in appchain.
* `amount`: The amount of the deposit.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `self.appchain_state` must be `staging`, `booting` or `active`.
* If `self.appchain_state` is `staging` or `booting`, the `sender_id` must not be existed in `self.current_validator_set` as key.
* If `self.appchain_state` is `active`:
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
  * `is_reserved`: false
* If `self.appchain_state` is `staging` or `booting`, add the new `validator` to `self.current_validator_set`.
* If `self.appchain_state` is `active`, add the new `validator` to `self.next_validator_set`.
* Create a new `staking history` with fact `ValidatorAdded` and insert it into `self.staking_histories` with key `self.staking_history_end_index + 1`.
* Add `1` to `self.staking_history_end_index`.
* Add `amount` to `self.total_stake`.
* Generate log: `Validator <sender_id> is registered with stake <amount>.`

### Increase stake of a validator

This action needs the following parameters:

* `sender_id`: The account id of a certain `validator` in NEAR protocol.
* `amount`: The amount of the deposit.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `self.appchain_state` must be `staging`, `booting` or `active`.
* If `self.appchain_state` is `staging` or `booting`, the `sender_id` must be existed in `self.current_validator_set` as key.
* If `self.appchain_state` is `active`:
  * The `sender_id` must be existed in `self.next_validator_set` as `validator_id_in_near`.
  * The `sender_id` must not be existed in `self.unbonded_validator_set` as `validator_id_in_near`.

Processing steps:

* If `self.appchain_state` is `staging` or `booting`, add `amount` to the `deposit_amount` of the given `validator` in `self.current_validator_set`.
* If `self.appchain_state` is `active`, add `amount` to the `deposit_amount` of the given `validator` in `self.next_validator_set`.
* Create a new `staking history` with fact `StakeIncreased` and insert it into `self.staking_histories` with key `self.staking_history_end_index + 1`.
* Add 1 to `self.staking_history_end_index`.
* Add `amount` to `self.total_stake`.
* Generate log: `Stake of validator <sender_id> raised by <amount>.`

### Register delegator

This action needs the following parameters:

* `sender_id`: The account id of the new `delegator` in NEAR protocol.
* `account_id_in_appchain`: The `delegator`'s account id in the corresponding appchain.
* `account_id`: The account id of a certain `validator` in NEAR protocol.
* `amount`: The amount of the deposit.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `self.appchain_state` must be `staging`, `booting` or `active`.
* If `self.appchain_state` is `staging`, `booting`:
  * The `account_id` as `validator_id_in_near` must be existed in `self.current_validator_set`.
  * The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must not be existed in `self.current_validator_set`.
* If `self.appchain_state` is `active`:
  * The `account_id` as `validator_id_in_near` must be existed in `self.next_validator_set`.
  * The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must not be existed in `self.next_validator_set`.
  * The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must not be existed in `self.unbonded_validator_set`.
* The amount of deposit must not be smaller than `self.protocol_settings.minimum_delegator_deposit`.
* The value of `is_reserved` of the `validator` corresponding to `account_id` in `self.next_validator_set` must be `false`.
* The count of `validator` of the `delegator` corresponding to `sender_id` in `self.next_validator_set` must be smaller than `self.protocol_settings.maximum_validators_per_delegator`.

Processing steps:

* Create a new `delegator` with following values:
  * `delegator_id_in_near`: `sender_id`
  * `delegator_id_in_appchain`: `account_id_in_appchain`
  * `validator_id_in_near`: `account_id`
  * `validator_id_in_appchain`: validator account id in appchain, get from corresponding validator set (depends on `self.appchain_state`)
  * `deposit_amount`: `amount`
  * `staking_state`: `StakingState::Active`
* If `self.appchain_state` is `staging` or `booting`, add the new `delegator` to `self.current_validator_set`.
* If `self.appchain_state` is `active`, add the new `delegator` to `self.next_validator_set`.
* Create a new `staking history` with fact `DelegatorAdded` and insert it into `self.staking_histories` with key `self.staking_history_end_index + 1`.
* Add 1 to `self.staking_history_end_index`.
* Add `amount` to `self.total_stake`.
* Generate log: `Delegator <sender_id> of validator <account_id> is registered with delegation <amount>.`

### Increase delegation of a delegator

This action needs the following parameters:

* `sender_id`: The account id of the new `delegator` in NEAR protocol.
* `account_id`: The account id of a certain `validator` in NEAR protocol.
* `amount`: The amount of the deposit.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `self.appchain_state` must be `staging`, `booting` or `active`.
* If `self.appchain_state` is `staging`, `booting`:
  * The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must be existed in `self.current_validator_set`.
* If `self.appchain_state` is `active`:
  * The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must be existed in `self.next_validator_set`.
  * The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must not be existed in `self.unbonded_validator_set`.

Processing steps:

* If `self.appchain_state` is `staging` or `booting`, add `amount` to `deposit_amount` of the `delegator` corresponding to pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) in `self.current_validator_set`.
* If `self.appchain_state` is `active`, add `amount` to `deposit_amount` of the `delegator` corresponding to pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) in `self.next_validator_set`.
* Create a new `staking history` with fact `DelegationIncreased` and insert it into `self.staking_histories` with key `self.staking_history_end_index + 1`.
* Add 1 to `self.staking_history_end_index`.
* Add `amount` to `self.total_stake`.
* Generate log: `The delegation of delegator <sender_id> of validator <account_id> raised by <amount>.`

## Handle relayed message

This action needs the following parameters:

* `encoded_messages`: The encoded fact data submitted by `octopus relayer`.
* `header_partial`: ?
* `leaf_proof`: ?
* `mmr_root`: ?

This action will verify the parameters by rule of light client of the appchain. If fail, throws an error.

Decode `encoded_messages`, the real message will be one of `appchain message`:

* `NativeTokenLocked`: Which indicate that the appchain has locked a certain amount of `wrapped appchain token`.
  * Perform [Mint wrapped appchain token](#mint-wrapped-appchain-token).
* `BridgeTokenBurnt`: Which indicate that the appchain has burnt a certain amount of `NEP-141 token`.
  * Perform [Unlock a certain amount of a NEP-141 token](#unlock-a-certain-amount-of-a-nep-141-token).
* `ValidatorUnbonded`: Which indicate that a validator has been unbonded on the appchain.
  * Perform [Unbond stake](#unbond-stake).
* `DelegatorUnbonded`: Which indicate that a delegator of a valicator has been unbonded on the appchain.
  * Perform [Unbond delegation](#unbond-delegation).
* `EraSwitched`: Which indicate that the era in the appchain has been switched.
  * Perform [Start applying staking history](#start-applying-staking-history).
* Other cases: throws an error.

## Manage appchain staking

### Unbond stake

This action needs the following parameters:

* `validator_id_in_appchain`: The account id of a certain `validator` in the appchain.

Qualification of this action:

* This action can ONLY be performed inside this contract.
* The `validator_id_in_appchain` must be existed in `self.validator_account_id_mapping` as a key.

Processing steps:

* Get `validator_id_in_near` from `self.validator_account_id_mapping` using `validator_id_in_appchain` as key.
* If `validator_id_in_near` is existed in `self.next_validator_set`, get `validator` data from `self.next_validator_set`. If not, get `validator` data from `self.current_validator_set`.
* If the value of `is_reserved` of the `validator` is `true`, throws an error.
* Remove `validator_id_in_near` from `self.current_validator_set` and `self.next_validator_set`.
* The `staking state` of the `validator` is set to `unbonded`.
* Add the `validator` to `self.unbonded_validator_set`.
* Reduce the value of `deposit_amount` of the `validator` and all its `delegators` from `self.total_stake`.

### Decrease stake

This action needs the following parameters:

* `amount`: The amount of stake to decrease.

Qualification of this action:

* The `sender` must be one of the registered `validator`.
* The `self.appchain_state` must be `staging`, `booting` or `active`.
* If `self.appchain_state` is `staging` or `booting`, the `sender_id` must be existed in `self.current_validator_set` as key.
* If `self.appchain_state` is `active`, the `sender_id` must be existed in `self.next_validator_set` as key.
* The `amount` must not be bigger than (`validator.deposit_amount` - `self.protocol_settings.minimum_validator_deposit`).

Processing steps:

* If `self.appchain_state` is `staging` or `booting`, reduce `amount` from `deposit_amount` of the given `validator` in `self.current_validator_set`.
* If `self.appchain_state` is `active`, reduce `amount` from `deposit_amount` of the given `validator` in `self.next_validator_set`.
* Create a new `staking history` with fact `StakeDecreased` and insert it into `self.staking_histories` with key `self.staking_history_end_index + 1`.
* Add 1 to `self.staking_history_end_index`.
* Reduce `amount` from `self.total_stake`.
* Call function `ft_transfer` of `oct_token_contract` with parameters `sender` and `amount`:
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
* Call function `ft_transfer` of `oct_token_contract` with parameters `sender` and `validator.deposit_amount`:
  * If success:
    * Generate log: `Staking deposit of <sender> is withdrawed. Amount: <amount>`
  * If fail:
    * Generate log: `Failed to withdraw staking deposit of <sender>. Amount: <amount>`
* Remove the `validator` from `self.unbonded_validator_set`.

### Unbond delegation

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
* If the pair (`delegator_id_in_near`, `validator_id_in_near`) is existed in `self.next_validator_set`, get the `delegator` data from `self.next_validator_set`. If not, get the `delegator` data from `self.current_validator_set`.
* Remove the `delegator` from `self.current_validator_set` and `self.next_validator_set`.
* The `staking state` of the `delegator` is set to `unbonded`.
* Add the `delegator` to `self.unbonded_validator_set`.
* Reduce the value of `deposit_amount` of the `delegator` from `self.total_stake`.

### Decrease delegation

This action needs the following parameters:

* `validator_id`: The account id of a certain `validator` in NEAR protocol.
* `amount`: The amount of to withdraw.

Qualification of this action:

* The pair of (`sender`, `validator_id`) must be one of the registered `delegator`.
* The `self.appchain_state` must be `staging`, `booting` or `active`.
* If `self.appchain_state` is `staging`, `booting`:
  * The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must be existed in `self.current_validator_set`.
* If `self.appchain_state` is `active`:
  * The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must be existed in `self.next_validator_set`.
  * The pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) must not be existed in `self.unbonded_validator_set`.
* The `amount` must not be bigger than (`delegator.deposit_amount` - `self.protocol_settings.minimum_delegator_deposit`).

Processing steps:

* If `self.appchain_state` is `staging` or `booting`, reduce `amount` from `deposit_amount` of the `delegator` corresponding to pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) in `self.current_validator_set`.
* If `self.appchain_state` is `active`, reduce `amount` from `deposit_amount` of the `delegator` corresponding to pair (`sender_id`, `account_id`) as (`delegator_id`, `validator_id`) in `self.next_validator_set`.
* Create a new `staking history` with fact `DelegationIncreased` and insert it into `self.staking_histories` with key `self.staking_history_end_index + 1`.
* Add 1 to `self.staking_history_end_index`.
* Reduce `amount` from `self.total_stake`.
* Call function `ft_transfer` of `oct_token_contract` with parameters `sender` and `amount`:
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
* Call function `ft_transfer` of `oct_token_contract` with parameters `sender` and `delegator.deposit_amount`:
  * If success:
    * Generate log: `Delegating deposit of <sender> for <validator_id> is withdrawed. Amount: <deposit_amount>`
  * If fail:
    * Generate log: `Failed to withdraw delegating deposit of <sender> for <validator_id>. Amount: <deposit_amount>`
* Remove the `delegator` from `self.unbonded_validator_set`.

## Manage appchain lifecycle

### Go booting

Qualification of this action:

* The `sender` must be the `owner`.
* The `self.appchain_state` must be `staging`.
* The metadata of `wrapped appchain token` has already been set by [Set metadata of wrapped appchain token](#set-metadata-of-wrapped-appchain-token).
* If the `contract_account` of `warpped appchain token` must be set.
* The count of `validator` in `self.current_validator_set` must be not smaller than `self.protocol_settings.minimum_validator_count`.
* The `self.total_stake` must be not smaller than `self.protocol_settings.minimum_total_stake_for_booting`.

Processing steps:

* The `self.appchain_state` is set to `booting`.
* Sync `self.appchain_state` to `appchain registry`.

### Go live

Qualification of this action:

* The `sender` must be the `owner`.
* The `self.appchain_state` must be `booting`.

Processing steps:

* The `self.appchain_state` is set to `active`.
* Store currently registered validators and delegators as `appchain message` with type `update validator set` in this contract.
* Sync `self.appchain_state` to `appchain registry`.
