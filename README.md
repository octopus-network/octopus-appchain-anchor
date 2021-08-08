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

## Implementation details

TBD
