use crate::interfaces::SudoActions;
use crate::*;
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;

#[near_bindgen]
impl SudoActions for AppchainAnchor {
    //
    fn set_owner_pk(&mut self, public_key: PublicKey) {
        self.assert_owner();
        self.owner_pk = public_key;
    }
    //
    fn set_metadata_of_wrapped_appchain_token(&mut self, metadata: FungibleTokenMetadata) {
        self.assert_owner();
        let mut wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
        wrapped_appchain_token.metadata = metadata;
        self.wrapped_appchain_token.set(&wrapped_appchain_token);
    }
    //
    fn set_premined_balance_of_wrapped_appchain_token(
        &mut self,
        premined_beneficiary: AccountId,
        premined_balance: U128,
    ) {
        self.assert_owner();
        let mut wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
        wrapped_appchain_token.premined_beneficiary = Some(premined_beneficiary);
        wrapped_appchain_token.premined_balance = premined_balance;
        self.wrapped_appchain_token.set(&wrapped_appchain_token);
    }
    //
    fn regenerate_user_staking_histories(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_owner();
        let mut user_staking_histories = self.user_staking_histories.get().unwrap();
        let staking_histories = self.staking_histories.get().unwrap();
        let index_range = staking_histories.index_range();
        for index in index_range.start_index.0..index_range.end_index.0 + 1 {
            if let Some(staking_history) = staking_histories.get(&index) {
                user_staking_histories.add_staking_history(&staking_history);
            }
            if env::used_gas() > Gas::ONE_TERA.mul(T_GAS_CAP_FOR_MULTI_TXS_PROCESSING) {
                self.user_staking_histories.set(&user_staking_histories);
                return MultiTxsOperationProcessingResult::NeedMoreGas;
            }
        }
        self.user_staking_histories.set(&user_staking_histories);
        MultiTxsOperationProcessingResult::Ok
    }
    //
    fn reset_beefy_light_client(&mut self, initial_public_keys: Vec<String>) {
        self.assert_owner();
        self.beefy_light_client_state
            .set(&beefy_light_client::new(initial_public_keys));
    }
    //
    fn pause_asset_transfer(&mut self) {
        self.assert_owner();
        assert!(
            !self.asset_transfer_is_paused,
            "Asset transfer is already paused."
        );
        self.asset_transfer_is_paused = true;
    }
    //
    fn resume_asset_transfer(&mut self) {
        self.assert_owner();
        assert!(
            self.asset_transfer_is_paused,
            "Asset transfer is already resumed."
        );
        self.asset_transfer_is_paused = false;
    }
    //
    fn pause_rewards_withdrawal(&mut self) {
        self.assert_owner();
        assert!(
            !self.rewards_withdrawal_is_paused,
            "Rewards withdrawal is already paused."
        );
        self.rewards_withdrawal_is_paused = true;
    }
    //
    fn resume_rewards_withdrawal(&mut self) {
        self.assert_owner();
        assert!(
            self.rewards_withdrawal_is_paused,
            "Rewards withdrawal is already resumed."
        );
        self.rewards_withdrawal_is_paused = false;
    }
    //
    fn change_account_id_in_appchain_of_validator(
        &mut self,
        validator_id: AccountId,
        account_id_in_appchain: String,
    ) {
        self.assert_owner();
        self.internal_change_account_id_in_appchain_of_validator(
            &validator_id,
            &account_id_in_appchain,
        );
    }
    //
    fn set_latest_applied_appchain_message_nonce(&mut self, nonce: u32) {
        self.assert_owner();
        let mut permissionless_actions_status = self.permissionless_actions_status.get().unwrap();
        permissionless_actions_status.latest_applied_appchain_message_nonce = nonce;
        permissionless_actions_status.processing_appchain_message_nonce = None;
        self.permissionless_actions_status
            .set(&permissionless_actions_status);
    }
    //
    fn unlock_auto_unbonded_stake_of(
        &mut self,
        delegator_id: Option<AccountId>,
        validator_id: AccountId,
        staking_history_index: U64,
    ) {
        self.assert_owner();
        self.assert_asset_transfer_is_not_paused();
        let unbonded_stake_references = match delegator_id.clone() {
            Some(delegator_id) => self.unbonded_stakes.get(&delegator_id).unwrap(),
            None => self.unbonded_stakes.get(&validator_id).unwrap(),
        };
        let staking_histories = self.staking_histories.get().unwrap();
        let mut remained_stakes = Vec::<UnbondedStakeReference>::new();
        let mut found = false;
        for reference in unbonded_stake_references {
            if reference.staking_history_index == staking_history_index.0 {
                let staking_history = staking_histories.get(&staking_history_index.0).unwrap();
                match staking_history.staking_fact {
                    StakingFact::ValidatorAutoUnbonded {
                        validator_id: unbonded_validator_id @ _,
                        amount,
                    } => {
                        assert!(
                            validator_id.eq(&unbonded_validator_id),
                            "Invalid staking history for validator '{}'.",
                            validator_id
                        );
                        ext_ft_core::ext(self.oct_token.get().unwrap().contract_account)
                            .with_attached_deposit(1)
                            .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_FT_TRANSFER))
                            .with_unused_gas_weight(0)
                            .ft_transfer(validator_id.clone(), amount.clone(), None);
                        found = true;
                        break;
                    }
                    StakingFact::DelegatorAutoUnbonded {
                        delegator_id: unbonded_delegator_id @ _,
                        validator_id: unbonded_validator_id @ _,
                        amount,
                    } => {
                        let delegator_id = delegator_id
                            .clone()
                            .unwrap_or(AccountId::new_unchecked(String::new()));
                        assert!(
                            validator_id.eq(&unbonded_validator_id)
                                && delegator_id.eq(&unbonded_delegator_id),
                            "Invalid staking history for delegator '{}' of validator '{}'.",
                            delegator_id,
                            validator_id
                        );
                        ext_ft_core::ext(self.oct_token.get().unwrap().contract_account)
                            .with_attached_deposit(1)
                            .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_FT_TRANSFER))
                            .with_unused_gas_weight(0)
                            .ft_transfer(delegator_id.clone(), amount.clone(), None);
                        found = true;
                        break;
                    }
                    _ => {
                        remained_stakes.push(reference.clone());
                    }
                }
            } else {
                remained_stakes.push(reference.clone());
            }
        }
        assert!(found, "Specified staking history is not found.");
        if remained_stakes.len() > 0 {
            self.unbonded_stakes
                .insert(&delegator_id.unwrap_or(validator_id), &remained_stakes);
        } else {
            self.unbonded_stakes
                .remove(&delegator_id.unwrap_or(validator_id));
        }
    }
}
