use std::{collections::HashMap, fmt::Display};

use near_sdk::{
    json_types::{I128, U64},
    BlockHeight,
};

use crate::*;

pub type AppchainId = String;

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
    /// The state which an appchain is broken for some technical or governance reasons.
    Broken,
    /// The state which the lifecycle of an appchain is end.
    Dead,
}

/// The fact that happens in the appchain anchor contract
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum AnchorFact {
    /// The fact that a certain amount of appchain native token is minted in its contract
    /// in NEAR protocol
    AppchainNativeTokenMinted {
        request_id: String,
        /// The account id of receiver in NEAR protocol
        receiver_id: AccountId,
        amount: U128,
    },
    /// The fact that a certain amount of appchain native token is burnt in its contract
    /// in NEAR protocol
    AppchainNativeTokenBurnt {
        sender_id: AccountId,
        /// The id of receiver on the appchain
        receiver_id: String,
        amount: U128,
    },
    /// The fact that a certain amount of bridge token has been locked in appchain anchor.
    BridgeTokenLocked {
        symbol: String,
        /// The account id of sender in NEAR protocol
        sender_id: AccountId,
        /// The id of receiver on the appchain
        receiver_id: String,
        amount: U128,
    },
    /// The fact that a certain amount of bridge token has been unlocked and
    /// transfered from this contract to the receiver.
    BridgeTokenUnlocked {
        request_id: String,
        symbol: String,
        /// The account id of receiver in NEAR protocol
        receiver_id: AccountId,
        amount: U128,
    },
    /// A new validator is registered in appchain anchor
    ValidatorAdded {
        validator_id: AccountId,
        amount: U128,
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
    /// A new delegator is registered in appchain anchor
    DelegatorAdded {
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
}

pub struct AnchorFactRecord {
    pub anchor_fact: AnchorFact,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
}

/// The fact that happens on the appchain
pub enum AppchainFact {
    /// The fact that a certain amount of bridge token has been burnt on the appchain.
    BridgeTokenBurnt { symbol: String, amount: U128 },
    /// The fact that a certain amount of appchain native token has been locked on the appchain.
    NativeTokenLocked { amount: U128 },
    /// The fact that a validator has been unbonded on the appchain.
    ValidatorUnbonded {
        validator_id: AccountId,
        set_id: u32,
    },
    /// The fact that a delegator has been unbonded on the appchain.
    DelegatorUnbonded {
        delegator_id: AccountId,
        validator_id: AccountId,
        set_id: u32,
    },
}

pub struct AppchainFactRecord {
    pub appchain_fact: AppchainFact,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
}

/// The staking state of a validator or delegator of the appchain.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum StakingState {
    /// Active in staking on corresponding appchain.
    Active,
    /// Has been unbonded from staking on corresponding appchain.
    Unbonded,
}

/// Appchain validator of an appchain.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainValidator {
    /// Account id of appchain validator.
    pub validator_id: AccountId,
    /// Staked balance of the validator.
    pub deposit_amount: Balance,
    /// Staking state of the validator.
    pub staking_state: StakingState,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainDelegator {
    /// Account id of appchain delegator.
    pub delegator_id: AccountId,
    /// Account id of a validator which the delegator delegates his rights to.
    pub validator_id: AccountId,
    /// Delegated balance of the delegator.
    pub deposit_amount: Balance,
    /// Staking state of the delegator.
    pub staking_state: StakingState,
}

/// The bridging state of bridge token.
pub enum BridgingState {
    /// The state which this contract is bridging the bridge token to the appchain.
    Active,
    /// The state which this contract has stopped bridging the bridge token to the appchain.
    Closed,
}

pub struct BridgeTokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
}

pub struct BridgeToken {
    pub metadata: BridgeTokenMetadata,
    pub contract_account: AccountId,
    pub price: U64,
    pub price_decimals: u8,
    /// The total balance locked in this contract
    pub locked_balance: Balance,
    pub bridging_state: BridgingState,
}

pub struct AppchainNativeTokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub spec: String,
    pub icon: Option<Vec<u8>>,
    pub reference: Option<Vec<u8>>,
    pub reference_hash: Option<Vec<u8>>,
}

pub struct AppchainNativeToken {
    pub metadata: AppchainNativeTokenMetadata,
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
    /// The time duration for updating validator set based on recent deposit actions
    /// happened in this contract.
    pub validator_set_duration: U64,
    /// The unlock period for validator(s) can withdraw their deposit after they are removed from
    /// the corresponding appchain.
    pub unlock_period_of_validator_deposit: U64,
    /// The unlock period for delegator(s) can withdraw their deposit after they no longer
    /// delegates their stake to a certain validator on the corresponding appchain.
    pub unlock_period_of_delegator_deposit: U64,
}
