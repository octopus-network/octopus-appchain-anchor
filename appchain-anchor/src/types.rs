use near_sdk::{json_types::U64, BlockHeight};

use crate::*;

pub type AppchainId = String;
pub type AccountIdInAppchain = String;

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
pub enum TokenBridgingHistory {
    /// The fact that a certain amount of wrapped appchain token is minted in its contract
    /// in NEAR protocol
    WrappedAppchainTokenMinted {
        request_id: String,
        /// The account id of receiver in NEAR protocol
        receiver_id: AccountId,
        amount: U128,
    },
    /// The fact that a certain amount of wrapped appchain token is burnt in its contract
    /// in NEAR protocol
    WrappedAppchainTokenBurnt {
        sender_id: AccountId,
        /// The id of receiver on the appchain
        receiver_id: String,
        amount: U128,
    },
    /// The fact that a certain amount of NEP-141 token has been locked in appchain anchor.
    Nep141TokenLocked {
        symbol: String,
        /// The account id of sender in NEAR protocol
        sender_id: AccountId,
        /// The id of receiver on the appchain
        receiver_id: String,
        amount: U128,
    },
    /// The fact that a certain amount of NEP-141 token has been unlocked and
    /// transfered from this contract to the receiver.
    Nep141TokenUnlocked {
        request_id: String,
        symbol: String,
        /// The account id of receiver in NEAR protocol
        receiver_id: AccountId,
        amount: U128,
    },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenBridgingHistoryRecord {
    pub token_bridging_history: TokenBridgingHistory,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub index: U64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum StakingHistory {
    /// A new validator is registered in appchain anchor
    ValidatorAdded {
        /// The validator's id in NEAR protocol.
        validator_id_in_near: AccountId,
        /// The validator's id in the appchain.
        validator_id_in_appchain: AccountIdInAppchain,
        amount: U128,
    },
    /// A validator increases his stake in appchain anchor
    StakeIncreased {
        /// The validator's id in NEAR protocol.
        validator_id_in_near: AccountId,
        amount: U128,
    },
    /// A validator decreases his stake in appchain anchor
    StakeDecreased {
        /// The validator's id in NEAR protocol.
        validator_id_in_near: AccountId,
        amount: U128,
    },
    /// A new delegator is registered in appchain anchor
    DelegatorAdded {
        /// The delegator's id in NEAR protocol.
        delegator_id_in_near: AccountId,
        /// The delegator's id in the appchain.
        delegator_id_in_appchain: AccountIdInAppchain,
        /// The validator's id in NEAR protocol.
        validator_id_in_near: AccountId,
        amount: U128,
    },
    /// A delegator increases his delegation for a validator in appchain anchor
    DelegationIncreased {
        /// The delegator's id in NEAR protocol.
        delegator_id_in_near: AccountId,
        /// The validator's id in NEAR protocol.
        validator_id_in_near: AccountId,
        amount: U128,
    },
    /// A delegator decreases his delegation for a validator in appchain anchor
    DelegationDecreased {
        /// The delegator's id in NEAR protocol.
        delegator_id_in_near: AccountId,
        /// The validator's id in NEAR protocol.
        validator_id_in_near: AccountId,
        amount: U128,
    },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct StakingHistoryRecord {
    pub staking_history: StakingHistory,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub index: U64,
}

/// The message which is sent from the appchain
pub enum AppchainMessage {
    /// The fact that a certain amount of bridge token has been burnt on the appchain.
    Nep141TokenBurnt { symbol: String, amount: U128 },
    /// The fact that a certain amount of appchain native token has been locked on the appchain.
    NativeTokenLocked { amount: U128 },
    /// The fact that a validator has been unbonded on the appchain.
    ValidatorUnbonded {
        validator_id: AccountIdInAppchain,
        block_height: BlockHeight,
        timestamp: Timestamp,
    },
    /// The fact that a delegator has been unbonded on the appchain.
    DelegatorUnbonded {
        delegator_id: AccountIdInAppchain,
        validator_id: AccountIdInAppchain,
        block_height: BlockHeight,
        timestamp: Timestamp,
    },
}

pub struct AppchainMessageRecord {
    pub appchain_fact: AppchainMessage,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub nonce: u32,
}

/// The staking state of a validator or delegator of the appchain.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum StakingState {
    /// Active in staking on corresponding appchain.
    Active {
        block_height: BlockHeight,
        timestamp: Timestamp,
    },
    /// Has been unbonded from staking on corresponding appchain.
    Unbonded {
        block_height: BlockHeight,
        timestamp: Timestamp,
    },
}

/// Appchain validator of an appchain.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainValidator {
    /// The validator's id in NEAR protocol.
    pub validator_id_in_near: AccountId,
    /// The validator's id in the appchain.
    pub validator_id_in_appchain: AccountIdInAppchain,
    /// The account id in the appchain for receiving income of the validator in appchain.
    pub payee_id_in_appchain: AccountIdInAppchain,
    /// Staked balance of the validator.
    pub deposit_amount: Balance,
    /// Staking state of the validator.
    pub staking_state: StakingState,
    /// The reserved validator can NOT be delegated to.
    pub is_reserved: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainDelegator {
    /// The delegator's id in NEAR protocol.
    pub delegator_id_in_near: AccountId,
    /// The delegator's id in the appchain.
    pub delegator_id_in_appchain: AccountIdInAppchain,
    /// The validator's id in NEAR protocol, which the delegator delegates his rights to.
    pub validator_id_in_near: AccountId,
    /// The validator's id in the appchain, which the delegator delegates his rights to.
    pub validator_id_in_appchain: AccountIdInAppchain,
    /// Delegated balance of the delegator.
    pub deposit_amount: Balance,
    /// Staking state of the delegator.
    pub staking_state: StakingState,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AppchainValidatorSet {
    /// The set of account id of validators.
    pub validator_ids: LazyOption<UnorderedSet<AccountId>>,
    /// The set of account id of delegators.
    pub delegator_ids: LazyOption<UnorderedSet<AccountId>>,
    /// The validators that a delegator delegates his/her voting rights to.
    pub validator_ids_of_delegator_id: LookupMap<AccountId, UnorderedSet<AccountId>>,
    /// The validators data, mapped by their account id in NEAR protocol.
    pub validators: LookupMap<AccountId, AppchainValidator>,
    /// The delegators data, mapped by the tuple of their delegator account id and
    /// validator account id in NEAR protocol.
    pub delegators: LookupMap<(AccountId, AccountId), AppchainDelegator>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct TaggedAppchainValidatorSet {
    /// The number of era in appchain.
    pub appchain_era_number: u64,
    /// The index of the latest staking history happened in the era of corresponding appchain
    pub staking_history_index: u64,
    /// The index of latest applied staking history
    pub applied_staking_history_index: u64,
    /// The validator set for tagging
    pub validator_set: AppchainValidatorSet,
}

/// The bridging state of bridge token.
pub enum BridgingState {
    /// The state which this contract is bridging the bridge token to the appchain.
    Active,
    /// The state which this contract has stopped bridging the bridge token to the appchain.
    Closed,
}

pub struct Nep141TokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
}

pub struct Nep141Token {
    pub metadata: Nep141TokenMetadata,
    pub contract_account: AccountId,
    pub price: U64,
    pub price_decimals: u8,
    /// The total balance locked in this contract
    pub locked_balance: Balance,
    pub bridging_state: BridgingState,
}

pub struct WrappedAppchainTokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub spec: String,
    pub icon: Option<Vec<u8>>,
    pub reference: Option<Vec<u8>>,
    pub reference_hash: Option<Vec<u8>>,
}

pub struct WrappedAppchainToken {
    pub metadata: WrappedAppchainTokenMetadata,
    pub contract_account: AccountId,
    pub price: U64,
    pub price_decimals: u8,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct ProtocolSettings {
    /// A validator has to deposit a certain amount of OCT token to this contract for
    /// being validator of the appchain.
    pub minimum_validator_deposit: Balance,
    /// The minimum deposit amount for a delegator to delegate his voting weight to
    /// a certain validator.
    pub minimum_delegator_deposit: Balance,
    /// The minimum number of validator(s) registered in this contract for
    /// booting the corresponding appchain and keep it alive.
    pub minimum_validator_count: U64,
    /// The maximum number of validator(s) which a delegator can delegate to.
    pub maximum_validators_per_delegator: U64,
    /// The unlock period (in days) for validator(s) can withdraw their deposit after
    /// they are removed from the corresponding appchain.
    pub unlock_period_of_validator_deposit: u16,
    /// The unlock period (in days) for delegator(s) can withdraw their deposit after
    /// they no longer delegates their stake to a certain validator on the corresponding appchain.
    pub unlock_period_of_delegator_deposit: u16,
}
