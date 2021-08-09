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

## Implementation details

### Initialization

This contract has to be initialized by the following parameters:

* `appchain_id`: The id of an appchain which is bound to this contract.
* `appchain_registry`: The account id of `appchain registry`.
* `oct_token_contract`: The account id of OCT token contract.

These parameters are stored in this contract. The `appchain state` is set to `staging`.

### Register appchain native token

This action needs the following parameters:

* `native_token_contract`: The account id of appchain native token contract.

The `native_token_contract` is stored in this contract.

### Callback function 'ft_on_transfer'

This contract has a callback interface `FungibleTokenReceiver::ft_on_transfer` for contract `fungible_token` of `near-contract-standards`.

The callback function `ft_on_transfer` needs the following parameters:

* `sender_id`: The account id of sender of the transfer.
* `amount`: The amount of the transfer.
* `msg`: The message attached to the transfer, which indicates the purpose of the deposit.

If the caller of this callback (`env::predecessor_account_id()`) is `oct_token_contract` which initialized at construction time of this contract, perform [Confirm and record OCT token deposit](#confirm-and-record-oct-token-deposit).

If the caller of this callback (`env::predecessor_account_id()`) is `native_token_contract` which is set by [Register appchain native token](#register-appchain-native-token), perform [Confirm and record appchain native token deposit](#confirm-and-record-appchain-native-token-deposit).

Otherwise, throws an error.

### Confirm and record OCT token deposit

This action will parse parameter `msg` of callback function `ft_on_transfer` and perform additional operations related to the deposit. The `msg` can be one of the following patterns:

* `register validator`:
  * The `appchain state` of the appchain corresponding to `appchain_id` must not be `broken` or `dead`. Otherwise, the deposit will be considered as `invalid deposit`.
  * The amount of deposit must not be less than `minimum validator deposit`. Otherwise, the deposit will be considered as `invalid deposit`.
  * Register `sender_id` as a `validator` of this appchain. The `staked balance` of `sender_id` is set to `amount`.
  * Generate log: `Validator <sender_id> is registered with <amount> staked.`
* `raise staking`:
  * The `appchain state` of the appchain corresponding to `appchain_id` must not be `broken` or `dead`. Otherwise, the deposit will be considered as `invalid deposit`.
  * If the `sender_id` is not a `validator` of this appchain, the deposit will be considered as `invalid deposit`.
  * Add the `amount` to the `staked balance` of `sender_id`.
  * Generate log: `Staked balance of validator <sender_id> raised by <amount>.`
* `register delegator of <account_id>`:
  * The `appchain state` of the appchain corresponding to `appchain_id` must not be `broken` or `dead`. Otherwise, the deposit will be considered as `invalid deposit`.
  * If the `account_id` is not a `validator` of this appchain, the deposit will be considered as `invalid deposit`.
  * Register `sender_id` as a `delegator` of `validator` corresponding to `account_id`. The `delegated balace` of `sender_id` of validator `account_id` is set to `amount`.
  * Generate log: `Delegator <sender_id> of validator <account_id> is registered with <amount> delegated.`
* `raise delegating to <account_id>`:
  * The `appchain state` of the appchain corresponding to `appchain_id` must not be `broken` or `dead`. Otherwise, the deposit will be considered as `invalid deposit`.
  * If the `sender_id` is not a `delegator` of this appchain, the deposit will be considered as `invalid deposit`.
  * Add the `amount` to the `delegated balance` of `sender_id` of validator `account_id`.
  * Generate log: `The delegated balance of delegator <sender_id> of validator <account_id> raised by <amount>.`
* other cases:
  * The deposit will be considered as `invalid deposit`.

For `invalid deposit` case, this contract will store the amount of the deposit to `invalid deposit` of `sender_id`. The sender can withdraw the deposit at anytime.

This action should generate log: `Received invalid deposit <amount> from <sender_id>.`

### Confirm and record appchain native token deposit

TBD
