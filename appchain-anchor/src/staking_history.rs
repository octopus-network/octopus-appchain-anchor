use near_sdk::BlockHeight;

use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct StakingHistory {
    pub staking_fact: StakingFact,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub index: U64,
}

pub trait StakingHistoryManager {
    ///
    fn apply_staking_histories_in_last_era(&mut self) -> bool;
}

impl AppchainAnchor {
    //
    fn start_applying_staking_histories_in_last_era(&mut self) {
        todo!()
    }
}
