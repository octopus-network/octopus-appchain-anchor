use near_sdk::BlockHeight;

use crate::*;

/// The message which is sent from the appchain
pub enum AppchainFact {
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
    /// The fact that the era is switched in the appchain
    EraSwitched { appchain_era_number: U64 },
}

pub struct AppchainMessage {
    pub appchain_fact: AppchainFact,
    pub block_height: BlockHeight,
    pub timestamp: Timestamp,
    pub nonce: u32,
}

pub trait AppchainMessageHandler {
    ///
    fn handle_appchain_message(
        &mut self,
        encoded_messages: Vec<u8>,
        header_partial: Vec<u8>,
        leaf_proof: Vec<u8>,
        mmr_root: Vec<u8>,
    );
}
