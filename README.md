# octopus-appchain-anchor

This contract provides an anchor for an appchain of [Octopus Network](https://oct.network). It is in charge of managing the necessary data of an appchain on NEAR protocol, providing the security and governance ability for the appchain.

Each appchain of Octopus Network will be bound to an instance of this contract, which is deployed to a subaccount of [octopus-appchain-registry](https://github.com/octopus-network/octopus-appchain-registry).

## Terminology

* `owner`: The owner of this contract, which is the Octopus Foundation.
* `appchain registry`: A NEAR contract which manage the lifecycle of appchains of Octopus Network, controlled by Octopus Foundation.
* `octopus relayer`: A standalone service which will monitor the state change of the validators of an appchain and facts happened on an appchain. It is controlled by the Octopus Foundation, and will relay messages between an appchain and this contract.
* `appchain owner`: The owner of an appchain.
* `appchain state`: The state of an appchain, which is one of the following:
  * `registered`: The initial state of an appchain, after it is successfully registered. This state is managed by `appchain registry`.
  * `auditing`: The state while the appchain is under auditing by Octopus Foundation. This state is managed by `appchain registry`.
  * `inQueue`: The state while `voter` can upvote or downvote an appchain. This state is managed by `appchain registry`.
  * `staging`: The state while `validator` and `delegator` can deposit OCT tokens to this contract to indicate their willing of staking for an appchain.
  * `booting`: The state while an appchain is booting.
  * `active`: The state while an appchain is active normally.
  * `broken`: The state which an appchain is broken for some technical or governance reasons.
  * `dead`: The state which the lifecycle of an appchain is end.
* `validator`: Who can deposit an amount of OCT token for an appchain when it is in `staging` state, to indicate that he/she wants to be the validator of an appchain after the appchain goes `booting` state.
* `delegator`: Who can deposit an amount of OCT token for an appchain when it is in `staging` state, to indicate that he/she wants to delegate his/her voting rights to an validator of an appchain after the appchain goes `booting` state.
* `sender`: A NEAR transaction sender, that is the account which perform actions (call functions) on this contract.
* `minimum validator deposit`: A `validator` has to deposit a certain amount of OCT token to this contract for being validator of the appchain.
* `staked balance`: The total amount of staked OCT tokens of a `validator` of the appchain bound to this contract.
* `delegated balance`: The total amount of delegated OCT tokens of a `delegator` of a `validator` of the appchain bound to this contract.
* `bridge token`: A token which is lived in NEAR protocol. It should be a NEP-141 compatible contract. This contract can bridge it to the appchain which is bound to this contract. The `bridging state` is one of the following:
  * `active`: The state which this contract is bridging the `bridge token` to the appchain.
  * `closed`: The state which this contract has stopped bridging the `bridge token` to the appchain.
* `appchain fact`: The fact which happens in the appchain or the changes of validators and delegators happens in this contract. There are three types of `appchain fact`:
  * `update validator set`: The fact that the validaors and delegators changes.
  * `lock asset`: The fact that a certain amount of appchain native token locked in the appchain.
  * `burn`: The fact that a certain amount of appchain native token burnt in the appchain.

## Implementation details

### Initialization

This contract has to be initialized by the following parameters:

* `appchain_id`: The id of an appchain which is bound to this contract.
* `appchain_registry`: The account id of `appchain registry`.
* `oct_token_contract`: The account id of OCT token contract.

These parameters are stored in this contract. The `appchain state` is set to `staging`.

### Register bridge token

This action needs the following parameters:

* `symbol`: The symbol of the `bridge token`.
* `decimals`: The decimals of the `bridge token`.
* `contract_account`: The account id of the `bridge token` contract.
* `price`: The price of the `bridge token`.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must not be registered.

These parameters are stored in this contract, mapped by `symbol`. The default `bridging state` of the `bridge token` is `closed`.

### Set price of a bridge token

This action needs the following parameters:

* `symbol`: The symbol of the bridge token.
* `price`: The price of the `bridge token`.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must already be registered.

The price of the `bridge token` corresponding to `symbol` is set to `price`.

### Open bridging for a bridge token

This action needs the following parameters:

* `symbol`: The symbol of the bridge token.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must already be registered.

The `bridging state` of the given `bridge token` is set to `active`.

### Close bridging for a bridge token

This action needs the following parameters:

* `symbol`: The symbol of the bridge token.

Qualification of this action:

* The `sender` must be the `owner`.
* The `symbol` must already be registered.

The `bridging state` of the given `bridge token` is set to `closed`.

### Set metadata of appchain native token

This action needs the following parameters:

* `name`: The name of appchain native token.
* `symbol`: The symbol of appchain native token.
* `decimals`: The decimals of appchain native token.
* `spec`: The specification of appchain native token.
* `icon`: (Optional) The data of icon file of appchain native token.
* `reference`: (Optional) The reference data of appchain native token.
* `reference_hash`: (Optional) The hash of reference data of appchain native token.

Qualification of this action:

* The `sender` must be the `owner`.
* The `appchain state` must be `staging`.

These parameters are stored in this contract as the metadata of appchain native token. These are used when [Go booting](#go-booting).

### Stage code of appchain native token contract

This action needs the following parameters:

* `code`: The wasm code of native token contract of the appchain.

Qualification of this action:

* The `sender` must be the `owner`.
* The `appchain state` must be `staging`.

The `code` is stored in this contract, it is used when [Go booting](#go-booting).

> Octopus Network provides [a standard implementation](https://github.com/octopus-network/appchain-native-token) of appchain native token contact.

### Set price of appchain native token

This action needs the following parameters:

* `price`: The price of the appchain native token.

Qualification of this action:

* The `sender` must be the `owner`.
* The `appchain state` must be `booting` or `active`.

The price of the appchain natvie token is set to `price`.

### Go booting

Qualification of this action:

* The `sender` must be the `owner`.
* The `appchain state` must be `staging`.
* The metadata of appchain native token has already set by [Set metadata of appchain native token](#set-metadata-of-appchain-native-token).
* The code of appchain native token contract has already staged by [Stage code of appchain natvie token contract](#stage-code-of-appchain-native-token-contract).

The `appchain state` is set to `booting`.

Store currently registered validators and delegators as the first `appchain fact` with type `update validator set` in this contract.

Deploy and initialize the contract of appchain native token:

* Deploy the code of appchain native token contract to account `token.<account id of this contract>`.
* Create a new full access key of the deployed contract for this contract.
* Call function `new` of the deployed contract with the metadata of appchain native token stored in this contract.

### Callback function 'ft_on_transfer'

This contract has a callback interface `FungibleTokenReceiver::ft_on_transfer` for contract `fungible_token` of `near-contract-standards`.

The callback function `ft_on_transfer` needs the following parameters:

* `sender_id`: The account id of sender of the transfer.
* `amount`: The amount of the transfer.
* `msg`: The message attached to the transfer, which indicates the purpose of the deposit.

If the caller of this callback (`env::predecessor_account_id()`) is `oct_token_contract` which is initialized at construction time of this contract, perform [Confirm and record OCT token deposit](#confirm-and-record-oct-token-deposit).

If the caller of this callback (`env::predecessor_account_id()`) is `token.<account id of this contract>` which is set when [Go booting](#go-booting), perform [Bridge appchain native token to appchain](#bridge-appchain-native-token-to-appchain).

If the caller of this callback (`env::predecessor_account_id()`) is `contract_account` of a `bridge token` registered in this contract, perform [Bridge NEAR native token to appchain](#bridge-near-native-token-to-appchain).

Otherwise, throws an error.

### Confirm and record OCT token deposit

This action will parse parameter `msg` of callback function `ft_on_transfer` and perform additional operations related to the deposit. The `msg` can be one of the following patterns:

* `register validator`:
  * The `appchain state` must not be `broken` or `dead`. Otherwise, the deposit will be considered as `invalid deposit`.
  * The amount of deposit must not be less than `minimum validator deposit`. Otherwise, the deposit will be considered as `invalid deposit`.
  * Register `sender_id` as a `validator` of this appchain. The `staked balance` of `sender_id` is set to `amount`.
  * Generate log: `Validator <sender_id> is registered with <amount> staked.`
* `raise staking`:
  * The `appchain state` must not be `broken` or `dead`. Otherwise, the deposit will be considered as `invalid deposit`.
  * If the `sender_id` is not a `validator` of this appchain, the deposit will be considered as `invalid deposit`.
  * Add the `amount` to the `staked balance` of `sender_id`.
  * Generate log: `Staked balance of validator <sender_id> raised by <amount>.`
* `register delegator of <account_id>`:
  * The `appchain state` must not be `broken` or `dead`. Otherwise, the deposit will be considered as `invalid deposit`.
  * If the `account_id` is not a `validator` of this appchain, the deposit will be considered as `invalid deposit`.
  * Register `sender_id` as a `delegator` of `validator` corresponding to `account_id`. The `delegated balace` of `sender_id` of validator `account_id` is set to `amount`.
  * Generate log: `Delegator <sender_id> of validator <account_id> is registered with <amount> delegated.`
* `raise delegating to <account_id>`:
  * The `appchain state` must not be `broken` or `dead`. Otherwise, the deposit will be considered as `invalid deposit`.
  * If the `sender_id` is not a `delegator` of this appchain, the deposit will be considered as `invalid deposit`.
  * Add the `amount` to the `delegated balance` of `sender_id` of validator `account_id`.
  * Generate log: `The delegated balance of delegator <sender_id> of validator <account_id> raised by <amount>.`
* other cases:
  * The deposit will be considered as `invalid deposit`.

For `invalid deposit` case, this contract will store the amount of the deposit to `invalid deposit` of `sender_id`, and generate log: `Received invalid deposit <amount> from <sender_id>.`

### Bridge appchain native token to appchain

This action will parse parameter `msg` of callback function `ft_on_transfer` and perform additional operations related to the deposit. The `msg` can be one of the following patterns:

* `bridge to <account_id>`:
  * Generate log: `<appchain_id> native token from <sender_id> locked. Target: <account_id>, Amount: <amount>`
* other cases:
  * The deposit will be considered as `invalid deposit`.

For `invalid deposit` case, this contract will store the amount of the deposit to `invalid deposit` of `sender_id`, and generate log: `Invalid deposit of <appchain_id> native token from <sender_id> received. Amount: <amount>`

### Bridge NEAR native token to appchain

This action will get the `bridge token` corresponding to the account id of caller (`env::predecessor_account_id()`). And then parse parameter `msg` of callback function `ft_on_transfer` and perform additional operations related to the deposit. The `msg` can be one of the following patterns:

* `bridge to <account_id>`:
  * Generate log: `<bridge token> from <sender_id> locked. Target: <account_id>, Amount: <amount>`
* other cases:
  * The deposit will be considered as `invalid deposit`.

For `invalid deposit` case, this contract will store the amount of the deposit to `invalid deposit` of `sender_id`, and generate log: `Invalid deposit of <bridge token> from <sender_id> received. Amount: <amount>`
