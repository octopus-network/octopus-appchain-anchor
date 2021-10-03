use near_sdk::BlockHeight;

use crate::*;
use core::convert::{TryFrom, TryInto};
use validator_set::*;

/// The message which is sent from the appchain
pub enum AppchainFact {
    /// The fact that a certain amount of bridge token has been burnt in the appchain.
    NearFungibleTokenBurnt { symbol: String, amount: U128 },
    /// The fact that a certain amount of appchain native token has been locked in the appchain.
    NativeTokenLocked { amount: U128 },
    /// The fact that the era switch is planed in the appchain.
    EraSwitchPlaned { era_number: U64 },
    /// The fact that the total reward and unprofitable validator list
    /// is concluded in the appchain.
    EraRewardConcluded {
        era_number: U64,
        unprofitable_validator_ids: Vec<AccountIdInAppchain>,
    },
    /// The era reward is changed in the appchain
    EraRewardChanged {
        era_number: U64,
        era_reward: Balance,
    },
}

pub struct AppchainMessage {
    pub appchain_fact: AppchainFact,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub nonce: u32,
}

pub trait PermissionlessActions {
    ///
    fn handle_appchain_message(
        &mut self,
        encoded_message: Vec<u8>,
        header_partial: Vec<u8>,
        leaf_proof: Vec<u8>,
        mmr_root: Vec<u8>,
    );
    ///
    fn try_complete_switching_era(&mut self) -> bool;
    ///
    fn try_complete_distributing_reward(&mut self) -> bool;
}

enum ResultOfLoopingValidatorSet {
    NoMoreDelegator,
    NoMoreValidator,
    NeedToContinue,
}

#[near_bindgen]
impl PermissionlessActions for AppchainAnchor {
    //
    fn handle_appchain_message(
        &mut self,
        encoded_message: Vec<u8>,
        header_partial: Vec<u8>,
        leaf_proof: Vec<u8>,
        mmr_root: Vec<u8>,
    ) {
        todo!()
    }
    //
    fn try_complete_switching_era(&mut self) -> bool {
        match self
            .permissionless_actions_status
            .get()
            .unwrap()
            .switching_era_number
        {
            Some(era_number) => self.complete_switching_era(era_number),
            None => true,
        }
    }
    //
    fn try_complete_distributing_reward(&mut self) -> bool {
        match self
            .permissionless_actions_status
            .get()
            .unwrap()
            .distributing_reward_era_number
        {
            Some(era_number) => self.complete_distributing_reward_of_era(era_number),
            None => true,
        }
    }
}

