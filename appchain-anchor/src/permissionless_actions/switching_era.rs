use super::{AppchainMessagesProcessingContext, ResultOfLoopingValidatorSet};
use crate::*;
use core::convert::TryFrom;
use user_actions::UnbondedStakeReference;

impl AppchainAnchor {
    //
    pub fn internal_start_switching_era(
        &mut self,
        processing_context: &mut AppchainMessagesProcessingContext,
        validator_set_histories: &mut LookupArray<ValidatorSetOfEra>,
        era_number: u64,
    ) -> MultiTxsOperationProcessingResult {
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
        }
        processing_context.set_switching_era_number(era_number);
        MultiTxsOperationProcessingResult::NeedMoreGas
    }
    //
    pub fn complete_switching_era(
        &mut self,
        processing_context: &mut AppchainMessagesProcessingContext,
        validator_set_histories: &mut LookupArray<ValidatorSetOfEra>,
        era_number: u64,
    ) -> MultiTxsOperationProcessingResult {
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
                    while processing_context.used_gas_of_current_function_call()
                        < Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING)
                    {
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
                                return MultiTxsOperationProcessingResult::NeedMoreGas;
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
                MultiTxsOperationProcessingResult::NeedMoreGas
            }
            ValidatorSetProcessingStatus::UnbondingValidator {
                unbonding_validator_index,
                unbonding_delegator_index,
            } => {
                let mut validator_index = unbonding_validator_index.0;
                let mut delegator_index = unbonding_delegator_index.0;
                while processing_context.used_gas_of_current_function_call()
                    < Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING)
                {
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
                            return MultiTxsOperationProcessingResult::NeedMoreGas;
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
                MultiTxsOperationProcessingResult::NeedMoreGas
            }
            ValidatorSetProcessingStatus::AutoUnbondingValidator {
                unbonding_validator_index,
                unbonding_delegator_index,
            } => {
                let mut validator_index = unbonding_validator_index.0;
                let mut delegator_index = unbonding_delegator_index.0;
                while processing_context.used_gas_of_current_function_call()
                    < Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING)
                {
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
                            return MultiTxsOperationProcessingResult::NeedMoreGas;
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
                MultiTxsOperationProcessingResult::NeedMoreGas
            }
            ValidatorSetProcessingStatus::ApplyingStakingHistory { mut applying_index } => {
                while processing_context.used_gas_of_current_function_call()
                    < Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING)
                    && applying_index.0 <= validator_set.staking_history_index()
                {
                    if let Some(staking_history) =
                        self.staking_histories.get().unwrap().get(&applying_index.0)
                    {
                        self.apply_staking_history_to_validator_set_of_era(
                            &mut validator_set,
                            &staking_history,
                        );
                    }
                    applying_index.0 += 1;
                }
                if applying_index.0 > validator_set.staking_history_index() {
                    processing_context.clear_switching_era_number();
                    validator_set.set_processing_status(
                        ValidatorSetProcessingStatus::ReadyForDistributingReward,
                    );
                    self.record_appchain_message_processing_result(
                        &AppchainMessageProcessingResult::Ok {
                            nonce: processing_context.processing_nonce().unwrap_or(0),
                            message: Some(format!(
                                "Validator set '{}' is generated and is ready for distributing reward.",
                                era_number
                            )),
                        },
                    );
                    validator_set_histories.insert(&era_number, &validator_set);
                    MultiTxsOperationProcessingResult::Ok
                } else {
                    validator_set.set_processing_status(
                        ValidatorSetProcessingStatus::ApplyingStakingHistory { applying_index },
                    );
                    validator_set_histories.insert(&era_number, &validator_set);
                    MultiTxsOperationProcessingResult::NeedMoreGas
                }
            }
            _ => MultiTxsOperationProcessingResult::Error(format!(
                "Wrong processing status '{:?}' of validator set '{}'.",
                validator_set.processing_status(),
                era_number
            )),
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
        if validator_index >= source_validator_set.validator_count() {
            return ResultOfLoopingValidatorSet::NoMoreValidator;
        }
        let validator = source_validator_set
            .get_validator_by_index(&validator_index)
            .unwrap();
        if delegator_index >= source_validator_set.get_delegator_count_of(&validator.validator_id) {
            target_validator_set.insert_validator(&validator);
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
        let delegator = source_validator_set
            .get_delegator_by_index(&delegator_index, &validator.validator_id)
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
        let unbonding_validator_ids = next_validator_set.get_unbonding_validator_ids();
        if validator_index >= unbonding_validator_ids.len().try_into().unwrap() {
            return ResultOfLoopingValidatorSet::NoMoreValidator;
        }
        let validator_id = unbonding_validator_ids
            .get(usize::from(u16::try_from(validator_index).unwrap()))
            .unwrap();
        let validator = next_validator_set
            .validator_set()
            .get_validator(&validator_id)
            .unwrap();
        if delegator_index
            >= next_validator_set
                .validator_set()
                .get_delegator_count_of(&validator.validator_id)
        {
            self.record_staking_fact(StakingFact::ValidatorUnbonded {
                validator_id: validator.validator_id,
                amount: U128::from(validator.deposit_amount),
            });
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
        let delegator = next_validator_set
            .validator_set()
            .get_delegator_by_index(&delegator_index, &validator.validator_id)
            .unwrap();
        self.record_staking_fact(StakingFact::DelegatorAutoUnbonded {
            delegator_id: delegator.delegator_id,
            validator_id: delegator.validator_id,
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
        let auto_unbonding_validator_ids = next_validator_set.get_auto_unbonding_validator_ids();
        if validator_index >= auto_unbonding_validator_ids.len().try_into().unwrap() {
            return ResultOfLoopingValidatorSet::NoMoreValidator;
        }
        let validator_id = auto_unbonding_validator_ids
            .get(usize::from(u16::try_from(validator_index).unwrap()))
            .unwrap();
        let validator = next_validator_set
            .validator_set()
            .get_validator(&validator_id)
            .unwrap();
        if delegator_index
            >= next_validator_set
                .validator_set()
                .get_delegator_count_of(&validator.validator_id)
        {
            self.record_staking_fact(StakingFact::ValidatorAutoUnbonded {
                validator_id: validator.validator_id,
                amount: U128::from(validator.deposit_amount),
            });
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
        let delegator = next_validator_set
            .validator_set()
            .get_delegator_by_index(&delegator_index, &validator.validator_id)
            .unwrap();
        self.record_staking_fact(StakingFact::DelegatorAutoUnbonded {
            delegator_id: delegator.delegator_id,
            validator_id: delegator.validator_id,
            amount: U128::from(delegator.deposit_amount),
        });
        return ResultOfLoopingValidatorSet::NeedToContinue;
    }
    //
    fn apply_staking_history_to_validator_set_of_era(
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
                self.add_unbonded_stake_of(
                    validator_id,
                    UnbondedStakeReference {
                        era_number: validator_set.era_number(),
                        staking_history_index: staking_history.index.0,
                    },
                );
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
                self.add_unbonded_stake_of(
                    delegator_id,
                    UnbondedStakeReference {
                        era_number: validator_set.era_number(),
                        staking_history_index: staking_history.index.0,
                    },
                );
            }
            _ => (),
        }
        match &staking_history.staking_fact {
            StakingFact::ValidatorUnbonded { .. }
            | StakingFact::ValidatorAutoUnbonded { .. }
            | StakingFact::DelegatorAutoUnbonded { .. } => {
                let mut next_validator_set = self.next_validator_set.get().unwrap();
                next_validator_set.apply_staking_fact(&staking_history.staking_fact);
                self.next_validator_set.set(&next_validator_set);
                self.sync_state_to_registry();
            }
            _ => (),
        }
    }
    //
    fn add_unbonded_stake_of(
        &mut self,
        account_id: &AccountId,
        unbonded_stake_reference: UnbondedStakeReference,
    ) {
        let mut stakes = match self.unbonded_stakes.get(account_id) {
            Some(stakes) => stakes,
            None => Vec::<UnbondedStakeReference>::new(),
        };
        stakes.push(unbonded_stake_reference);
        self.unbonded_stakes.insert(account_id, &stakes);
    }
}
