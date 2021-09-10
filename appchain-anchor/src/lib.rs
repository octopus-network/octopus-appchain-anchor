pub mod types;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, LookupSet, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{U128, U64, WrappedDuration};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    assert_self, env, ext_contract, log, near_bindgen, AccountId, Balance, Duration, Promise,
    PromiseOrValue, PromiseResult, PublicKey, Timestamp,
};
use types::*;

/// Multiple of nano seconds for a second
const NANO_SECONDS_MULTIPLE: u64 = 1_000_000_000;

pub struct AppchainAnchor {
    /// The id of corresponding appchain.
    pub appchain_id: AppchainId,
    /// The account id of appchain registry contract.
    pub appchain_registry_contract: AccountId,
    /// The account id of OCT token contract.
    pub oct_token_contract: AccountId,
    /// The wrapped appchain token in NEAR protocol.
    pub wrapped_appchain_token: WrappedAppchainToken,
    /// The set of symbols of NEP-141 tokens.
    pub nep141_token_symbols: UnorderedSet<String>,
    /// The NEP-141 tokens data, mapped by the symbol of the token.
    pub nep141_tokens: LookupMap<String, Nep141Token>,
    /// The currently used validator set in appchain
    pub current_validator_set: TaggedAppchainValidatorSet,
    /// The validator set for unbonded validators and delegators
    pub unbonded_validator_set: AppchainValidatorSet,
    /// The mapping for validators' accounts, from account id in the appchain to
    /// account id in NEAR protocol
    pub validator_account_id_mapping: LookupMap<AccountIdInAppchain, AccountId>,
    /// The protocol settings for appchain anchor
    pub protocol_settings: LazyOption<ProtocolSettings>,
    /// The state of the corresponding appchain
    pub appchain_state: AppchainState,
    /// The current total stake of all validators and delegators in this contract.
    pub total_stake: Balance,
    /// The staking history data happened in this contract
    pub staking_histories: LookupMap<u64, StakingHistoryRecord>,
    /// The start index of valid staking history in `staking_histories`.
    pub staking_history_start_index: u64,
    /// The end index of valid staking history in `staking_histories`.
    pub staking_history_end_index: u64,
    /// The token bridging history data happened in this contract
    pub token_bridging_histories: LookupMap<u64, TokenBridgingHistoryRecord>,
    /// The start index of valid token bridging history in `token_bridging_histories`.
    pub token_bridging_history_start_index: u64,
    /// The end index of valid token bridging history in `token_bridging_histories`.
    pub token_bridging_history_end_index: u64,
}