#[near_bindgen]
impl AppchainAnchor {
    //
    fn start_switching_era(&mut self, era_number: u64) {
        let mut permissionless_actions_status = self.permissionless_actions_status.get().unwrap();
        assert!(
            permissionless_actions_status.switching_era_number.is_none(),
            "Contract is still switching to era {}.",
            permissionless_actions_status.switching_era_number.unwrap()
        );
        self.validator_set_histories.insert(
            &era_number,
            &ValidatorSetOfEra::new(era_number, self.staking_histories.get().unwrap().end_index),
        );
        permissionless_actions_status.switching_era_number = Some(era_number);
        self.permissionless_actions_status
            .set(&permissionless_actions_status);
        self.complete_switching_era(era_number);
    }
    //
    fn complete_switching_era(&mut self, era_number: u64) -> bool {
        let mut validator_set = self.validator_set_histories.get(&era_number).unwrap();
        match validator_set.processing_status {
            ProcessingStatus::CopyingFromLastEra {
                copying_validator_index,
                copying_delegator_index,
            } => {
                if era_number > 0 {
                    assert!(
                        self.validator_set_histories.contains_key(&(era_number - 1)),
                        "Missing validator set of last era"
                    );
                    let last_validator_set =
                        self.validator_set_histories.get(&(era_number - 1)).unwrap();
                    let mut validator_index = copying_validator_index;
                    let mut delegator_index = copying_delegator_index;
                    while env::used_gas() < GAS_CAP_FOR_COMPLETE_SWITCHING_ERA {
                        match self.copy_delegator_to_era(
                            &last_validator_set,
                            &mut validator_set,
                            validator_index,
                            delegator_index,
                        ) {
                            ResultOfLoopingValidatorSet::NoMoreDelegator => {
                                validator_index += 1;
                                delegator_index = 0;
                            }
                            ResultOfLoopingValidatorSet::NoMoreValidator => {
                                validator_set.validator_set.total_stake =
                                    last_validator_set.validator_set.total_stake;
                                validator_set.processing_status =
                                    ProcessingStatus::ApplyingStakingHistory { applying_index: 0 };
                                self.validator_set_histories
                                    .insert(&era_number, &validator_set);
                                return true;
                            }
                            ResultOfLoopingValidatorSet::NeedToContinue => delegator_index += 1,
                        }
                    }
                    validator_set.processing_status = ProcessingStatus::CopyingFromLastEra {
                        copying_validator_index: validator_index,
                        copying_delegator_index: delegator_index,
                    };
                } else {
                    validator_set.processing_status =
                        ProcessingStatus::ApplyingStakingHistory { applying_index: 0 };
                }
                self.validator_set_histories
                    .insert(&era_number, &validator_set);
                false
            }
            ProcessingStatus::ApplyingStakingHistory { mut applying_index } => {
                while env::used_gas() < GAS_CAP_FOR_COMPLETE_SWITCHING_ERA
                    || applying_index >= validator_set.staking_history_index
                {
                    let staking_history = self
                        .staking_histories
                        .get()
                        .unwrap()
                        .histories
                        .get(&applying_index)
                        .unwrap();
                    self.apply_staking_history_to_era(&mut validator_set, &staking_history);
                    applying_index += 1;
                }
                if applying_index >= validator_set.staking_history_index {
                    validator_set.processing_status = ProcessingStatus::ReadyForDistributingReward;
                    return true;
                } else {
                    validator_set.processing_status =
                        ProcessingStatus::ApplyingStakingHistory { applying_index };
                    return false;
                }
            }
            _ => true,
        }
    }
    //
    fn copy_delegator_to_era(
        &mut self,
        source_validator_set: &ValidatorSetOfEra,
        target_validator_set: &mut ValidatorSetOfEra,
        validator_index: u64,
        delegator_index: u64,
    ) -> ResultOfLoopingValidatorSet {
        let validator_ids = source_validator_set.validator_set.validator_ids.to_vec();
        if validator_index >= validator_ids.len().try_into().unwrap() {
            return ResultOfLoopingValidatorSet::NoMoreValidator;
        }
        let validator_id = validator_ids
            .get(usize::try_from(validator_index).unwrap())
            .unwrap();
        let validator = source_validator_set
            .validator_set
            .validators
            .get(validator_id)
            .unwrap();
        if !target_validator_set
            .validator_set
            .validator_ids
            .contains(validator_id)
        {
            target_validator_set
                .validator_set
                .validator_ids
                .insert(validator_id);
            target_validator_set
                .validator_set
                .validators
                .insert(validator_id, &validator);
        }
        let delegater_ids = source_validator_set
            .validator_set
            .validator_id_to_delegator_ids
            .get(validator_id)
            .unwrap()
            .to_vec();
        if delegator_index >= delegater_ids.len().try_into().unwrap() {
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
        let delegator_id = delegater_ids
            .get(usize::try_from(delegator_index).unwrap())
            .unwrap();
        let delegator = source_validator_set
            .validator_set
            .delegators
            .get(&(delegator_id.clone(), validator_id.clone()))
            .unwrap();
        target_validator_set
            .validator_set
            .delegators
            .insert(&(delegator_id.clone(), validator_id.clone()), &delegator);
        target_validator_set
            .validator_set
            .validator_id_to_delegator_ids
            .get(validator_id)
            .unwrap()
            .insert(delegator_id);
        target_validator_set
            .validator_set
            .delegator_id_to_validator_ids
            .get(delegator_id)
            .unwrap()
            .insert(validator_id);
        return ResultOfLoopingValidatorSet::NeedToContinue;
    }
    //
    fn apply_staking_history_to_era(
        &mut self,
        validator_set: &mut ValidatorSetOfEra,
        staking_history: &StakingHistory,
    ) {
        match &staking_history.staking_fact {
            StakingFact::StakeDecreased {
                validator_id,
                amount: _,
            }
            | StakingFact::ValidatorUnbonded { validator_id } => {
                let mut stakes = match self.unbonded_stakes.get(validator_id) {
                    Some(stakes) => stakes,
                    None => Vec::<UnbondedStake>::new(),
                };
                stakes.push(UnbondedStake {
                    era_number: validator_set.validator_set.era_number + 1,
                    staking_history_index: staking_history.index.0,
                });
                self.unbonded_stakes.insert(validator_id, &stakes);
            }
            StakingFact::DelegationDecreased {
                delegator_id,
                validator_id: _,
                amount: _,
            }
            | StakingFact::DelegatorUnbonded {
                delegator_id,
                validator_id: _,
            } => {
                let mut stakes = match self.unbonded_stakes.get(delegator_id) {
                    Some(stakes) => stakes,
                    None => Vec::<UnbondedStake>::new(),
                };
                stakes.push(UnbondedStake {
                    era_number: validator_set.validator_set.era_number + 1,
                    staking_history_index: staking_history.index.0,
                });
                self.unbonded_stakes.insert(delegator_id, &stakes);
            }
            _ => (),
        }
        validator_set.apply_staking_history(staking_history);
    }
}

#[near_bindgen]
impl AppchainAnchor {
    //
    fn start_distributing_reward_of_era(
        &mut self,
        era_number: u64,
        unprofitable_validator_ids: Vec<AccountIdInAppchain>,
    ) {
        let mut permissionless_actions_status = self.permissionless_actions_status.get().unwrap();
        assert!(
            permissionless_actions_status
                .distributing_reward_era_number
                .is_none(),
            "Contract is still distributing reward for era {}.",
            permissionless_actions_status
                .distributing_reward_era_number
                .unwrap()
        );
        assert!(
            self.validator_set_histories.contains_key(&era_number),
            "Validator set is not existed."
        );
        let mut validator_set = self.validator_set_histories.get(&era_number).unwrap();
        assert!(
            validator_set
                .processing_status
                .is_ready_for_distributing_reward(),
            "Validator set is not ready for distributing reward."
        );
        let mut uv_ids = Vec::<AccountId>::new();
        for id_in_appchain in unprofitable_validator_ids {
            assert!(
                self.validator_account_id_mapping
                    .contains_key(&id_in_appchain),
                "Invalid validator id in appchain: {}",
                id_in_appchain
            );
            uv_ids.push(
                self.validator_account_id_mapping
                    .get(&id_in_appchain)
                    .unwrap(),
            );
        }
        validator_set.set_unprofitable_validator_ids(uv_ids);
        validator_set.calculate_valid_total_stake();
        validator_set.processing_status = ProcessingStatus::DistributingReward {
            distributing_validator_index: 0,
            distributing_delegator_index: 0,
        };
        self.validator_set_histories
            .insert(&era_number, &validator_set);
        permissionless_actions_status.distributing_reward_era_number = Some(era_number);
        self.permissionless_actions_status
            .set(&permissionless_actions_status);
        // TODO: mint `total_reward` in the contract of wrapped appchain token.

        // Start distributing reward of this era
        self.complete_distributing_reward_of_era(era_number);
    }
    //
    fn complete_distributing_reward_of_era(&mut self, era_number: u64) -> bool {
        let mut validator_set = self.validator_set_histories.get(&era_number).unwrap();
        match validator_set.processing_status {
            ProcessingStatus::CopyingFromLastEra {
                copying_validator_index: _,
                copying_delegator_index: _,
            } => false,
            ProcessingStatus::ApplyingStakingHistory { applying_index: _ } => false,
            ProcessingStatus::ReadyForDistributingReward => false,
            ProcessingStatus::DistributingReward {
                distributing_validator_index,
                distributing_delegator_index,
            } => {
                let delegation_fee_percent =
                    u128::from(self.protocol_settings.get().unwrap().delegation_fee_percent);
                let mut validator_index = distributing_validator_index;
                let mut delegator_index = distributing_delegator_index;
                let era_reward = self.appchain_settings.get().unwrap().era_reward;
                while env::used_gas() < GAS_CAP_FOR_COMPLETE_SWITCHING_ERA {
                    match self.distribute_reward(
                        &validator_set,
                        validator_index,
                        delegator_index,
                        era_reward,
                        delegation_fee_percent,
                    ) {
                        ResultOfLoopingValidatorSet::NoMoreDelegator => {
                            validator_index += 1;
                            delegator_index = 0;
                        }
                        ResultOfLoopingValidatorSet::NoMoreValidator => {
                            validator_set.processing_status = ProcessingStatus::Completed;
                            self.validator_set_histories
                                .insert(&era_number, &validator_set);
                            return false;
                        }
                        ResultOfLoopingValidatorSet::NeedToContinue => delegator_index += 1,
                    }
                }
                validator_set.processing_status = ProcessingStatus::DistributingReward {
                    distributing_validator_index: validator_index,
                    distributing_delegator_index: delegator_index,
                };
                self.validator_set_histories
                    .insert(&era_number, &validator_set);
                false
            }
            ProcessingStatus::Completed => true,
        }
    }
    //
    fn distribute_reward(
        &mut self,
        validator_set: &ValidatorSetOfEra,
        validator_index: u64,
        delegator_index: u64,
        era_reward: Balance,
        delegation_fee_percent: u128,
    ) -> ResultOfLoopingValidatorSet {
        let validator_ids = validator_set.validator_set.validator_ids.to_vec();
        if validator_index >= validator_ids.len().try_into().unwrap() {
            return ResultOfLoopingValidatorSet::NoMoreValidator;
        }
        let validator_id = validator_ids
            .get(usize::try_from(validator_index).unwrap())
            .unwrap();
        if validator_set
            .unprofitable_validator_ids
            .contains(validator_id)
        {
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
        let validator = validator_set
            .validator_set
            .validators
            .get(validator_id)
            .unwrap();
        let total_reward_of_validator =
            era_reward * validator.total_stake / validator_set.valid_total_stake;
        if !self
            .unwithdrawed_validator_rewards
            .contains_key(&(validator_set.validator_set.era_number, validator_id.clone()))
        {
            self.unwithdrawed_validator_rewards.insert(
                &(validator_set.validator_set.era_number, validator_id.clone()),
                &total_reward_of_validator,
            );
        }
        let delegater_ids = validator_set
            .validator_set
            .validator_id_to_delegator_ids
            .get(validator_id)
            .unwrap()
            .to_vec();
        if delegator_index >= delegater_ids.len().try_into().unwrap() {
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
        let delegator_id = delegater_ids
            .get(usize::try_from(delegator_index).unwrap())
            .unwrap();
        let delegator = validator_set
            .validator_set
            .delegators
            .get(&(delegator_id.clone(), validator_id.clone()))
            .unwrap();
        let delegator_reward =
            total_reward_of_validator * delegator.deposit_amount * (100 - delegation_fee_percent)
                / (validator.total_stake * 100);
        let mut validator_reward = self
            .unwithdrawed_validator_rewards
            .get(&(validator_set.validator_set.era_number, validator_id.clone()))
            .unwrap();
        self.unwithdrawed_delegator_rewards.insert(
            &(
                validator_set.validator_set.era_number,
                delegator_id.clone(),
                validator_id.clone(),
            ),
            &delegator_reward,
        );
        validator_reward -= delegator_reward;
        self.unwithdrawed_validator_rewards.insert(
            &(validator_set.validator_set.era_number, validator_id.clone()),
            &validator_reward,
        );
        return ResultOfLoopingValidatorSet::NeedToContinue;
    }
}
