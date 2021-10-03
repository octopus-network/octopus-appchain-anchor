use near_sdk::BlockHeight;

use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct StakingHistories {
    /// The staking history data happened in this contract.
    pub histories: LookupMap<u64, StakingHistory>,
    /// The start index of valid staking history in `staking_histories`.
    pub start_index: u64,
    /// The end index of valid staking history in `staking_histories`.
    pub end_index: u64,
}

pub trait StakingManager {
    ///
    fn increase_stake(&mut self, amount: U64);
    ///
    fn decrease_stake(&mut self, amount: U64);
    ///
    fn unbond_stake(&mut self);
    ///
    fn enable_delegation(&mut self);
    ///
    fn disable_delegation(&mut self);
    ///
    fn increase_delegation(&mut self, validator_id: AccountId, amount: U64);
    ///
    fn decrease_delegation(&mut self, validator_id: AccountId, amount: U64);
    ///
    fn unbond_delegation(&mut self, validator_id: AccountId);
    ///
    fn withdraw_stake(&mut self, era_number: Option<U64>);
    ///
    fn withdraw_validator_rewards(&mut self, validator_id: AccountId);
    ///
    fn withdraw_delegator_rewards(&mut self, delegator_id: AccountId, validator_id: AccountId);
}

impl AppchainAnchor {
    //
    fn register_validator(
        &mut self,
        validator_id: AccountId,
        validator_id_in_appchain: AccountIdInAppchain,
        amount: u64,
        can_be_delegated_to: bool,
    ) {
        todo!()
    }
    //
    fn register_delegator(
        &mut self,
        delegator_id: AccountId,
        validator_id: AccountId,
        amount: u64,
    ) {
        todo!()
    }
}

#[near_bindgen]
impl StakingManager for AppchainAnchor {
    //
    fn increase_stake(&mut self, amount: U64) {
        todo!()
    }
    //
    fn decrease_stake(&mut self, amount: U64) {
        todo!()
    }
    //
    fn unbond_stake(&mut self) {
        todo!()
    }
    //
    fn enable_delegation(&mut self) {
        todo!()
    }
    //
    fn disable_delegation(&mut self) {
        todo!()
    }
    //
    fn increase_delegation(&mut self, validator_id: AccountId, amount: U64) {
        todo!()
    }
    //
    fn decrease_delegation(&mut self, validator_id: AccountId, amount: U64) {
        todo!()
    }
    //
    fn unbond_delegation(&mut self, validator_id: AccountId) {
        todo!()
    }
    //
    fn withdraw_stake(&mut self, era_number: Option<U64>) {
        todo!()
    }
    //
    fn withdraw_validator_rewards(&mut self, validator_id: AccountId) {
        todo!()
    }
    //
    fn withdraw_delegator_rewards(&mut self, delegator_id: AccountId, validator_id: AccountId) {
        todo!()
    }
}
