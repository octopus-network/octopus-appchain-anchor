use near_sdk::json_types::I128;

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
pub struct AppchainSettings {
    pub chain_spec: String,
    pub raw_chain_spec: String,
    pub boot_node: String,
    pub rpc_endpoint: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AnchorSettings {
    pub token_price_maintainer_account: AccountId,
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
    /// The minimum value of total stake in this contract for booting corresponding appchain
    pub minimum_total_stake_for_booting: Balance,
    /// The maximum percentage of the total market value of all NEP-141 tokens to the total
    /// market value of OCT token staked in this contract
    pub maximum_market_value_percent_of_near_fungible_tokens: u16,
    /// The maximum percentage of the total market value of wrapped appchain token to the total
    /// market value of OCT token staked in this contract
    pub maximum_market_value_percent_of_wrapped_appchain_token: u16,
    /// The minimum number of validator(s) registered in this contract for
    /// booting the corresponding appchain and keep it alive.
    pub minimum_validator_count: u16,
    /// The maximum number of validator(s) which a delegator can delegate to.
    pub maximum_validators_per_delegator: u16,
    /// The unlock period (in days) for validator(s) can withdraw their deposit after
    /// they are removed from the corresponding appchain.
    pub unlock_period_of_validator_deposit: u16,
    /// The unlock period (in days) for delegator(s) can withdraw their deposit after
    /// they no longer delegates their stake to a certain validator on the corresponding appchain.
    pub unlock_period_of_delegator_deposit: u16,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct OctToken {
    pub contract_account: AccountId,
    pub price_in_usd: U64,
    pub total_stake: Balance,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WrappedAppchainTokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub spec: String,
    pub icon: Option<Vec<u8>>,
    pub reference: Option<Vec<u8>>,
    pub reference_hash: Option<Vec<u8>>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct WrappedAppchainToken {
    pub metadata: WrappedAppchainTokenMetadata,
    pub contract_account: AccountId,
    pub initial_balance: Balance,
    pub changed_balance: I128,
    pub price_in_usd: U64,
}

/// The bridging state of NEP-141 token.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum BridgingState {
    /// The state which this contract is bridging the bridge token to the appchain.
    Active,
    /// The state which this contract has stopped bridging the bridge token to the appchain.
    Closed,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NearFungibleTokenMetadata {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NearFungibleToken {
    pub metadata: NearFungibleTokenMetadata,
    pub contract_account: AccountId,
    pub price_in_usd: U64,
    /// The total balance locked in this contract
    pub locked_balance: Balance,
    pub bridging_state: BridgingState,
}
