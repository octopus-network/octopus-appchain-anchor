use crate::{interfaces::StakingManager, *};
use borsh::maybestd::collections::HashMap;
use near_sdk::serde_json;

impl AppchainAnchor {
    //
    pub fn internal_process_oct_deposit(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        deposit_message: DepositMessage,
    ) -> PromiseOrValue<U128> {
        match deposit_message {
            DepositMessage::RegisterValidator {
                validator_id_in_appchain,
                can_be_delegated_to,
                profile,
            } => {
                self.register_validator(
                    sender_id,
                    validator_id_in_appchain,
                    profile,
                    amount,
                    can_be_delegated_to,
                );
                PromiseOrValue::Value(0.into())
            }
            DepositMessage::IncreaseStake => {
                self.increase_stake(sender_id, amount);
                PromiseOrValue::Value(0.into())
            }
            DepositMessage::RegisterDelegator { validator_id } => {
                self.register_delegator(sender_id, validator_id, amount);
                PromiseOrValue::Value(0.into())
            }
            DepositMessage::IncreaseDelegation { validator_id } => {
                self.increase_delegation(sender_id, validator_id, amount);
                PromiseOrValue::Value(0.into())
            }
            _ => panic!(
                "Internal error: misuse of internal function 'internal_process_oct_deposit'."
            ),
        }
    }
    //
    fn register_validator(
        &mut self,
        validator_id: AccountId,
        validator_id_in_appchain: Option<String>,
        profile: HashMap<String, String>,
        deposit_amount: U128,
        can_be_delegated_to: bool,
    ) {
        match self.appchain_state {
            AppchainState::Staging | AppchainState::Active => (),
            _ => panic!(
                "Cannot register validator while appchain state is '{}'.",
                serde_json::to_string(&self.appchain_state).unwrap()
            ),
        };
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        assert!(
            !next_validator_set.contains_validator(&validator_id),
            "The account '{}' has already been registered.",
            &validator_id
        );
        let mut validator_profiles = self.validator_profiles.get().unwrap();
        let formatted_validator_id_in_appchain =
            AccountIdInAppchain::new(validator_id_in_appchain.clone());
        if validator_id_in_appchain.is_some() {
            formatted_validator_id_in_appchain.assert_valid();
            if let Some(validator_profile) = validator_profiles
                .get_by_id_in_appchain(&formatted_validator_id_in_appchain.to_string())
            {
                assert!(
                    !next_validator_set.contains_validator(&validator_profile.validator_id),
                    "The account '{}' in appchain is already used by a validator in next era.",
                    &formatted_validator_id_in_appchain.origin_to_string()
                );
            }
        }
        self.assert_validator_stake_is_valid(deposit_amount.0, None);
        let protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            next_validator_set.validator_count() < protocol_settings.maximum_validator_count.0,
            "The count of registered validators exceeds the upper limit."
        );
        //
        if self.appchain_state.eq(&AppchainState::Active) {
            let validator_set_histories = self.validator_set_histories.get().unwrap();
            let latest_validator_set = validator_set_histories
                .get(&validator_set_histories.end_index)
                .unwrap();
            if latest_validator_set.validator_count() >= 12 {
                assert!(
                    next_validator_set.validator_count()
                        <= latest_validator_set.validator_count() * 5 / 4,
                    "Too many new validators in this era. Please try again in the next era."
                );
            }
        }
        //
        let staking_history = self.record_staking_fact(StakingFact::ValidatorRegistered {
            validator_id: validator_id.clone(),
            validator_id_in_appchain: formatted_validator_id_in_appchain.to_string(),
            amount: deposit_amount,
            can_be_delegated_to,
        });
        //
        next_validator_set.apply_staking_fact(&staking_history.staking_fact);
        self.next_validator_set.set(&next_validator_set);
        //
        self.sync_state_to_registry();
        //
        validator_profiles.insert(ValidatorProfile {
            validator_id,
            validator_id_in_appchain: formatted_validator_id_in_appchain.to_string(),
            profile,
        });
        self.validator_profiles.set(&validator_profiles);
    }
    //
    fn increase_stake(&mut self, validator_id: AccountId, amount: U128) {
        match self.appchain_state {
            AppchainState::Staging | AppchainState::Active => (),
            _ => panic!(
                "Cannot increase stake while appchain state is '{}'.",
                serde_json::to_string(&self.appchain_state).unwrap()
            ),
        };
        let protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            amount.0
                >= protocol_settings
                    .minimum_validator_deposit_changing_amount
                    .0,
            "Too few amount to increase. It should not be less than {} OCT.",
            protocol_settings
                .minimum_validator_deposit_changing_amount
                .0
                / OCT_DECIMALS_VALUE
        );
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        self.assert_validator_id(&validator_id, &next_validator_set);
        let validator = next_validator_set.get_validator(&validator_id).unwrap();
        self.assert_validator_stake_is_valid(
            validator.deposit_amount + amount.0,
            Some(validator.total_stake + amount.0),
        );
        let staking_history = self.record_staking_fact(StakingFact::StakeIncreased {
            validator_id,
            amount,
        });
        //
        next_validator_set.apply_staking_fact(&staking_history.staking_fact);
        self.next_validator_set.set(&next_validator_set);
        //
        self.sync_state_to_registry();
    }
    //
    fn register_delegator(
        &mut self,
        delegator_id: AccountId,
        validator_id: AccountId,
        deposit_amount: U128,
    ) {
        match self.appchain_state {
            AppchainState::Staging | AppchainState::Active => (),
            _ => panic!(
                "Cannot register delegator while appchain state is '{}'.",
                serde_json::to_string(&self.appchain_state).unwrap()
            ),
        };
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        self.assert_validator_id(&validator_id, &next_validator_set);
        assert!(
            !next_validator_set.contains_delegator(&delegator_id, &validator_id),
            "The account '{}' has already been registered to validator '{}'.",
            &delegator_id,
            &validator_id
        );
        let validator = next_validator_set.get_validator(&validator_id).unwrap();
        assert!(
            validator.can_be_delegated_to,
            "Validator '{}' cannot be delegated to.",
            &validator_id
        );
        let protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            next_validator_set.get_validator_count_of(&delegator_id)
                < protocol_settings.maximum_validators_per_delegator.0,
            "Too many validators delegated."
        );
        assert!(
            deposit_amount.0 >= protocol_settings.minimum_delegator_deposit.0,
            "The deposit for registering delegator is too few."
        );
        self.assert_validator_stake_is_valid(
            validator.deposit_amount,
            Some(validator.total_stake + deposit_amount.0),
        );
        let staking_history = self.record_staking_fact(StakingFact::DelegatorRegistered {
            delegator_id,
            validator_id,
            amount: U128::from(deposit_amount),
        });
        //
        next_validator_set.apply_staking_fact(&staking_history.staking_fact);
        self.next_validator_set.set(&next_validator_set);
        //
        self.sync_state_to_registry();
    }
    //
    pub fn record_staking_fact(&mut self, staking_fact: StakingFact) -> StakingHistory {
        //
        let mut staking_histories = self.staking_histories.get().unwrap();
        let staking_history = staking_histories.append(&mut StakingHistory {
            staking_fact,
            block_height: env::block_index(),
            timestamp: env::block_timestamp(),
            index: U64::from(0),
        });
        self.staking_histories.set(&staking_histories);
        //
        let mut user_staking_histories = self.user_staking_histories.get().unwrap();
        user_staking_histories.add_staking_history(&staking_history);
        self.user_staking_histories.set(&user_staking_histories);
        //
        staking_history
    }
    //
    fn increase_delegation(
        &mut self,
        delegator_id: AccountId,
        validator_id: AccountId,
        amount: U128,
    ) {
        match self.appchain_state {
            AppchainState::Staging | AppchainState::Active => (),
            _ => panic!(
                "Cannot increase delegation while appchain state is '{}'.",
                serde_json::to_string(&self.appchain_state).unwrap()
            ),
        };
        let protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            amount.0
                >= protocol_settings
                    .minimum_delegator_deposit_changing_amount
                    .0,
            "Too few amount to increase. It should not be less than {} OCT.",
            protocol_settings
                .minimum_delegator_deposit_changing_amount
                .0
                / OCT_DECIMALS_VALUE
        );
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        self.assert_delegator_id(&delegator_id, &validator_id, &next_validator_set);
        let validator = next_validator_set.get_validator(&validator_id).unwrap();
        self.assert_validator_stake_is_valid(
            validator.deposit_amount,
            Some(validator.total_stake + amount.0),
        );
        let staking_history = self.record_staking_fact(StakingFact::DelegationIncreased {
            delegator_id,
            validator_id,
            amount,
        });
        //
        next_validator_set.apply_staking_fact(&staking_history.staking_fact);
        self.next_validator_set.set(&next_validator_set);
        //
        self.sync_state_to_registry();
    }
}

