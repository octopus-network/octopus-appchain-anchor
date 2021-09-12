use crate::*;

pub trait StakingManager {
    ///
    fn decrease_stake(&mut self, amount: U64);
    ///
    fn withdraw_stake(&mut self);
    ///
    fn decrease_delegation(&mut self, validator_id: AccountId, amount: U64);
    ///
    fn withdraw_delegation(&mut self, validator_id: AccountId);
}

impl AppchainAnchor {
    //
    fn register_reserved_validator(
        &mut self,
        validator_id: AccountId,
        validator_id_in_appchain: AccountIdInAppchain,
        amount: u64,
    ) {
        todo!()
    }
    //
    fn register_validator(
        &mut self,
        validator_id: AccountId,
        validator_id_in_appchain: AccountIdInAppchain,
        payee_id_in_appchain: AccountIdInAppchain,
        amount: u64,
    ) {
        todo!()
    }
    //
    fn increase_stake(&mut self, validator_id: AccountId, amount: u64) {
        todo!()
    }
    //
    fn register_delegator(
        &mut self,
        delegator_id: AccountId,
        delegator_id_in_appchain: AccountIdInAppchain,
        validator_id: AccountId,
        amount: u64,
    ) {
        todo!()
    }
    //
    fn increase_delegation(&mut self, delegator_id: AccountId, validator_id: AccountId) {
        todo!()
    }
    //
    fn unbond_stake(&mut self, validator_id_in_appchain: AccountIdInAppchain) {
        todo!()
    }
    //
    fn unbond_delegation(
        &mut self,
        delegator_id_in_appchain: AccountIdInAppchain,
        validator_id_in_appchain: AccountIdInAppchain,
    ) {
        todo!()
    }
}
