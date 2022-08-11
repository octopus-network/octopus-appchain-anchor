use crate::*;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_contract_standards::non_fungible_token::metadata::NFTContractMetadata;
use near_sdk::borsh::maybestd::collections::HashMap;
use near_sdk::json_types::I128;

pub type AppchainId = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainTemplateType {
    Barnacle,
    BarnacleEvm,
}

pub struct AccountIdInAppchain {
    appchain_template_type: AppchainTemplateType,
    origin: Option<String>,
    raw_string: String,
}

impl AccountIdInAppchain {
    ///
    pub fn new(
        id_in_appchain: Option<String>,
        appchain_template_type: &AppchainTemplateType,
    ) -> Self {
        let mut value = String::new();
        if let Some(id_in_appchain) = id_in_appchain.clone() {
            if !id_in_appchain.to_lowercase().starts_with("0x") {
                value.push_str("0x");
            }
            value.push_str(&id_in_appchain);
        }
        Self {
            appchain_template_type: appchain_template_type.clone(),
            origin: id_in_appchain,
            raw_string: value.to_lowercase(),
        }
    }
    ///
    pub fn is_valid(&self) -> bool {
        if self.raw_string.len() > 2 {
            match hex::decode(&self.raw_string.as_str()[2..self.raw_string.len()]) {
                Ok(bytes) => match self.appchain_template_type {
                    AppchainTemplateType::Barnacle => bytes.len() == 32,
                    AppchainTemplateType::BarnacleEvm => bytes.len() == 20,
                },
                Err(_) => false,
            }
        } else {
            false
        }
    }
    ///
    pub fn assert_valid(&self) {
        assert!(
            self.is_valid(),
            "Invalid validator id in appchain: '{}'",
            &self.origin_to_string()
        );
    }
    ///
    pub fn origin_to_string(&self) -> String {
        match &self.origin {
            Some(id) => id.clone(),
            None => String::new(),
        }
    }
    ///
    pub fn to_string(&self) -> String {
        self.raw_string.clone()
    }
}

