use super::ResultOfLoopingValidatorSet;
use crate::*;
use core::convert::{TryFrom, TryInto};

impl ValidatorSetProcessingStatus {
    ///
    pub fn can_distribute_reward(&self) -> bool {
        match self {
            ValidatorSetProcessingStatus::ReadyForDistributingReward
            | ValidatorSetProcessingStatus::Completed => true,
            _ => false,
        }
    }
}

impl AppchainAnchor {
    //
    pub fn internal_start_distributing_reward_of_era(
        &mut self,
        appchain_message_nonce: u32,
        era_number: u64,
        unprofitable_validator_ids: Vec<String>,
    ) -> AppchainMessageProcessingResult {
        let mut permissionless_actions_status = self.permissionless_actions_status.get().unwrap();
        if permissionless_actions_status
            .distributing_reward_era_number
            .is_some()
        {
            return AppchainMessageProcessingResult::Error {
                nonce: appchain_message_nonce,
                message: format!(
                    "Contract is still distributing reward for era {}.",
                    permissionless_actions_status
                        .distributing_reward_era_number
                        .unwrap()
                        .0
                ),
            };
        }
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        if !validator_set_histories.contains(&era_number) {
            return AppchainMessageProcessingResult::Error {
                nonce: appchain_message_nonce,
                message: format!("Validator set is not existed."),
            };
        }
        let mut validator_set = validator_set_histories.get(&era_number).unwrap();
        if !validator_set.processing_status().can_distribute_reward() {
            return AppchainMessageProcessingResult::Error {
                nonce: appchain_message_nonce,
                message: format!("Validator set is not ready for distributing reward."),
            };
        }
        let mut unprofitable_validator_ids_in_near = Vec::<AccountId>::new();
        let validator_profiles = self.validator_profiles.get().unwrap();
        for id_in_appchain in unprofitable_validator_ids {
            let account_id_in_appchain = AccountIdInAppchain::new(Some(id_in_appchain.clone()));
            match validator_profiles.get_by_id_in_appchain(&account_id_in_appchain.to_string()) {
                Some(validator_profile) => {
                    if validator_set.contains_validator(&validator_profile.validator_id) {
                        if !unprofitable_validator_ids_in_near
                            .contains(&validator_profile.validator_id)
                        {
                            unprofitable_validator_ids_in_near.push(validator_profile.validator_id);
                        }
                    } else {
                        return AppchainMessageProcessingResult::Error {
                            nonce: appchain_message_nonce,
                            message: format!("Validator id in appchain '{}' is not a valid validator in era '{}'.", id_in_appchain, era_number),
                        };
                    }
                }
                None => {
                    return AppchainMessageProcessingResult::Error {
                        nonce: appchain_message_nonce,
                        message: format!("Invalid validator id in appchain: '{}'", id_in_appchain),
                    }
                }
            }
        }
        validator_set.set_unprofitable_validator_ids(unprofitable_validator_ids_in_near);
        validator_set.calculate_valid_total_stake();
        validator_set.set_processing_status(ValidatorSetProcessingStatus::DistributingReward {
            appchain_message_nonce,
            distributing_validator_index: U64::from(0),
            distributing_delegator_index: U64::from(0),
        });
        validator_set_histories.insert(&era_number, &validator_set);
        permissionless_actions_status.distributing_reward_era_number = Some(U64::from(era_number));
        self.permissionless_actions_status
            .set(&permissionless_actions_status);
        // Mint `total_reward` in the contract of wrapped appchain token.
        let appchain_settings = self.appchain_settings.get().unwrap();
        self.internal_mint_wrapped_appchain_token(
            None,
            env::current_account_id(),
            appchain_settings.era_reward,
            appchain_message_nonce,
        )
    }
    //
    pub fn complete_distributing_reward_of_era(&mut self, era_number: u64) -> bool {
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        let mut validator_set = validator_set_histories.get(&era_number).unwrap();
        match validator_set.processing_status() {
            ValidatorSetProcessingStatus::CopyingFromLastEra { .. } => false,
            ValidatorSetProcessingStatus::UnbondingValidator { .. } => false,
            ValidatorSetProcessingStatus::AutoUnbondingValidator { .. } => false,
            ValidatorSetProcessingStatus::ApplyingStakingHistory { .. } => false,
            ValidatorSetProcessingStatus::ReadyForDistributingReward => false,
            ValidatorSetProcessingStatus::DistributingReward {
                appchain_message_nonce,
                distributing_validator_index,
                distributing_delegator_index,
            } => {
                let validator_commission_percent = u128::from(
                    self.protocol_settings
                        .get()
                        .unwrap()
                        .validator_commission_percent,
                );
                let mut validator_index = distributing_validator_index.0;
                let mut delegator_index = distributing_delegator_index.0;
                let era_reward = self.appchain_settings.get().unwrap().era_reward;
                while env::used_gas() < GAS_CAP_FOR_MULTI_TXS_PROCESSING {
                    match self.distribute_reward_in_validator_set(
                        appchain_message_nonce,
                        &mut validator_set,
                        validator_index,
                        delegator_index,
                        era_reward.0,
                        validator_commission_percent,
                    ) {
                        ResultOfLoopingValidatorSet::NoMoreDelegator => {
                            validator_index += 1;
                            delegator_index = 0;
                        }
                        ResultOfLoopingValidatorSet::NoMoreValidator => {
                            validator_set.set_processing_status(
                                ValidatorSetProcessingStatus::CheckingForAutoUnbondingValidator {
                                    unprofitable_validator_index: U64::from(0),
                                },
                            );
                            validator_set_histories.insert(&era_number, &validator_set);
                            return false;
                        }
                        ResultOfLoopingValidatorSet::NeedToContinue => delegator_index += 1,
                    }
                }
                validator_set.set_processing_status(
                    ValidatorSetProcessingStatus::DistributingReward {
                        appchain_message_nonce,
                        distributing_validator_index: U64::from(validator_index),
                        distributing_delegator_index: U64::from(delegator_index),
                    },
                );
                validator_set_histories.insert(&era_number, &validator_set);
                false
            }
            ValidatorSetProcessingStatus::CheckingForAutoUnbondingValidator {
                mut unprofitable_validator_index,
            } => {
                let unprofitable_validators = validator_set.unprofitable_validator_ids();
                let protocol_settings = self.protocol_settings.get().unwrap();
                let mut next_validator_set = self.next_validator_set.get().unwrap();
                while env::used_gas() < GAS_CAP_FOR_MULTI_TXS_PROCESSING {
                    if unprofitable_validator_index.0
                        >= unprofitable_validators.len().try_into().unwrap()
                    {
                        validator_set
                            .set_processing_status(ValidatorSetProcessingStatus::Completed);
                        validator_set_histories.insert(&era_number, &validator_set);
                        return false;
                    }
                    let validator_id = unprofitable_validators
                        .get(usize::try_from(unprofitable_validator_index.0).unwrap())
                        .unwrap();
                    if next_validator_set.contains_validator(validator_id) {
                        let start_checking_index = match era_number
                            >= u64::from(protocol_settings.maximum_allowed_unprofitable_era_count)
                        {
                            true => {
                                era_number
                                    - u64::from(
                                        protocol_settings.maximum_allowed_unprofitable_era_count,
                                    )
                                    + 1
                            }
                            false => 0,
                        };
                        let mut should_be_unbonded = true;
                        for index in start_checking_index..era_number {
                            if let Some(set_of_era) = validator_set_histories.get(&index) {
                                if !set_of_era
                                    .unprofitable_validator_ids()
                                    .contains(validator_id)
                                {
                                    should_be_unbonded = false;
                                    break;
                                }
                            }
                        }
                        if should_be_unbonded {
                            self.record_unbonding_validator(
                                &protocol_settings,
                                &mut next_validator_set,
                                validator_id,
                                true,
                            );
                            self.next_validator_set.set(&next_validator_set);
                        }
                    }
                    unprofitable_validator_index = U64::from(unprofitable_validator_index.0 + 1);
                }
                validator_set.set_processing_status(
                    ValidatorSetProcessingStatus::CheckingForAutoUnbondingValidator {
                        unprofitable_validator_index,
                    },
                );
                validator_set_histories.insert(&era_number, &validator_set);
                false
            }
            ValidatorSetProcessingStatus::Completed => true,
        }
    }
    //
    fn distribute_reward_in_validator_set(
        &mut self,
        appchain_message_nonce: u32,
        validator_set: &mut ValidatorSetOfEra,
        validator_index: u64,
        delegator_index: u64,
        era_reward: Balance,
        validator_commission_percent: u128,
    ) -> ResultOfLoopingValidatorSet {
        if validator_index >= validator_set.validator_count() {
            return ResultOfLoopingValidatorSet::NoMoreValidator;
        }
        let validator = validator_set
            .get_validator_by_index(&validator_index)
            .unwrap();
        if validator_set
            .unprofitable_validator_ids()
            .contains(&validator.validator_id)
        {
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
        let total_reward_of_validator = era_reward * (validator.total_stake / OCT_DECIMALS_VALUE)
            / (validator_set.valid_total_stake() / OCT_DECIMALS_VALUE);
        let validator_commission_reward =
            total_reward_of_validator * validator_commission_percent / 100;
        let mut reward_distribution_records = self.reward_distribution_records.get().unwrap();
        if delegator_index >= validator_set.get_delegator_count_of(&validator.validator_id) {
            let validator_reward = validator_commission_reward
                + (total_reward_of_validator - validator_commission_reward)
                    * (validator.deposit_amount / OCT_DECIMALS_VALUE)
                    / (validator.total_stake / OCT_DECIMALS_VALUE);
            self.add_reward_for_validator(validator_set, &validator.validator_id, validator_reward);
            reward_distribution_records.insert(
                appchain_message_nonce,
                validator_set.era_number(),
                &String::new(),
                &validator.validator_id,
            );
            self.reward_distribution_records
                .set(&reward_distribution_records);
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
        let delegator = validator_set
            .get_delegator_by_index(&delegator_index, &validator.validator_id)
            .unwrap();
        if !reward_distribution_records.contains_record(
            appchain_message_nonce,
            validator_set.era_number(),
            &delegator.delegator_id,
            &delegator.validator_id,
        ) {
            let delegator_reward = (total_reward_of_validator - validator_commission_reward)
                * (delegator.deposit_amount / OCT_DECIMALS_VALUE)
                / (validator.total_stake / OCT_DECIMALS_VALUE);
            self.add_reward_for_delegator(
                validator_set,
                &delegator.delegator_id,
                &delegator.validator_id,
                delegator_reward,
            );
            reward_distribution_records.insert(
                appchain_message_nonce,
                validator_set.era_number(),
                &delegator.delegator_id,
                &delegator.validator_id,
            );
            self.reward_distribution_records
                .set(&reward_distribution_records);
        }
        return ResultOfLoopingValidatorSet::NeedToContinue;
    }
    //
    fn add_reward_for_validator(
        &mut self,
        validator_set: &mut ValidatorSetOfEra,
        validator_id: &String,
        amount: u128,
    ) {
        let validator_reward = match validator_set.get_validator_rewards_of(validator_id) {
            Some(reward) => reward + amount,
            None => amount,
        };
        validator_set.set_validator_reward(validator_id, validator_reward);
        let unwithdrawn_validator_reward = match self
            .unwithdrawn_validator_rewards
            .get(&(validator_set.era_number(), validator_id.clone()))
        {
            Some(reward) => reward + amount,
            None => amount,
        };
        self.unwithdrawn_validator_rewards.insert(
            &(validator_set.era_number(), validator_id.clone()),
            &unwithdrawn_validator_reward,
        );
    }
    //
    fn add_reward_for_delegator(
        &mut self,
        validator_set: &mut ValidatorSetOfEra,
        delegator_id: &String,
        validator_id: &String,
        amount: u128,
    ) {
        let delegator_reward =
            match validator_set.get_delegator_rewards_of(&delegator_id, &validator_id) {
                Some(reward) => reward + amount,
                None => amount,
            };
        validator_set.set_delegator_reward(&delegator_id, &validator_id, delegator_reward);
        let unwithdrawn_delegator_reward = match self.unwithdrawn_delegator_rewards.get(&(
            validator_set.era_number(),
            delegator_id.clone(),
            validator_id.clone(),
        )) {
            Some(reward) => reward + amount,
            None => amount,
        };
        self.unwithdrawn_delegator_rewards.insert(
            &(
                validator_set.era_number(),
                delegator_id.clone(),
                validator_id.clone(),
            ),
            &unwithdrawn_delegator_reward,
        );
    }
}
