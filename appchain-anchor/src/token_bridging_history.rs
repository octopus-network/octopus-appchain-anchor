use near_sdk::BlockHeight;

use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum TokenBridgingFact {
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
    NearFungibleTokenLocked {
        symbol: String,
        /// The account id of sender in NEAR protocol
        sender_id: AccountId,
        /// The id of receiver on the appchain
        receiver_id: String,
        amount: U128,
    },
    /// The fact that a certain amount of NEP-141 token has been unlocked and
    /// transfered from this contract to the receiver.
    NearFungibleTokenUnlocked {
        request_id: String,
        symbol: String,
        /// The account id of receiver in NEAR protocol
        receiver_id: AccountId,
        amount: U128,
    },
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenBridgingHistory {
    pub token_bridging_fact: TokenBridgingFact,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub index: U64,
}

pub trait TokenBridgingHistoryManager {
    ///
    fn clear_token_bridging_history_before(&mut self, timestamp: Timestamp);
}
