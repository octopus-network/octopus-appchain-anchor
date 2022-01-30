use super::ResultOfLoopingValidatorSet;
use crate::*;
use core::convert::{TryFrom, TryInto};
use user_actions::UnbondedStakeReference;

impl AppchainAnchor {
    //
    pub fn internal_start_switching_era(
        &mut self,
        era_number: u64,
        appchain_message_nonce: u32,
    ) -> AppchainMessageProcessingResult {
        let mut permissionless_actions_status = self.permissionless_actions_status.get().unwrap();
        if permissionless_actions_status.switching_era_number.is_some() {
            return AppchainMessageProcessingResult::Error {
                nonce: appchain_message_nonce,
                message: format!(
                    "Contract is still switching to era {}.",
                    permissionless_actions_status
                        .switching_era_number
                        .unwrap()
                        .0
                ),
            };
        }
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        if !validator_set_histories.contains(&era_number) {
            validator_set_histories.insert(
                &era_number,
                &ValidatorSetOfEra::new(
                    era_number,
                    self.staking_histories
                        .get()
                        .unwrap()
                        .index_range()
                        .end_index
                        .0,
                ),
            );
            self.validator_set_histories.set(&validator_set_histories);
        }
        permissionless_actions_status.switching_era_number = Some(U64::from(era_number));
        self.permissionless_actions_status
            .set(&permissionless_actions_status);
        AppchainMessageProcessingResult::Ok {
            nonce: appchain_message_nonce,
            message: None,
        }
    }
    //
    pub fn complete_switching_era(&mut self, era_number: u64) -> bool {
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        let mut validator_set = validator_set_histories.get(&era_number).unwrap();
        match validator_set.processing_status() {
            ValidatorSetProcessingStatus::CopyingFromLastEra {
                copying_validator_index,
                copying_delegator_index,
            } => {
                if era_number > 0 {
                    assert!(
                        validator_set_histories.contains(&(era_number - 1)),
                        "Missing validator set of last era"
                    );
                    let last_validator_set =
                        validator_set_histories.get(&(era_number - 1)).unwrap();
                    let mut validator_index = copying_validator_index.0;
                    let mut delegator_index = copying_delegator_index.0;
                    while env::used_gas() < GAS_CAP_FOR_MULTI_TXS_PROCESSING {
                        match self.copy_delegator_to_validator_set(
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
                                validator_set.set_total_stake(last_validator_set.total_stake());
                                validator_set.set_processing_status(
                                    ValidatorSetProcessingStatus::UnbondingValidator {
                                        unbonding_validator_index: U64::from(0),
                                        unbonding_delegator_index: U64::from(0),
                                    },
                                );
                                validator_set_histories.insert(&era_number, &validator_set);
                                return false;
                            }
                            ResultOfLoopingValidatorSet::NeedToContinue => delegator_index += 1,
                        }
                    }
                    validator_set.set_processing_status(
                        ValidatorSetProcessingStatus::CopyingFromLastEra {
                            copying_validator_index: U64::from(validator_index),
                            copying_delegator_index: U64::from(delegator_index),
                        },
                    );
                } else {
                    validator_set.set_processing_status(
                        ValidatorSetProcessingStatus::ApplyingStakingHistory {
                            applying_index: U64::from(0),
                        },
                    );
                }
                validator_set_histories.insert(&era_number, &validator_set);
                false
            }
            ValidatorSetProcessingStatus::UnbondingValidator {
                unbonding_validator_index,
                unbonding_delegator_index,
            } => {
                let mut validator_index = unbonding_validator_index.0;
                let mut delegator_index = unbonding_delegator_index.0;
                while env::used_gas() < GAS_CAP_FOR_MULTI_TXS_PROCESSING {
                    match self
                        .unbond_validator_of_next_validator_set(validator_index, delegator_index)
                    {
                        ResultOfLoopingValidatorSet::NoMoreDelegator => {
                            validator_index += 1;
                            delegator_index = 0;
                        }
                        ResultOfLoopingValidatorSet::NoMoreValidator => {
                            let mut next_validator_set = self.next_validator_set.get().unwrap();
                            next_validator_set.clear_unbonding_validator_ids();
                            self.next_validator_set.set(&next_validator_set);
                            validator_set.set_processing_status(
                                ValidatorSetProcessingStatus::AutoUnbondingValidator {
                                    unbonding_validator_index: U64::from(0),
                                    unbonding_delegator_index: U64::from(0),
                                },
                            );
                            validator_set_histories.insert(&era_number, &validator_set);
                            return false;
                        }
                        ResultOfLoopingValidatorSet::NeedToContinue => delegator_index += 1,
                    }
                }
                validator_set.set_processing_status(
                    ValidatorSetProcessingStatus::UnbondingValidator {
                        unbonding_validator_index: U64::from(validator_index),
                        unbonding_delegator_index: U64::from(delegator_index),
                    },
                );
                validator_set_histories.insert(&era_number, &validator_set);
                false
            }
            ValidatorSetProcessingStatus::AutoUnbondingValidator {
                unbonding_validator_index,
                unbonding_delegator_index,
            } => {
                let mut validator_index = unbonding_validator_index.0;
                let mut delegator_index = unbonding_delegator_index.0;
                while env::used_gas() < GAS_CAP_FOR_MULTI_TXS_PROCESSING {
                    match self.auto_unbond_validator_of_next_validator_set(
                        validator_index,
                        delegator_index,
                    ) {
                        ResultOfLoopingValidatorSet::NoMoreDelegator => {
                            validator_index += 1;
                            delegator_index = 0;
                        }
                        ResultOfLoopingValidatorSet::NoMoreValidator => {
                            let mut next_validator_set = self.next_validator_set.get().unwrap();
                            next_validator_set.clear_auto_unbonding_validator_ids();
                            self.next_validator_set.set(&next_validator_set);
                            let last_validator_set =
                                validator_set_histories.get(&(era_number - 1)).unwrap();
                            validator_set.set_processing_status(
                                ValidatorSetProcessingStatus::ApplyingStakingHistory {
                                    applying_index: U64::from(
                                        last_validator_set.staking_history_index() + 1,
                                    ),
                                },
                            );
                            validator_set.set_staking_history_index(
                                self.staking_histories
                                    .get()
                                    .unwrap()
                                    .index_range()
                                    .end_index
                                    .0,
                            );
                            validator_set_histories.insert(&era_number, &validator_set);
                            return false;
                        }
                        ResultOfLoopingValidatorSet::NeedToContinue => delegator_index += 1,
                    }
                }
                validator_set.set_processing_status(
                    ValidatorSetProcessingStatus::AutoUnbondingValidator {
                        unbonding_validator_index: U64::from(validator_index),
                        unbonding_delegator_index: U64::from(delegator_index),
                    },
                );
                validator_set_histories.insert(&era_number, &validator_set);
                false
            }
            ValidatorSetProcessingStatus::ApplyingStakingHistory { mut applying_index } => {
                while env::used_gas() < GAS_CAP_FOR_MULTI_TXS_PROCESSING
                    && applying_index.0 <= validator_set.staking_history_index()
                {
                    if let Some(staking_history) =
                        self.staking_histories.get().unwrap().get(&applying_index.0)
                    {
                        self.apply_staking_history_to_validator_set(
                            &mut validator_set,
                            &staking_history,
                        );
                    }
                    applying_index.0 += 1;
                }
                if applying_index.0 > validator_set.staking_history_index() {
                    validator_set.set_processing_status(
                        ValidatorSetProcessingStatus::ReadyForDistributingReward,
                    );
                } else {
                    validator_set.set_processing_status(
                        ValidatorSetProcessingStatus::ApplyingStakingHistory { applying_index },
                    );
                }
                validator_set_histories.insert(&era_number, &validator_set);
                false
            }
            _ => true,
        }
    }
    //
    fn copy_delegator_to_validator_set(
        &mut self,
        source_validator_set: &ValidatorSetOfEra,
        target_validator_set: &mut ValidatorSetOfEra,
        validator_index: u64,
        delegator_index: u64,
    ) -> ResultOfLoopingValidatorSet {
        let validator_ids = source_validator_set.get_validator_ids();
        if validator_index >= validator_ids.len().try_into().unwrap() {
            return ResultOfLoopingValidatorSet::NoMoreValidator;
        }
        let validator_id = validator_ids
            .get(usize::try_from(validator_index).unwrap())
            .unwrap();
        let validator = source_validator_set.get_validator(validator_id).unwrap();
        let delegator_ids = source_validator_set.get_delegator_ids_of(validator_id);
        if delegator_index >= delegator_ids.len().try_into().unwrap() {
            target_validator_set.insert_validator(&validator);
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
        let delegator_id = delegator_ids
            .get(usize::try_from(delegator_index).unwrap())
            .unwrap();
        let delegator = source_validator_set
            .get_delegator(&delegator_id, &validator_id)
            .unwrap();
        target_validator_set.insert_delegator(&delegator);
        return ResultOfLoopingValidatorSet::NeedToContinue;
    }
    //
    fn unbond_validator_of_next_validator_set(
        &mut self,
        validator_index: u64,
        delegator_index: u64,
    ) -> ResultOfLoopingValidatorSet {
        let next_validator_set = self.next_validator_set.get().unwrap();
        let validator_ids = next_validator_set.get_unbonding_validator_ids();
        if validator_index >= validator_ids.len().try_into().unwrap() {
            return ResultOfLoopingValidatorSet::NoMoreValidator;
        }
        let validator_id = validator_ids
            .get(usize::try_from(validator_index).unwrap())
            .unwrap();
        let validator = next_validator_set.get_validator(validator_id).unwrap();
        let delegator_ids = next_validator_set.get_delegator_ids_of(validator_id);
        if delegator_index >= delegator_ids.len().try_into().unwrap() {
            self.record_and_apply_staking_fact(StakingFact::ValidatorUnbonded {
                validator_id: validator_id.clone(),
                amount: U128::from(validator.deposit_amount),
            });
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
        let delegator_id = delegator_ids
            .get(usize::try_from(delegator_index).unwrap())
            .unwrap();
        let delegator = next_validator_set
            .get_delegator(&delegator_id, &validator_id)
            .unwrap();
        self.record_and_apply_staking_fact(StakingFact::DelegatorAutoUnbonded {
            delegator_id: delegator_id.clone(),
            validator_id: validator_id.clone(),
            amount: U128::from(delegator.deposit_amount),
        });
        return ResultOfLoopingValidatorSet::NeedToContinue;
    }
    //
    fn auto_unbond_validator_of_next_validator_set(
        &mut self,
        validator_index: u64,
        delegator_index: u64,
    ) -> ResultOfLoopingValidatorSet {
        let next_validator_set = self.next_validator_set.get().unwrap();
        let validator_ids = next_validator_set.get_auto_unbonding_validator_ids();
        if validator_index >= validator_ids.len().try_into().unwrap() {
            return ResultOfLoopingValidatorSet::NoMoreValidator;
        }
        let validator_id = validator_ids
            .get(usize::try_from(validator_index).unwrap())
            .unwrap();
        let validator = next_validator_set.get_validator(validator_id).unwrap();
        let delegator_ids = next_validator_set.get_delegator_ids_of(validator_id);
        if delegator_index >= delegator_ids.len().try_into().unwrap() {
            self.record_and_apply_staking_fact(StakingFact::ValidatorAutoUnbonded {
                validator_id: validator_id.clone(),
                amount: U128::from(validator.deposit_amount),
            });
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
        let delegator_id = delegator_ids
            .get(usize::try_from(delegator_index).unwrap())
            .unwrap();
        let delegator = next_validator_set
            .get_delegator(&delegator_id, &validator_id)
            .unwrap();
        self.record_and_apply_staking_fact(StakingFact::DelegatorAutoUnbonded {
            delegator_id: delegator_id.clone(),
            validator_id: validator_id.clone(),
            amount: U128::from(delegator.deposit_amount),
        });
        return ResultOfLoopingValidatorSet::NeedToContinue;
    }
    //
    fn apply_staking_history_to_validator_set(
        &mut self,
        validator_set: &mut ValidatorSetOfEra,
        staking_history: &StakingHistory,
    ) {
        validator_set.apply_staking_fact(&staking_history.staking_fact);
        match &staking_history.staking_fact {
            StakingFact::StakeDecreased {
                validator_id,
                amount: _,
            }
            | StakingFact::ValidatorUnbonded {
                validator_id,
                amount: _,
            }
            | StakingFact::ValidatorAutoUnbonded {
                validator_id,
                amount: _,
            } => {
                let mut stakes = match self.unbonded_stakes.get(validator_id) {
                    Some(stakes) => stakes,
                    None => Vec::<UnbondedStakeReference>::new(),
                };
                stakes.push(UnbondedStakeReference {
                    era_number: validator_set.era_number(),
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
                amount: _,
            }
            | StakingFact::DelegatorAutoUnbonded {
                delegator_id,
                validator_id: _,
                amount: _,
            } => {
                let mut stakes = match self.unbonded_stakes.get(delegator_id) {
                    Some(stakes) => stakes,
                    None => Vec::<UnbondedStakeReference>::new(),
                };
                stakes.push(UnbondedStakeReference {
                    era_number: validator_set.era_number(),
                    staking_history_index: staking_history.index.0,
                });
                self.unbonded_stakes.insert(delegator_id, &stakes);
            }
            _ => (),
        }
    }
}