#[near_bindgen]
impl StakingManager for AppchainAnchor {
    //
    fn decrease_stake(&mut self, amount: U128) {
        match self.appchain_state {
            AppchainState::Active => (),
            _ => panic!(
                "Cannot decrease stake while appchain state is '{}'.",
                serde_json::to_string(&self.appchain_state).unwrap()
            ),
        };
        let protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            amount.0
                >= protocol_settings
                    .minimum_validator_deposit_changing_amount
                    .0,
            "Too few amount to decrease. It should not be less than {} OCT.",
            protocol_settings
                .minimum_validator_deposit_changing_amount
                .0
                / OCT_DECIMALS_VALUE
        );
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        let validator_id = env::predecessor_account_id();
        self.assert_validator_id(&validator_id, &next_validator_set);
        let validator = next_validator_set.get_validator(&validator_id).unwrap();
        assert!(
            validator.deposit_amount > amount.0,
            "Unable to decrease so much stake."
        );
        self.assert_validator_stake_is_valid(validator.deposit_amount - amount.0, None);
        self.assert_total_stake_price(&protocol_settings, &next_validator_set, amount.0);
        let staking_history = self.record_staking_fact(StakingFact::StakeDecreased {
            validator_id: validator_id.clone(),
            amount,
        });
        //
        next_validator_set.apply_staking_fact(&staking_history.staking_fact);
        self.next_validator_set.set(&next_validator_set);
        //
        self.sync_state_to_registry();
    }
    //
    fn unbond_stake(&mut self) {
        match self.appchain_state {
            AppchainState::Active | AppchainState::Broken => (),
            _ => panic!(
                "Cannot unbond stake while appchain state is '{}'.",
                serde_json::to_string(&self.appchain_state).unwrap()
            ),
        };
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        let protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            next_validator_set.validator_count() > protocol_settings.minimum_validator_count.0,
            "Too few validators. Cannot unbond any more."
        );
        let validator_id = env::predecessor_account_id();
        self.assert_validator_id(&validator_id, &next_validator_set);
        self.record_unbonding_validator(
            &protocol_settings,
            &mut next_validator_set,
            &validator_id,
            false,
        );
        self.next_validator_set.set(&next_validator_set);
    }
    //
    fn enable_delegation(&mut self) {
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        let validator_id = env::predecessor_account_id();
        self.assert_validator_id(&validator_id, &next_validator_set);
        let staking_history =
            self.record_staking_fact(StakingFact::ValidatorDelegationEnabled { validator_id });
        //
        next_validator_set.apply_staking_fact(&staking_history.staking_fact);
        self.next_validator_set.set(&next_validator_set);
        //
        self.sync_state_to_registry();
    }
    //
    fn disable_delegation(&mut self) {
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        let validator_id = env::predecessor_account_id();
        self.assert_validator_id(&validator_id, &next_validator_set);
        let staking_history =
            self.record_staking_fact(StakingFact::ValidatorDelegationDisabled { validator_id });
        //
        next_validator_set.apply_staking_fact(&staking_history.staking_fact);
        self.next_validator_set.set(&next_validator_set);
        //
        self.sync_state_to_registry();
    }
    //
    fn decrease_delegation(&mut self, validator_id: AccountId, amount: U128) {
        match self.appchain_state {
            AppchainState::Active => (),
            _ => panic!(
                "Cannot decrease delegation while appchain state is '{}'.",
                serde_json::to_string(&self.appchain_state).unwrap()
            ),
        };
        let protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            amount.0
                >= protocol_settings
                    .minimum_delegator_deposit_changing_amount
                    .0,
            "Too few amount to decrease. It should not be less than {} OCT.",
            protocol_settings
                .minimum_delegator_deposit_changing_amount
                .0
                / OCT_DECIMALS_VALUE
        );
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        let delegator_id = env::predecessor_account_id();
        self.assert_delegator_id(&delegator_id, &validator_id, &next_validator_set);
        let protocol_settings = self.protocol_settings.get().unwrap();
        assert!(
            next_validator_set
                .get_delegator(&delegator_id, &validator_id)
                .unwrap()
                .deposit_amount
                >= protocol_settings.minimum_delegator_deposit.0 + amount.0,
            "Unable to decrease so much stake."
        );
        self.assert_total_stake_price(&protocol_settings, &next_validator_set, amount.0);
        let staking_history = self.record_staking_fact(StakingFact::DelegationDecreased {
            delegator_id: delegator_id.clone(),
            validator_id: validator_id.clone(),
            amount,
        });
        //
        next_validator_set.apply_staking_fact(&staking_history.staking_fact);
        self.next_validator_set.set(&next_validator_set);
        //
        self.sync_state_to_registry();
    }
    //
    fn unbond_delegation(&mut self, validator_id: AccountId) {
        match self.appchain_state {
            AppchainState::Active | AppchainState::Broken => (),
            _ => panic!(
                "Cannot unbond delegation while appchain state is '{}'.",
                serde_json::to_string(&self.appchain_state).unwrap()
            ),
        };
        let mut next_validator_set = self.next_validator_set.get().unwrap();
        let delegator_id = env::predecessor_account_id();
        self.assert_delegator_id(&delegator_id, &validator_id, &next_validator_set);
        let delegator = next_validator_set
            .get_delegator(&delegator_id, &validator_id)
            .unwrap();
        let protocol_settings = self.protocol_settings.get().unwrap();
        self.assert_total_stake_price(
            &protocol_settings,
            &next_validator_set,
            delegator.deposit_amount,
        );
        let staking_history = self.record_staking_fact(StakingFact::DelegatorUnbonded {
            delegator_id: delegator_id.clone(),
            validator_id: validator_id.clone(),
            amount: U128::from(delegator.deposit_amount),
        });
        //
        next_validator_set.apply_staking_fact(&staking_history.staking_fact);
        self.next_validator_set.set(&next_validator_set);
        //
        self.sync_state_to_registry();
    }
    //
    fn withdraw_stake(&mut self, account_id: AccountId) {
        self.assert_asset_transfer_is_not_paused();
        let protocol_settings = self.protocol_settings.get().unwrap();
        let mut balance_to_withdraw: u128 = 0;
        let mut remained_stakes = Vec::<UnbondedStakeReference>::new();
        if let Some(unbonded_stake_references) = self.unbonded_stakes.get(&account_id) {
            unbonded_stake_references.iter().for_each(|reference| {
                let validator_set = self
                    .validator_set_histories
                    .get()
                    .unwrap()
                    .get(&reference.era_number)
                    .unwrap();
                let staking_history = self
                    .staking_histories
                    .get()
                    .unwrap()
                    .get(&reference.staking_history_index)
                    .unwrap();
                match staking_history.staking_fact {
                    StakingFact::StakeDecreased {
                        validator_id: _,
                        amount,
                    }
                    | StakingFact::ValidatorUnbonded {
                        validator_id: _,
                        amount,
                    }
                    | StakingFact::ValidatorAutoUnbonded {
                        validator_id: _,
                        amount,
                    } => {
                        if validator_set.start_timestamp()
                            + protocol_settings.unlock_period_of_validator_deposit.0
                                * SECONDS_OF_A_DAY
                                * NANO_SECONDS_MULTIPLE
                            < env::block_timestamp()
                        {
                            balance_to_withdraw += amount.0;
                        } else {
                            remained_stakes.push(reference.clone());
                        }
                    }
                    StakingFact::DelegationDecreased {
                        delegator_id: _,
                        validator_id: _,
                        amount,
                    }
                    | StakingFact::DelegatorUnbonded {
                        delegator_id: _,
                        validator_id: _,
                        amount,
                    }
                    | StakingFact::DelegatorAutoUnbonded {
                        delegator_id: _,
                        validator_id: _,
                        amount,
                    } => {
                        if validator_set.start_timestamp()
                            + protocol_settings.unlock_period_of_delegator_deposit.0
                                * SECONDS_OF_A_DAY
                                * NANO_SECONDS_MULTIPLE
                            < env::block_timestamp()
                        {
                            balance_to_withdraw += amount.0;
                        } else {
                            remained_stakes.push(reference.clone());
                        }
                    }
                    _ => (),
                };
            });
            if remained_stakes.len() > 0 {
                self.unbonded_stakes.insert(&account_id, &remained_stakes);
            } else {
                self.unbonded_stakes.remove(&account_id);
            }
            if balance_to_withdraw > 0 {
                ext_fungible_token::ft_transfer(
                    account_id,
                    balance_to_withdraw.into(),
                    None,
                    &self.oct_token.get().unwrap().contract_account,
                    1,
                    GAS_FOR_FT_TRANSFER,
                );
            }
        };
    }
    //
    fn withdraw_validator_rewards(&mut self, validator_id: AccountId) {
        self.assert_asset_transfer_is_not_paused();
        self.assert_rewards_withdrawal_is_not_paused();
        let end_era = self
            .validator_set_histories
            .get()
            .unwrap()
            .index_range()
            .end_index
            .0;
        let protocol_settings = self.protocol_settings.get().unwrap();
        let start_era = match end_era > protocol_settings.maximum_era_count_of_unwithdrawn_reward.0
        {
            true => end_era - protocol_settings.maximum_era_count_of_unwithdrawn_reward.0 + 1,
            false => 0,
        };
        let mut reward_to_withdraw: u128 = 0;
        for era_number in start_era..end_era + 1 {
            if let Some(reward) = self
                .unwithdrawn_validator_rewards
                .get(&(era_number, validator_id.clone()))
            {
                reward_to_withdraw += reward;
                self.unwithdrawn_validator_rewards
                    .remove(&(era_number, validator_id.clone()));
            }
        }
        if reward_to_withdraw > 0 {
            ext_fungible_token::ft_transfer(
                validator_id,
                reward_to_withdraw.into(),
                None,
                &self.wrapped_appchain_token.get().unwrap().contract_account,
                1,
                GAS_FOR_FT_TRANSFER,
            );
        }
    }
    //
    fn withdraw_delegator_rewards(&mut self, delegator_id: AccountId, validator_id: AccountId) {
        self.assert_asset_transfer_is_not_paused();
        self.assert_rewards_withdrawal_is_not_paused();
        let end_era = self
            .validator_set_histories
            .get()
            .unwrap()
            .index_range()
            .end_index
            .0;
        let protocol_settings = self.protocol_settings.get().unwrap();
        let start_era = match end_era > protocol_settings.maximum_era_count_of_unwithdrawn_reward.0
        {
            true => end_era - protocol_settings.maximum_era_count_of_unwithdrawn_reward.0 + 1,
            false => 0,
        };
        let mut reward_to_withdraw: u128 = 0;
        for era_number in start_era..end_era {
            if let Some(reward) = self.unwithdrawn_delegator_rewards.get(&(
                era_number,
                delegator_id.clone(),
                validator_id.clone(),
            )) {
                reward_to_withdraw += reward;
                self.unwithdrawn_delegator_rewards.remove(&(
                    era_number,
                    delegator_id.clone(),
                    validator_id.clone(),
                ));
            }
        }
        if reward_to_withdraw > 0 {
            ext_fungible_token::ft_transfer(
                delegator_id,
                reward_to_withdraw.into(),
                None,
                &self.wrapped_appchain_token.get().unwrap().contract_account,
                1,
                GAS_FOR_FT_TRANSFER,
            );
        }
    }
}

impl AppchainAnchor {
    //
    fn assert_total_stake_price(
        &self,
        protocol_settings: &ProtocolSettings,
        next_validator_set: &NextValidatorSet,
        stake_reduction: u128,
    ) {
        let oct_token = self.oct_token.get().unwrap();
        assert!(
            next_validator_set.total_stake() > stake_reduction,
            "Not enough stake deposited in anchor."
        );
        assert!(
            (next_validator_set.total_stake() - stake_reduction) * oct_token.price_in_usd.0
                >= protocol_settings.minimum_total_stake_price_for_booting.0,
            "Market value of stake deposited in anchor is too low."
        );
    }
    //
    pub fn record_unbonding_validator(
        &self,
        protocol_settings: &ProtocolSettings,
        next_validator_set: &mut NextValidatorSet,
        validator_id: &AccountId,
        auto_unbond: bool,
    ) {
        if let Some(validator) = next_validator_set.get_validator(validator_id) {
            self.assert_total_stake_price(
                protocol_settings,
                next_validator_set,
                validator.total_stake,
            );
            if auto_unbond {
                next_validator_set.add_auto_unbonding_validator(validator_id);
            } else {
                next_validator_set.add_unbonding_validator(validator_id);
            }
        }
    }
}
