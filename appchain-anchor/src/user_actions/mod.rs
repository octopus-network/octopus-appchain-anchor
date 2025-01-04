use crate::*;

mod settings_manager;

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct UnbondedStakeReference {
    /// The number of era in appchain.
    pub era_number: u64,
    /// The index of corresponding `staking history`
    pub staking_history_index: u64,
}
