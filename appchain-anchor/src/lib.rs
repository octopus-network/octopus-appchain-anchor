pub mod types;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::json_types::{U128, U64, WrappedDuration};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_self, env, ext_contract, log, near_bindgen, AccountId, Balance, Duration, Promise,
    PromiseOrValue, PromiseResult, PublicKey, Timestamp,
};
use types::*;

pub struct AppchainAnchor {
    /// The id of corresponding appchain.
    pub appchain_id: AppchainId,
    /// The account id of appchain registry contract.
    pub appchain_registry_contract: AccountId,
    /// The account id of OCT token contract.
    pub oct_token_contract: AccountId,
    /// The native token of appchain.
    pub appchain_native_token: AppchainNativeToken,
    /// The bridge tokens data, mapped by the symbol of the token.
    pub bridge_tokens: UnorderedMap<String, BridgeToken>,
    /// The protocol settings for appchain anchor
    pub protocol_settings: ProtocolSettings,
    /// The state of the corresponding appchain
    pub appchain_state: AppchainState,
    /// The start index of anchor facts stored in the storage of this contract.
    pub anchor_fact_start_index: U64,
    /// The end index of anchor facts stored in the storage of this contract.
    pub anchor_fact_end_index: U64,
    /// The start index of appchain facts stored in the storage of this contract.
    pub appchain_fact_start_index: U64,
    /// The end index of appchain facts stored in the storage of this contract.
    pub appchain_fact_end_index: U64,
}
