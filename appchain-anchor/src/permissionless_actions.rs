use crate::*;
use crate::{interfaces::PermissionlessActions, message_decoder::AppchainMessage};
use core::convert::{TryFrom, TryInto};
use staking::UnbondedStakeReference;
use validator_set::*;

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainEvent {
    /// The fact that a certain amount of bridge token has been burnt in the appchain.
    NearFungibleTokenBurnt {
        symbol: String,
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        amount: U128,
    },
    /// The fact that a certain amount of appchain native token has been locked in the appchain.
    NativeTokenLocked {
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        amount: U128,
    },
    /// The fact that the era switch is planed in the appchain.
    EraSwitchPlaned { era_number: u32 },
    /// The fact that the total reward and unprofitable validator list
    /// is concluded in the appchain.
    EraRewardConcluded {
        era_number: u32,
        unprofitable_validator_ids: Vec<String>,
    },
}

enum ResultOfLoopingValidatorSet {
    NoMoreDelegator,
    NoMoreValidator,
    NeedToContinue,
}

#[near_bindgen]
impl PermissionlessActions for AppchainAnchor {
    ///
    fn start_updating_state_of_beefy_light_client(
        &mut self,
        signed_commitment: Vec<u8>,
        validator_proofs: Vec<ValidatorMerkleProof>,
        mmr_leaf: Vec<u8>,
        mmr_proof: Vec<u8>,
    ) {
        let anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            !anchor_settings.beefy_light_client_witness_mode,
            "Beefy light client is in witness mode."
        );
        self.assert_light_client_is_ready();
        let mut light_client = self.beefy_light_client_state.get().unwrap();
        if let Err(err) = light_client.start_updating_state(
            &signed_commitment,
            &validator_proofs
                .iter()
                .map(|proof| beefy_light_client::ValidatorMerkleProof {
                    proof: proof.proof.clone(),
                    number_of_leaves: proof.number_of_leaves.try_into().unwrap_or_default(),
                    leaf_index: proof.leaf_index.try_into().unwrap_or_default(),
                    leaf: proof.leaf.clone(),
                })
                .collect::<Vec<beefy_light_client::ValidatorMerkleProof>>(),
            &mmr_leaf,
            &mmr_proof,
        ) {
            panic!(
                "Failed to start updating state of beefy light client: {:?}",
                err
            );
        }
        self.beefy_light_client_state.set(&light_client);
    }
    //
    fn try_complete_updating_state_of_beefy_light_client(
        &mut self,
    ) -> MultiTxsOperationProcessingResult {
        let anchor_settings = self.anchor_settings.get().unwrap();
        assert!(
            !anchor_settings.beefy_light_client_witness_mode,
            "Beefy light client is in witness mode."
        );
        self.assert_light_client_initialized();
        let mut light_client = self.beefy_light_client_state.get().unwrap();
        if !light_client.is_updating_state() {
            return MultiTxsOperationProcessingResult::Ok;
        }
        loop {
            match light_client.complete_updating_state(1) {
                Ok(flag) => match flag {
                    true => {
                        self.beefy_light_client_state.set(&light_client);
                        return MultiTxsOperationProcessingResult::Ok;
                    }
                    false => (),
                },
                Err(err) => {
                    self.beefy_light_client_state.set(&light_client);
                    return MultiTxsOperationProcessingResult::Error(format!("{:?}", err));
                }
            }
            if env::used_gas() > GAS_CAP_FOR_MULTI_TXS_PROCESSING {
                break;
            }
        }
        self.beefy_light_client_state.set(&light_client);
        MultiTxsOperationProcessingResult::NeedMoreGas
    }
    //
    fn verify_and_apply_appchain_messages(
        &mut self,
        encoded_messages: Vec<u8>,
        header: Vec<u8>,
        mmr_leaf: Vec<u8>,
        mmr_proof: Vec<u8>,
    ) -> Vec<AppchainMessageProcessingResult> {
        let anchor_settings = self.anchor_settings.get().unwrap();
        if anchor_settings.beefy_light_client_witness_mode {
            assert!(
                env::predecessor_account_id().eq(&anchor_settings.relayer_account),
                "Only relayer account can perform this action while beefy light client is in witness mode."
            );
        } else {
            self.assert_light_client_is_ready();
            let light_client = self.beefy_light_client_state.get().unwrap();
            if let Err(err) = light_client.verify_solochain_messages(
                &encoded_messages,
                &header,
                &mmr_leaf,
                &mmr_proof,
            ) {
                panic!("Failed in verifying appchain messages: {:?}", err);
            }
        }
        let messages = message_decoder::decode(encoded_messages);
        messages
            .iter()
            .map(|m| self.internal_apply_appchain_message(m.clone()))
            .collect::<Vec<AppchainMessageProcessingResult>>()
    }
    //
    fn try_complete_switching_era(&mut self) -> MultiTxsOperationProcessingResult {
        match self
            .permissionless_actions_status
            .get()
            .unwrap()
            .switching_era_number
        {
            Some(era_number) => {
                let completed = self.complete_switching_era(era_number.0);
                if completed {
                    let mut permissionless_actions_status =
                        self.permissionless_actions_status.get().unwrap();
                    permissionless_actions_status.switching_era_number = None;
                    self.permissionless_actions_status
                        .set(&permissionless_actions_status);
                    MultiTxsOperationProcessingResult::Ok
                } else {
                    MultiTxsOperationProcessingResult::NeedMoreGas
                }
            }
            None => MultiTxsOperationProcessingResult::Ok,
        }
    }
    //
    fn try_complete_distributing_reward(&mut self) -> MultiTxsOperationProcessingResult {
        match self
            .permissionless_actions_status
            .get()
            .unwrap()
            .distributing_reward_era_number
        {
            Some(era_number) => {
                let completed = self.complete_distributing_reward_of_era(era_number.0);
                if completed {
                    let mut permissionless_actions_status =
                        self.permissionless_actions_status.get().unwrap();
                    permissionless_actions_status.distributing_reward_era_number = None;
                    self.permissionless_actions_status
                        .set(&permissionless_actions_status);
                    MultiTxsOperationProcessingResult::Ok
                } else {
                    MultiTxsOperationProcessingResult::NeedMoreGas
                }
            }
            None => MultiTxsOperationProcessingResult::Ok,
        }
    }
}