/// The state of an appchain
#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainState {
    /// The initial state of an appchain, after it is successfully registered.
    /// This state is managed by appchain registry.
    Registered,
    /// The state while the appchain is under auditing by Octopus Network.
    /// This state is managed by appchain registry.
    Auditing,
    /// The state while voter can upvote or downvote an appchain.
    /// This state is managed by appchain registry.
    InQueue,
    /// The state while validator and delegator can deposit OCT tokens to this contract
    /// to indicate their willing of staking for an appchain.
    Staging,
    /// The state while an appchain is booting.
    Booting,
    /// The state while an appchain is active normally.
    Active,
    /// The state while an appchain is under challenging, which all deposit and withdraw actions
    /// are frozen.
    Frozen,
    /// The state which an appchain is broken for some technical or governance reasons.
    Broken,
    /// The state which the lifecycle of an appchain is end.
    Dead,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainSettings {
    pub rpc_endpoint: String,
    pub subql_endpoint: String,
    pub era_reward: U128,
    pub bonus_for_new_validator: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AnchorSettings {
    pub token_price_maintainer_account: Option<AccountId>,
    pub relayer_account: Option<AccountId>,
    pub beefy_light_client_witness_mode: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProtocolSettings {
    /// A validator has to deposit a certain amount of OCT token to this contract for
    /// being validator of the appchain.
    pub minimum_validator_deposit: U128,
    /// The minimum amount for a validator to increase or decrease his/her deposit.
    pub minimum_validator_deposit_changing_amount: U128,
    /// The maximum percent value that the deposit of a validator in total stake
    pub maximum_validator_stake_percent: u16,
    /// The minimum deposit amount for a delegator to delegate his voting weight to
    /// a certain validator.
    pub minimum_delegator_deposit: U128,
    /// The minimum amount for a delegator to increase or decrease his/her delegation
    /// to a validator.
    pub minimum_delegator_deposit_changing_amount: U128,
    /// The minimum price (in USD) of total stake in this contract for
    /// booting corresponding appchain
    pub minimum_total_stake_price_for_booting: U128,
    /// The maximum percentage of the total market value of all NEP-141 tokens to the total
    /// market value of OCT token staked in this contract
    pub maximum_market_value_percent_of_near_fungible_tokens: u16,
    /// The maximum percentage of the total market value of wrapped appchain token to the total
    /// market value of OCT token staked in this contract
    pub maximum_market_value_percent_of_wrapped_appchain_token: u16,
    /// The minimum number of validator(s) registered in this contract for
    /// booting the corresponding appchain and keep it alive.
    pub minimum_validator_count: U64,
    /// The maximum number of validator(s) registered in this contract for
    /// the corresponding appchain.
    pub maximum_validator_count: U64,
    /// The maximum number of validator(s) which a delegator can delegate to.
    pub maximum_validators_per_delegator: U64,
    /// The unlock period (in days) for validator(s) can withdraw their deposit after
    /// they are removed from the corresponding appchain.
    pub unlock_period_of_validator_deposit: U64,
    /// The unlock period (in days) for delegator(s) can withdraw their deposit after
    /// they no longer delegates their stake to a certain validator on the corresponding appchain.
    pub unlock_period_of_delegator_deposit: U64,
    /// The maximum number of historical eras that the validators or delegators are allowed to
    /// withdraw their reward
    pub maximum_era_count_of_unwithdrawn_reward: U64,
    /// The maximum number of valid appchain message.
    /// If the era number of appchain message is smaller than the latest era number minus
    /// this value, the message will be considered as `invalid`.
    pub maximum_era_count_of_valid_appchain_message: U64,
    /// The percent of commission fees of a validator's reward in an era
    pub validator_commission_percent: u16,
    /// The maximum unprofitable era count for auto-unbonding a validator
    pub maximum_allowed_unprofitable_era_count: u16,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OctToken {
    pub contract_account: AccountId,
    pub price_in_usd: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WrappedAppchainToken {
    pub metadata: FungibleTokenMetadata,
    pub contract_account: Option<AccountId>,
    pub premined_beneficiary: Option<AccountId>,
    pub premined_balance: U128,
    pub changed_balance: I128,
    pub price_in_usd: U128,
    pub total_supply: U128,
}

/// The bridging state of NEP-141 token.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum BridgingState {
    /// The state which this contract is bridging the bridge token to the appchain.
    Active,
    /// The state which this contract has stopped bridging the bridge token to the appchain.
    Closed,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NearFungibleToken {
    pub metadata: FungibleTokenMetadata,
    pub contract_account: AccountId,
    pub price_in_usd: U128,
    /// The total balance locked in this contract
    pub locked_balance: U128,
    pub bridging_state: BridgingState,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum StakingFact {
    /// A new validator is registered in appchain anchor
    ValidatorRegistered {
        validator_id: AccountId,
        validator_id_in_appchain: String,
        amount: U128,
        can_be_delegated_to: bool,
    },
    /// A validator increases his stake in appchain anchor
    StakeIncreased {
        validator_id: AccountId,
        amount: U128,
    },
    /// A validator decreases his stake in appchain anchor
    StakeDecreased {
        validator_id: AccountId,
        amount: U128,
    },
    /// A validator unbonded his stake in appchain anchor
    ValidatorUnbonded {
        validator_id: AccountId,
        amount: U128,
    },
    /// The flag of `can_be_delegated_to` is set to `true`
    ValidatorDelegationEnabled { validator_id: AccountId },
    /// The flag of `can_be_delegated_to` is set to `false`
    ValidatorDelegationDisabled { validator_id: AccountId },
    /// A new delegator is registered in appchain anchor
    DelegatorRegistered {
        delegator_id: AccountId,
        validator_id: AccountId,
        amount: U128,
    },
    /// A delegator increases his delegation for a validator in appchain anchor
    DelegationIncreased {
        delegator_id: AccountId,
        validator_id: AccountId,
        amount: U128,
    },
    /// A delegator decreases his delegation for a validator in appchain anchor
    DelegationDecreased {
        delegator_id: AccountId,
        validator_id: AccountId,
        amount: U128,
    },
    /// A delegator unbonded his delegation for a validator in appchain anchor
    DelegatorUnbonded {
        delegator_id: AccountId,
        validator_id: AccountId,
        amount: U128,
    },
    /// A validator is unbonded by contract automatically
    ValidatorAutoUnbonded {
        validator_id: AccountId,
        amount: U128,
    },
    /// A delegator is unbonded by contract automatically
    DelegatorAutoUnbonded {
        delegator_id: AccountId,
        validator_id: AccountId,
        amount: U128,
    },
    /// A validator's account id in appchain changed
    ValidatorIdInAppchainChanged {
        validator_id: AccountId,
        validator_id_in_appchain: String,
    },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct StakingHistory {
    pub staking_fact: StakingFact,
    pub block_height: U64,
    pub timestamp: U64,
    pub index: U64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum AnchorEvent {
    /// The event that a certain amount of a NEAR fungible token has been locked
    /// in appchain anchor.
    NearFungibleTokenLocked {
        symbol: String,
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: U128,
    },
    /// The event that a certain amount of a NEAR fungible token has been unlocked and
    /// transfered to a receiver in NEAR protocol.
    NearFungibleTokenUnlocked {
        symbol: String,
        sender_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        amount: U128,
        /// The nonce of the appchain message
        appchain_message_nonce: u32,
    },
    /// The event that the action for unlocking a certain amount of a NEAR fungible token
    /// had failed due to some reasons.
    FailedToUnlockNearFungibleToken {
        symbol: String,
        sender_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        amount: U128,
        /// The nonce of the appchain message
        appchain_message_nonce: u32,
        reason: String,
    },
    /// The event that a certain amount of wrapped appchain token is burnt in its contract
    /// in NEAR protocol.
    WrappedAppchainTokenBurnt {
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: U128,
    },
    /// The event that the action for burning a certain amount of wrapped appchain token
    /// had failed due to some reasons.
    FailedToBurnWrappedAppchainToken {
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: U128,
        reason: String,
    },
    /// The event that a certain amount of wrapped appchain token had been minted in NEAR protocol
    WrappedAppchainTokenMinted {
        /// The id of sender on the appchain, or `None` in distributing era rewards case
        sender_id_in_appchain: Option<String>,
        receiver_id_in_near: AccountId,
        amount: U128,
        /// The nonce of the appchain message
        appchain_message_nonce: u32,
    },
    /// The event that the action for minting a certain amount of wrapped appchain token
    /// had failed due to some reasons.
    FailedToMintWrappedAppchainToken {
        /// The id of sender on the appchain, or `None` in distributing era rewards case
        sender_id_in_appchain: Option<String>,
        receiver_id_in_near: AccountId,
        amount: U128,
        /// The nonce of the appchain message
        appchain_message_nonce: u32,
        reason: String,
    },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainValidator {
    pub validator_id: AccountId,
    pub validator_id_in_appchain: String,
    pub deposit_amount: U128,
    pub total_stake: U128,
    pub delegators_count: U64,
    pub can_be_delegated_to: bool,
    pub is_unbonding: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainDelegator {
    pub delegator_id: AccountId,
    pub validator_id: AccountId,
    pub delegation_amount: U128,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct UnbondedStake {
    /// The number of era in appchain.
    pub era_number: U64,
    /// The account id of the owner of unbonded stake.
    pub account_id: AccountId,
    /// The amount of unbonded stake.
    pub amount: U128,
    /// The unlock time of the stake.
    /// If the unlock time is not determined at the time, the value will be `None`.
    pub unlock_time: U64,
}

/// The actual processing order is:
/// `CopyingFromLastEra` -> `UnbondingValidator`-> `AutoUnbondingValidator`
/// -> `ApplyingStakingHistory` -> `ReadyForDistributingReward` -> `DistributingReward`
/// -> `CheckingForAutoUnbondingValidator` -> `Completed`
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum ValidatorSetProcessingStatus {
    CopyingFromLastEra {
        copying_validator_index: U64,
        copying_delegator_index: U64,
    },
    ApplyingStakingHistory {
        applying_index: U64,
    },
    ReadyForDistributingReward,
    DistributingReward {
        appchain_message_nonce: u32,
        distributing_validator_index: U64,
        distributing_delegator_index: U64,
    },
    Completed,
    UnbondingValidator {
        unbonding_validator_index: U64,
        unbonding_delegator_index: U64,
    },
    AutoUnbondingValidator {
        unbonding_validator_index: U64,
        unbonding_delegator_index: U64,
    },
    CheckingForAutoUnbondingValidator {
        unprofitable_validator_index: U64,
    },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PermissionlessActionsStatus {
    /// The era number that is switching by permissionless actions
    pub switching_era_number: Option<U64>,
    /// The era number that is distributing reward by permissionless actions
    pub distributing_reward_era_number: Option<U64>,
    ///
    pub processing_appchain_message_nonce: Option<u32>,
    ///
    pub max_nonce_of_staged_appchain_messages: u32,
    ///
    pub latest_applied_appchain_message_nonce: u32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct IndexRange {
    pub start_index: U64,
    pub end_index: U64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardHistory {
    pub era_number: U64,
    pub total_reward: U128,
    pub unwithdrawn_reward: U128,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AnchorStatus {
    pub total_stake_in_next_era: U128,
    pub validator_count_in_next_era: U64,
    pub delegator_count_in_next_era: U64,
    pub index_range_of_appchain_notification_history: IndexRange,
    pub index_range_of_validator_set_history: IndexRange,
    pub index_range_of_staking_history: IndexRange,
    pub nonce_range_of_appchain_messages: IndexRange,
    pub index_range_of_appchain_challenges: IndexRange,
    pub permissionless_actions_status: PermissionlessActionsStatus,
    pub asset_transfer_is_paused: bool,
    pub rewards_withdrawal_is_paused: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorSetInfo {
    /// The number of era in appchain.
    pub era_number: U64,
    /// Total stake of current set
    pub total_stake: U128,
    /// The validator list for query
    pub validator_list: Vec<AppchainValidator>,
    /// The block height when the era starts.
    pub start_block_height: U64,
    /// The timestamp when the era starts.
    pub start_timestamp: U64,
    /// The index of the latest staking history happened in the era of corresponding appchain.
    pub staking_history_index: U64,
    /// The set of validator id which will not be profited.
    pub unprofitable_validator_ids: Vec<AccountId>,
    /// Total stake excluding all unprofitable validators' stake.
    pub valid_total_stake: U128,
    /// The status of creation of this set
    pub processing_status: ValidatorSetProcessingStatus,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorProfile {
    ///
    pub validator_id: AccountId,
    ///
    pub validator_id_in_appchain: String,
    ///
    pub profile: HashMap<String, String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainNotification {
    /// A certain amount of a NEAR fungible token has been locked in appchain anchor.
    NearFungibleTokenLocked {
        contract_account: AccountId,
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: U128,
    },
    /// A certain amount of wrapped appchain token is burnt in its contract in NEAR protocol.
    WrappedAppchainTokenBurnt {
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: U128,
    },
    /// A certain wrapped non-fungible token is burnt in its contract in NEAR protocol.
    WrappedNonFungibleTokenBurnt {
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        class_id: String,
        instance_id: String,
    },
    /// A certain wrapped appchain NFT is locked in appchain anchor.
    WrappedAppchainNFTLocked {
        class_id: String,
        token_id: String,
        sender_id_in_near: AccountId,
        owner_id_in_near: AccountId,
        receiver_id_in_appchain: String,
    },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainNotificationHistory {
    pub appchain_notification: AppchainNotification,
    pub block_height: U64,
    pub timestamp: U64,
    pub index: U64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainMessageProcessingResult {
    Ok { nonce: u32, message: Option<String> },
    Error { nonce: u32, message: String },
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum MultiTxsOperationProcessingResult {
    NeedMoreGas,
    Ok,
    Error(String),
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorMerkleProof {
    /// Proof items (does not contain the leaf hash, nor the root obviously).
    ///
    /// This vec contains all inner node hashes necessary to reconstruct the root hash given the
    /// leaf hash.
    pub proof: Vec<Hash>,
    /// Number of leaves in the original tree.
    ///
    /// This is needed to detect a case where we have an odd number of leaves that "get promoted"
    /// to upper layers.
    pub number_of_leaves: u32,
    /// Index of the leaf the proof is for (0-based).
    pub leaf_index: u32,
    /// Leaf content.
    pub leaf: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum BeefyLightClientStatus {
    Uninitialized,
    UpdatingState,
    Ready,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainCommitment {
    pub block_number: u32,
    pub validator_set_id: U64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct UserStakingHistory {
    pub staking_fact: StakingFact,
    pub block_height: U64,
    pub timestamp: U64,
    pub has_taken_effect: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum FTDepositMessage {
    RegisterValidator {
        validator_id_in_appchain: String,
        can_be_delegated_to: bool,
        profile: HashMap<String, String>,
    },
    IncreaseStake,
    RegisterDelegator {
        validator_id: AccountId,
    },
    IncreaseDelegation {
        validator_id: AccountId,
    },
    BridgeToAppchain {
        receiver_id_in_appchain: String,
    },
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum NFTTransferMessage {
    BridgeToAppchain { receiver_id_in_appchain: String },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WrappedAppchainNFT {
    pub class_id: String,
    pub metadata: NFTContractMetadata,
    pub contract_account: AccountId,
    pub bridging_state: BridgingState,
    pub count_of_locked_tokens: U64,
}
