# octopus-appchain-anchor

This contract provides an anchor for an appchain of [Octopus Network](https://oct.network). It is in charge of managing the necessary data of an appchain in NEAR protocol, , providing security and interoperability for the appchain.

Each appchain of Octopus Network will be bonded to an instance of this contract, which is deployed to a subaccount of [octopus-appchain-registry](https://github.com/octopus-network/octopus-appchain-registry).

Contents

* [Terminology](#terminology)
* [Function specification](#function-specification)
  * [Manage appchain settings](#manage-appchain-settings)
  * [Manage anchor settings](#manage-anchor-settings)
  * [Manage protocol settings](#manage-protocol-settings)
  * [Manage NEAR fungible token](#manage-near-fungible-token)
  * [Manage wrapped appchain token](#manage-wrapped-appchain-token)
  * [Manage staking](#manage-staking)
  * [Switch validator set](#switch-validator-set)
  * [Withdraw reward](#withdraw-reward)
  * [Manage appchain lifecycle](#manage-appchain-lifecycle)

## Terminology

* `owner`: The owner of this contract, that is the Octopus Network.
* `appchain registry`: A NEAR contract which manage the lifecycle of appchains of Octopus Network, controlled by Octopus Network.
* `appchain owner`: The owner of an appchain.
* `appchain state`: The state of an appchain, the state `staging`, `booting`, `active`, `frozen`, `broken` and `dead` will be managed in this contract.
* `account id in appchain`: The account id in the appchain, which is usually the public key of an account in the appchain. The id is bonded to an account id in NEAR protocol in this contract.
* `validator`: A person who wants to act as a validator on the appchain corresponding to this contract. The person has to deposit a certain amount of OCT token in this contract.
* `delegator`: A person who wants to act as a delegator in the corresponding appchain. The person has to deposit a certain amount of OCT token in this contract, to indicate that he/she wants to delegate his/her voting rights to a certain `validator` of the appchain.
* `validator set`: A set of validators and delegators of the corresponding appchain. This set may change in every era of appchain, based on the actions happened in this contract during the period of last era of corresponding appchain.
* `era`: A certain period in the corresponding appchain that the reward distribution and validator set switching need to be performed.
* `OCT token`: The OCT token is used to stake for the validators of corresponding appchain.
* `NEAR fungible token`: A token which is lived in NEAR protocol. It should be a NEP-141 compatible contract. This contract can bridge the token to the corresponding appchain.
* `wrapped appchain token`: The wrapped token of the appchain native token, which is managed by a contract in NEAR protocol.
* `token bridging history`: The token bridging fact happens in this contract. These data will be used to browse or audit all bridging actions happened in this contract.
* `staking history`: The staking history happens in this contract. These data will be used to recover the status of `validator set` at a certain time.
* `appchain message`: The message which is relayed to this contract by `octopus relayer`.
* `anchor event`: The events which indicate that the corresponding appchain may need to perform necessary operations.
* `octopus relayer`: A standalone service which will relay the `appchain message` to this contract.
* `appchain settings`: A set of settings for booting corresponding appchain, which includes `chain_spec`, `raw_chain_spec`, `boot_nodes`, `rpc_endpoint` and other necessary field(s).
* `anchor settings`: A set of settings for current appchain anchor, which includes `token_price_maintainer_account` and other necessary field(s).
* `protocol settings`: A set of settings for Octopus Network protocol, maintained by the `owner`, which includes the following fields:
  * `minimum_validator_deposit`: The minimum deposit amount for a validator to register itself to this contract.
  * `minimum_delegator_deposit`: The minimum deposit amount for a delegator to delegate his voting weight to a certain validator.
  * `minimum_total_stake_for_booting`: The minimum value of total stake in this contract for booting corresponding appchain.
  * `maximum_market_value_percent_of_near_fungible_tokens`: The maximum percentage of the total market value of all NEAR fungible tokens to the total market value of OCT token staked in this contract
  * `maximum_market_value_percent_of_wrapped_appchain_token`: The maximum percentage of the total market value of wrapped appchain token to the total market value of OCT token staked in this contract.
  * `minimum_validator_count`: The minimum number of validator(s) registered in this contract for booting the corresponding appchain and keep it alive.
  * `maximum_validators_per_delegator`: The maximum number of validator(s) which a delegator can delegate to.
  * `unlock_period_of_validator_deposit`: The unlock period (in days) for validator(s) can withdraw their deposit after they are removed from the corresponding appchain.
  * `unlock_period_of_delegator_deposit`: The unlock period (in days) for delegator(s) can withdraw their deposit after they no longer delegates their stake to a certain validator on the corresponding appchain.
  * `maximum_era_count_of_unwithdrawn_benefit`: The maximum number of historical eras that the validators or delegators are allowed to withdraw their benefit.
* `sender`: A NEAR transaction sender, that is the account which perform actions (call functions) in this contract.

## Function specification

> For the data design, interface design and processing details, please refer to [implementation detail](https://github.com/octopus-network/octopus-appchain-anchor/blob/main/implementation_detail.md).

### Manage appchain settings

This contract has a set of functions to manage the value of each field of `appchain settings`.

### Manage anchor settings

This contract has a set of functions to manage the value of each field of `anchor settings`.

### Manage protocol settings

This contract has a set of functions to manage the value of each field of `protocol settings`.

### Manage NEAR fungible token

This contract can bridge multiple NEAR fungible tokens to the corresponding appchain. The limitation is: the total market value of all `NEAR fungible token` bridged to the corresponding appchain, cannot exceed the market value of a certain percent of all OCT token staked in this contract. The percentage is managed by `maximum_market_value_percent_of_near_fungible_tokens` of `protocol settings`.

This contract should provide the following public interfaces related to NEAR fungible token management:

* Regster NEAR fungible token.
* Set price of a NEAR fungible token. This action can only be performed by `token_price_maintainer_account` which is managed in `anchor settings`.
* Open bridging for a NEAR fungible token.
* Close bridging for a NEAR fungible token.

When this contract receives a deposit of a certain amount of a registered `NEAR fungible token`, this contract should check the limitation and then generate `token bridging history` for corresponding appchain to mint equivalent amount of the `NEAR fungible token`.

When this contract receives an `appchain message` which indicates that the appchain has burnt a certain amount of a registered `NEAR fungible token`, this contract should unlock equivalent amount of the NEAR fungible token and transfer them to the proper account in NEAR protocol.

### Manage wrapped appchain token

The contract of `wrapped appchain token` in NEAR protocol should be deployed before the appchain go `active`. The owner of the token contract should be set to the owner of this contract. The initial total supply of `wrapped appchain token` should be minted to an account belongs to the appchain team.

The limitation of `wrapped appchain token` is: the market value of `wrapped appchain token` in NEAR contract cannot exceed the market value of a certain percent of all OCT token staked in this contract. The percentage is managed by `maximum_market_value_percent_of_wrapped_appchain_token` of `protocol settings`.

This contract should provide the following public interfaces related to wrapped appchain token management:

* Set metadata of wrapped appchain token.
* Set contract account of wrapped appchain token.
* Set initial balance of wrapped appchain token.
* Set price of wrapped appchain token. This action can only be performed by `token_price_maintainer_account` which is managed in `anchor settings`.
* Burn wrapped appchain token. Which will generate `token bridging history` for corresponding appchain to mint equivalent amount of native token.

When this contract receives an `appchain message` which indicates that the appchain has locked a certain amount of `wrapped appchain token`, this contract should mint equivalent amount of `wrapped appchain token` in the corresponding NEAR fungible token contract.

### Manage staking

Any user in NEAR protocol can deposit a certain amount (not less than `minimum_validator_deposit` of `protocol settings`) of OCT token to this contract to register his/her account as a `validator` of next `era` of corresponding appchain. The user should also specify the validator account id which will be used in the corresponding appchain, and specify the flag which indicates that 'whether the validator wants to be delegated to'.

Any user in NEAR protocol can deposit a certain amount of OCT token to this contract to increase his/her stake as a `validator` in next `era` of corresponding appchain. The user must be already a registered `validator` and the `validator` must not be unbonded.

Any user in NEAR protocol can deposit a certain amount (not less than `minimum_delegator_deposit` of `protocol settings`) of OCT token to this contract to register his/her account as a `delegator` of next `era` of corresponding appchain. The user should also specify the validator account id (in the corresponding appchain) that he/she want to delegate to.

Any user in NEAR protocol can deposit a certain amount of OCT token to this contract to increase his/her delegation as a `delegator` in next `era` of corresponding appchain. The user must be already a registered `delegator` of a certain `validator` and the `delegator` must not be unbonded.

A registered `validator` can unbond himself/herself from corresponding appchain. At this case, this contract should:

* Remove the `validator` from the `validator set` of next `era` of corresponding appchain and move it to `unbonded validator set`. The lock period of the unbonded stake will start from the start time of next `era` and last for the duration of `unlock_period_of_validator_deposit` of `protocol settings`, before the validator can withdraw the unbonded stake.
* Remove all delegators of the `validator` from the `validator set` of next `era` of corresponding appchain. The lock period of the decreased delegation will start from the start time of next `era` and last for the duration of `unlock_period_of_delegator_deposit` of `protocol settings`, before the delegator can withdraw the unbonded delegation.

A registered `delegator` can unbond himself/herself from a specific `validator` of corresponding appchain. At this case, this contract should remove the `delegator` from the `validator set` of next `era` of corresponding appchain. The lock period of the unbonded delegation will start from the start time of next `era` and last for the duration of `unlock_period_of_delegator_deposit` of `protocol settings`, before the delegator can withdraw the unbonded delegation.

A validator can decrease his/her stake while the validator is still active (not unbonded) in corresponding appchain. The deposit of the validator after the reduction cannot be less than `minimum_validator_deposit` of `protocol settings`, and the total stake of the `validator set` of next `era` after the reduction cannot be less than 2/3 of the total stake of the `validator set` of last `era`. The lock period of the decreased stake will start from the start time of next `era` and last for the duration of `unlock_period_of_validator_deposit` of `protocol settings`, before the validator can withdraw the decreased stake.

A delegator can decrease his/her delegation while the delegator is still active (not unbonded) in corresponding appchain. The deposit of the delegator after the reduction cannot be less than `minimum_delegator_deposit` of `protocol settings`, and the total stake of the `validator set` of next `era` after the reduction cannot be less than 2/3 of the total stake of the `validator set` of last `era`. The lock period of the decreased delegation will start from the start time of next `era` and last for the duration of `unlock_period_of_delegator_deposit` of `protocol settings`, before the delegator can withdraw the decreased delegation.

Each of the above actions will generate a corresponding `staking history` that is stored in this contract. These staking histories are used to restore the `validator set` of a certain `era`.

A validator can also change the flag which is set at registering time and stored in this contract, the flag indicates that 'whether he/she wants to be delegated to'. After this flag is set to `false`, delegators cannot delegate to this validator any more. But those delegators already delegated to this validator will be kept.

### Switch validator set

When this contract receives an `appchain message` which indicates that the corresponding appchain has switched to a new `era`, this contract should:

* Create a new (empty) `validator set` for the next `era` that stored in this contract.
* Store the `total benefit` and `unprofitable validator id list` carried by the `appchain message` in the `validator set` of next `era`.
* Restore the state of the `validator set` of next `era` by sequentially applying all staking histories happened before the time of this `appchain message` is received. During this process, also generate a copy of the status of all `validator`(s) in the `validator set` of next `era` for the query of appchain nodes. (Because the data struct for query of appchain nodes may be defferent with the internal storage of this contract.)
* Calculate the unwithdrawn benefit of all validators and delegators in the `validator set` of next `era` based on the `total benefit` and `unprofitable validator id list` of the `era` (distribute the `total benefit` proportionally to all profitable validators and delegators), and store the results in the `validator set` of next `era`.
* Change the next `era` stored in this contract to the new `era`.

> The next `era` stored in this contract is the one before the new `era` specified by current `appchain message`. It is actually the `era` which is specified by the last `appchain message` of this type.

Notice that, due to the gas limit of a transaction, the whole process may cost more than one transaction to complete.

### Withdraw reward

A validator or deleagtor can withdraw their benefit in latest eras at any time. The earliest era in which they can withdraw is limited by `maximum_era_count_of_unwithdrawn_benefit` of `protocol settings`.

### Manage appchain lifecycle

The owner of appchain anchor can manually change the state of corresponding appchain. These actions need to check necessary conditions before changing the state of corresponding appchain. And after changing the state, this contract will call function `sync_state_of` of `appchain registry` contract to synchronize the state to `appchain registry`. (The `appchain registry` will ensure the caller account of this function is `<appchain_id>.<appchain registry account>`.)