impl AppchainAnchor {
    /// Apply a certain `AppchainMessage`
    pub fn internal_apply_appchain_message(
        &mut self,
        appchain_message: AppchainMessage,
    ) -> AppchainMessageProcessingResult {
        match appchain_message.appchain_event {
            permissionless_actions::AppchainEvent::NearFungibleTokenBurnt {
                symbol,
                owner_id_in_appchain,
                receiver_id_in_near,
                amount,
            } => {
                if self.asset_transfer_is_paused {
                    return AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message: format!("Asset transfer is now paused."),
                    };
                }
                self.internal_unlock_near_fungible_token(
                    owner_id_in_appchain,
                    symbol,
                    receiver_id_in_near,
                    amount,
                    appchain_message.nonce,
                )
            }
            permissionless_actions::AppchainEvent::NativeTokenLocked {
                owner_id_in_appchain,
                receiver_id_in_near,
                amount,
            } => {
                if self.asset_transfer_is_paused {
                    return AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message: format!("Asset transfer is now paused."),
                    };
                }
                let wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
                let protocol_settings = self.protocol_settings.get().unwrap();
                let owner_id = AccountIdInAppchain::new(Some(owner_id_in_appchain.clone()));
                owner_id.assert_valid();
                if wrapped_appchain_token.total_market_value()
                    + wrapped_appchain_token.get_market_value_of(amount.0)
                    > self.get_market_value_of_staked_oct_token().0
                        * u128::from(
                            protocol_settings
                                .maximum_market_value_percent_of_wrapped_appchain_token,
                        )
                        / 100
                {
                    let message = format!("Too much wrapped appchain token to mint.");
                    self.internal_append_anchor_event(
                        AnchorEvent::FailedToMintWrappedAppchainToken {
                            sender_id_in_appchain: Some(owner_id.to_string()),
                            receiver_id_in_near,
                            amount,
                            appchain_message_nonce: appchain_message.nonce,
                            reason: message.clone(),
                        },
                    );
                    AppchainMessageProcessingResult::Error {
                        nonce: appchain_message.nonce,
                        message,
                    }
                } else {
                    self.internal_mint_wrapped_appchain_token(
                        Some(owner_id.to_string()),
                        receiver_id_in_near,
                        amount,
                        appchain_message.nonce,
                    )
                }
            }
            permissionless_actions::AppchainEvent::EraSwitchPlaned { era_number } => {
                self.assert_era_number_is_valid(u64::from(era_number));
                self.internal_start_switching_era(u64::from(era_number), appchain_message.nonce)
            }
            permissionless_actions::AppchainEvent::EraRewardConcluded {
                era_number,
                unprofitable_validator_ids,
            } => {
                self.assert_era_number_is_valid(u64::from(era_number));
                self.internal_start_distributing_reward_of_era(
                    appchain_message.nonce,
                    u64::from(era_number),
                    unprofitable_validator_ids,
                )
            }
        }
    }
    //
    fn assert_era_number_is_valid(&self, era_number: u64) {
        let protocol_settings = self.protocol_settings.get().unwrap();
        let validator_set_histories = self.validator_set_histories.get().unwrap();
        let latest_era_number = validator_set_histories.index_range().end_index.0;
        if latest_era_number
            > protocol_settings
                .maximum_era_count_of_valid_appchain_message
                .0
        {
            assert!(
                era_number
                    >= latest_era_number
                        - protocol_settings
                            .maximum_era_count_of_valid_appchain_message
                            .0,
                "Message is too old."
            );
        }
    }
}

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
    fn complete_switching_era(&mut self, era_number: u64) -> bool {
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        let mut validator_set = validator_set_histories.get(&era_number).unwrap();
        match validator_set.processing_status {
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
                                validator_set.validator_set.total_stake =
                                    last_validator_set.validator_set.total_stake;
                                validator_set.processing_status =
                                    ValidatorSetProcessingStatus::ApplyingStakingHistory {
                                        applying_index: U64::from(
                                            last_validator_set.staking_history_index + 1,
                                        ),
                                    };
                                validator_set_histories.insert(&era_number, &validator_set);
                                return false;
                            }
                            ResultOfLoopingValidatorSet::NeedToContinue => delegator_index += 1,
                        }
                    }
                    validator_set.processing_status =
                        ValidatorSetProcessingStatus::CopyingFromLastEra {
                            copying_validator_index: U64::from(validator_index),
                            copying_delegator_index: U64::from(delegator_index),
                        };
                } else {
                    validator_set.processing_status =
                        ValidatorSetProcessingStatus::ApplyingStakingHistory {
                            applying_index: U64::from(0),
                        };
                }
                validator_set_histories.insert(&era_number, &validator_set);
                false
            }
            ValidatorSetProcessingStatus::ApplyingStakingHistory { mut applying_index } => {
                while env::used_gas() < GAS_CAP_FOR_MULTI_TXS_PROCESSING
                    && applying_index.0 <= validator_set.staking_history_index
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
                if applying_index.0 > validator_set.staking_history_index {
                    validator_set.processing_status =
                        ValidatorSetProcessingStatus::ReadyForDistributingReward;
                } else {
                    validator_set.processing_status =
                        ValidatorSetProcessingStatus::ApplyingStakingHistory { applying_index };
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
        let validator_ids = source_validator_set.validator_set.validator_id_set.to_vec();
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
            .validator_id_set
            .contains(validator_id)
        {
            target_validator_set
                .validator_set
                .validator_id_set
                .insert(validator_id);
            target_validator_set
                .validator_set
                .validators
                .insert(validator_id, &validator);
        }
        if let Some(delegator_id_set) = source_validator_set
            .validator_set
            .validator_id_to_delegator_id_set
            .get(validator_id)
        {
            let delegator_ids = delegator_id_set.to_vec();
            if delegator_index >= delegator_ids.len().try_into().unwrap() {
                return ResultOfLoopingValidatorSet::NoMoreDelegator;
            }
            let delegator_id = delegator_ids
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
            //
            if !target_validator_set
                .validator_set
                .validator_id_to_delegator_id_set
                .contains_key(validator_id)
            {
                target_validator_set
                    .validator_set
                    .validator_id_to_delegator_id_set
                    .insert(
                        validator_id,
                        &UnorderedSet::new(
                            StorageKey::DelegatorIdsInMapOfVToDOfEra {
                                era_number: target_validator_set.validator_set.era_number,
                                validator_id: validator_id.clone(),
                            }
                            .into_bytes(),
                        ),
                    );
            }
            let mut delegator_id_set = target_validator_set
                .validator_set
                .validator_id_to_delegator_id_set
                .get(validator_id)
                .unwrap();
            delegator_id_set.insert(delegator_id);
            target_validator_set
                .validator_set
                .validator_id_to_delegator_id_set
                .insert(validator_id, &delegator_id_set);
            //
            if !target_validator_set
                .validator_set
                .delegator_id_to_validator_id_set
                .contains_key(delegator_id)
            {
                target_validator_set
                    .validator_set
                    .delegator_id_to_validator_id_set
                    .insert(
                        delegator_id,
                        &UnorderedSet::new(
                            StorageKey::ValidatorIdsInMapOfDToVOfEra {
                                era_number: target_validator_set.validator_set.era_number,
                                delegator_id: delegator_id.clone(),
                            }
                            .into_bytes(),
                        ),
                    );
            }
            let mut validator_id_set = target_validator_set
                .validator_set
                .delegator_id_to_validator_id_set
                .get(delegator_id)
                .unwrap();
            validator_id_set.insert(validator_id);
            target_validator_set
                .validator_set
                .delegator_id_to_validator_id_set
                .insert(delegator_id, &validator_id_set);
            return ResultOfLoopingValidatorSet::NeedToContinue;
        } else {
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
    }
    //
    fn apply_staking_history_to_validator_set(
        &mut self,
        validator_set: &mut ValidatorSetOfEra,
        staking_history: &StakingHistory,
    ) {
        validator_set.apply_staking_history(staking_history);
        match &staking_history.staking_fact {
            StakingFact::StakeDecreased {
                validator_id,
                amount: _,
            }
            | StakingFact::ValidatorUnbonded {
                validator_id,
                amount: _,
            } => {
                let mut stakes = match self.unbonded_stakes.get(validator_id) {
                    Some(stakes) => stakes,
                    None => Vec::<UnbondedStakeReference>::new(),
                };
                stakes.push(UnbondedStakeReference {
                    era_number: validator_set.validator_set.era_number,
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
            } => {
                let mut stakes = match self.unbonded_stakes.get(delegator_id) {
                    Some(stakes) => stakes,
                    None => Vec::<UnbondedStakeReference>::new(),
                };
                stakes.push(UnbondedStakeReference {
                    era_number: validator_set.validator_set.era_number,
                    staking_history_index: staking_history.index.0,
                });
                self.unbonded_stakes.insert(delegator_id, &stakes);
            }
            _ => (),
        }
    }
}

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
        if !validator_set.processing_status.can_distribute_reward() {
            return AppchainMessageProcessingResult::Error {
                nonce: appchain_message_nonce,
                message: format!("Validator set is not ready for distributing reward."),
            };
        }
        let mut unprofitable_validator_ids_in_near = Vec::<AccountId>::new();
        let validator_profiles = self.validator_profiles.get().unwrap();
        for id_in_appchain in unprofitable_validator_ids {
            let account_id_in_appchain = AccountIdInAppchain::new(Some(id_in_appchain.clone()));
            account_id_in_appchain.assert_valid();
            if validator_profiles
                .get_by_id_in_appchain(&account_id_in_appchain.to_string())
                .is_none()
            {
                return AppchainMessageProcessingResult::Error {
                    nonce: appchain_message_nonce,
                    message: format!("Invalid validator id in appchain: '{}'", id_in_appchain),
                };
            }
            unprofitable_validator_ids_in_near.push(
                validator_profiles
                    .get_by_id_in_appchain(&account_id_in_appchain.to_string())
                    .unwrap()
                    .validator_id,
            );
        }
        validator_set.set_unprofitable_validator_ids(unprofitable_validator_ids_in_near);
        validator_set.calculate_valid_total_stake();
        validator_set.processing_status = ValidatorSetProcessingStatus::DistributingReward {
            appchain_message_nonce,
            distributing_validator_index: U64::from(0),
            distributing_delegator_index: U64::from(0),
        };
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
        );
        AppchainMessageProcessingResult::Ok {
            nonce: appchain_message_nonce,
            message: None,
        }
    }
    //
    fn complete_distributing_reward_of_era(&mut self, era_number: u64) -> bool {
        let mut validator_set_histories = self.validator_set_histories.get().unwrap();
        let mut validator_set = validator_set_histories.get(&era_number).unwrap();
        match validator_set.processing_status {
            ValidatorSetProcessingStatus::CopyingFromLastEra {
                copying_validator_index: _,
                copying_delegator_index: _,
            } => false,
            ValidatorSetProcessingStatus::ApplyingStakingHistory { applying_index: _ } => false,
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
                            validator_set.processing_status =
                                ValidatorSetProcessingStatus::AutoUnbondingValidator {
                                    unprofitable_validator_index: U64::from(0),
                                };
                            validator_set_histories.insert(&era_number, &validator_set);
                            return false;
                        }
                        ResultOfLoopingValidatorSet::NeedToContinue => delegator_index += 1,
                    }
                }
                validator_set.processing_status =
                    ValidatorSetProcessingStatus::DistributingReward {
                        appchain_message_nonce,
                        distributing_validator_index: U64::from(validator_index),
                        distributing_delegator_index: U64::from(delegator_index),
                    };
                validator_set_histories.insert(&era_number, &validator_set);
                false
            }
            ValidatorSetProcessingStatus::AutoUnbondingValidator {
                mut unprofitable_validator_index,
            } => {
                let unprofitable_validators = validator_set.unprofitable_validator_id_set.to_vec();
                let protocol_settings = self.protocol_settings.get().unwrap();
                while env::used_gas() < GAS_CAP_FOR_MULTI_TXS_PROCESSING {
                    if unprofitable_validator_index.0
                        >= unprofitable_validators.len().try_into().unwrap()
                    {
                        validator_set.processing_status = ValidatorSetProcessingStatus::Completed;
                        validator_set_histories.insert(&era_number, &validator_set);
                        return false;
                    }
                    let validator_id = unprofitable_validators
                        .get(usize::try_from(unprofitable_validator_index.0).unwrap())
                        .unwrap();
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
                                .unprofitable_validator_id_set
                                .contains(validator_id)
                            {
                                should_be_unbonded = false;
                                break;
                            }
                        }
                    }
                    if should_be_unbonded {
                        self.internal_unbond_validator(validator_id, true);
                    }
                    unprofitable_validator_index = U64::from(unprofitable_validator_index.0 + 1);
                }
                validator_set.processing_status =
                    ValidatorSetProcessingStatus::AutoUnbondingValidator {
                        unprofitable_validator_index,
                    };
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
        let validator_ids = validator_set.validator_set.validator_id_set.to_vec();
        if validator_index >= validator_ids.len().try_into().unwrap() {
            return ResultOfLoopingValidatorSet::NoMoreValidator;
        }
        let validator_id = validator_ids
            .get(usize::try_from(validator_index).unwrap())
            .unwrap();
        if validator_set
            .unprofitable_validator_id_set
            .contains(validator_id)
        {
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
        let validator = validator_set
            .validator_set
            .validators
            .get(validator_id)
            .unwrap();
        let total_reward_of_validator = era_reward * (validator.total_stake / OCT_DECIMALS_VALUE)
            / (validator_set.valid_total_stake / OCT_DECIMALS_VALUE);
        let validator_commission_reward =
            total_reward_of_validator * validator_commission_percent / 100;
        let mut reward_distribution_records = self.reward_distribution_records.get().unwrap();
        if !reward_distribution_records.contains_record(
            appchain_message_nonce,
            validator_set.validator_set.era_number,
            &String::new(),
            validator_id,
        ) {
            let validator_reward = validator_commission_reward
                + (total_reward_of_validator - validator_commission_reward)
                    * (validator.deposit_amount / OCT_DECIMALS_VALUE)
                    / (validator.total_stake / OCT_DECIMALS_VALUE);
            self.add_reward_for_validator(validator_set, validator_id, validator_reward);
            reward_distribution_records.insert(
                appchain_message_nonce,
                validator_set.validator_set.era_number,
                &String::new(),
                validator_id,
            );
            self.reward_distribution_records
                .set(&reward_distribution_records);
        }
        if let Some(delegator_id_set) = validator_set
            .validator_set
            .validator_id_to_delegator_id_set
            .get(&validator_id)
        {
            let delegater_ids = delegator_id_set.to_vec();
            if delegator_index >= delegater_ids.len().try_into().unwrap() {
                return ResultOfLoopingValidatorSet::NoMoreDelegator;
            }
            let delegator_id = delegater_ids
                .get(usize::try_from(delegator_index).unwrap())
                .unwrap();
            if !reward_distribution_records.contains_record(
                appchain_message_nonce,
                validator_set.validator_set.era_number,
                delegator_id,
                validator_id,
            ) {
                let delegator = validator_set
                    .validator_set
                    .delegators
                    .get(&(delegator_id.clone(), validator_id.clone()))
                    .unwrap();
                let delegator_reward = (total_reward_of_validator - validator_commission_reward)
                    * (delegator.deposit_amount / OCT_DECIMALS_VALUE)
                    / (validator.total_stake / OCT_DECIMALS_VALUE);
                self.add_reward_for_delegator(
                    validator_set,
                    delegator_id,
                    validator_id,
                    delegator_reward,
                );
                reward_distribution_records.insert(
                    appchain_message_nonce,
                    validator_set.validator_set.era_number,
                    delegator_id,
                    validator_id,
                );
                self.reward_distribution_records
                    .set(&reward_distribution_records);
            }
            return ResultOfLoopingValidatorSet::NeedToContinue;
        } else {
            return ResultOfLoopingValidatorSet::NoMoreDelegator;
        }
    }
    //
    fn add_reward_for_validator(
        &mut self,
        validator_set: &mut ValidatorSetOfEra,
        validator_id: &String,
        amount: u128,
    ) {
        let validator_reward = match validator_set.validator_rewards.get(validator_id) {
            Some(reward) => reward + amount,
            None => amount,
        };
        validator_set
            .validator_rewards
            .insert(validator_id, &validator_reward);
        let unwithdrawn_validator_reward = match self
            .unwithdrawn_validator_rewards
            .get(&(validator_set.validator_set.era_number, validator_id.clone()))
        {
            Some(reward) => reward + amount,
            None => amount,
        };
        self.unwithdrawn_validator_rewards.insert(
            &(validator_set.validator_set.era_number, validator_id.clone()),
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
        let delegator_reward = match validator_set
            .delegator_rewards
            .get(&(delegator_id.clone(), validator_id.clone()))
        {
            Some(reward) => reward + amount,
            None => amount,
        };
        validator_set.delegator_rewards.insert(
            &(delegator_id.clone(), validator_id.clone()),
            &delegator_reward,
        );
        let unwithdrawn_delegator_reward = match self.unwithdrawn_delegator_rewards.get(&(
            validator_set.validator_set.era_number,
            delegator_id.clone(),
            validator_id.clone(),
        )) {
            Some(reward) => reward + amount,
            None => amount,
        };
        self.unwithdrawn_delegator_rewards.insert(
            &(
                validator_set.validator_set.era_number,
                delegator_id.clone(),
                validator_id.clone(),
            ),
            &unwithdrawn_delegator_reward,
        );
    }
}
