use near_sdk::BlockHeight;

use crate::*;

/// Appchain validator of an appchain.
#[derive(BorshDeserialize, BorshSerialize)]
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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct ValidatorOfEra {
    /// Base data of validator
    pub validator: Validator,
    /// Total deposited balance of the validator in a era.
    pub deposit_amount: Balance,
    /// Total stake of the validator in a era, including delegations of all delegators.
    pub total_stake: Balance,
    /// Whether the validator accepts delegation from delegators.
    pub can_be_delegated_to: bool,
    /// The unwithdrawed benefit of a era, in unit of wrapped appchain token.
    pub unwithdrawed_benefit: Balance,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UnbondedValidator {
    /// The base data of validator
    pub validator: Validator,
    /// Total deposited balance of the validator when he/she is unbonded.
    pub deposit_amount: Balance,
    /// The block height when the validator is unbonded.
    pub unbonded_block_height: BlockHeight,
    /// The timestamp when the validator is unbonded.
    pub unbonded_timestamp: Timestamp,
    /// The time when the validator can withdraw their deposit.
    pub staking_unlock_time: Timestamp,
}

#[derive(BorshDeserialize, BorshSerialize)]
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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct DelegatorOfEra {
    /// The base data of delegator
    pub delegator: Delegator,
    /// Delegated balance of the delegator in a era.
    pub deposit_amount: Balance,
    /// The unwithdrawed benefit of a era, in unit of wrapped appchain token.
    pub unwithdrawed_benefit: Balance,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UnbondedDelegator {
    /// The base data of delegator
    pub delegator: Delegator,
    /// Total deposited balance of the delegator when he/she is unbonded.
    pub deposit_amount: Balance,
    /// The block height when the delegator is unbonded.
    pub unbonded_block_height: BlockHeight,
    /// The timestamp when the delegator is unbonded.
    pub unbonded_timestamp: Timestamp,
    /// The time when the delegator can withdraw their deposit.
    pub staking_unlock_time: Timestamp,
}

#[derive(BorshDeserialize, BorshSerialize)]
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

#[derive(BorshDeserialize, BorshSerialize)]
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
