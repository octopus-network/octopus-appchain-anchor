use near_sdk::BlockHeight;

use crate::*;

/// The staking state of a validator or delegator of the appchain.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
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

/// Appchain validator of an appchain.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
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

#[derive(BorshDeserialize, BorshSerialize)]
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

#[derive(BorshDeserialize, BorshSerialize)]
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
