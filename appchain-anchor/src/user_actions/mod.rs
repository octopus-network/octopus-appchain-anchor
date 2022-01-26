use crate::*;

mod appchain_lifecycle;
mod owner_actions;
mod settings_manager;
mod staking;
mod sudo_actions;
mod validator_actions;

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub struct UnbondedStakeReference {
    /// The number of era in appchain.
    pub era_number: u64,
    /// The index of corresponding `staking history`
    pub staking_history_index: u64,
}
