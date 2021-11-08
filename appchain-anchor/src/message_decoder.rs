use crate::*;
use codec::{Decode, Encode, Input};

#[derive(Encode, Decode, Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub enum PayloadType {
    Lock,
    BurnAsset,
    PlanNewEra,
    EraPayout,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BurnAssetPayload {
    token_id: String,
    sender: String,
    receiver_id: AccountId,
    amount: u128,
    era: u32,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct LockPayload {
    sender: String,
    receiver_id: AccountId,
    amount: u128,
    era: u32,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PlanNewEraPayload {
    pub next_set_id: u32,
    pub era: u32,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EraPayoutPayload {
    pub current_set_id: u32,
    pub excluded_validators: Vec<String>,
    pub era: u32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AppchainMessage {
    pub appchain_event: AppchainEvent,
    // pub block_height: U64,
    // pub timestamp: U64,
    pub nonce: u32,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum MessagePayload {
    BurnAsset(BurnAssetPayload),
    Lock(LockPayload),
    PlanNewEra(PlanNewEraPayload),
    EraPayout(EraPayoutPayload),
}

pub trait ProofDecoder {
    fn decode(&self, encoded_message: Vec<u8>) -> Vec<AppchainMessage>;
}

#[derive(Encode, Decode, Clone)]
pub struct RawMessage {
    nonce: u64,
    payload_type: PayloadType,
    payload: Vec<u8>,
}

impl ProofDecoder for AppchainAnchor {
    fn decode(&self, encoded_message: Vec<u8>) -> Vec<AppchainMessage> {
        let decoded_messages: Vec<RawMessage> = Decode::decode(&mut &encoded_message[..]).unwrap();

        decoded_messages
            .iter()
            .map(|m| match m.payload_type {
                PayloadType::BurnAsset => {
                    let payload_result: Result<BurnAssetPayload, std::io::Error> =
                        BorshDeserialize::deserialize(&mut &m.payload[..]);
                    let payload = payload_result.unwrap();
                    AppchainMessage {
                        nonce: m.nonce as u32,
                        appchain_event: AppchainEvent::NearFungibleTokenBurnt {
                            symbol: payload.token_id,
                            owner_id_in_appchain: payload.sender,
                            receiver_id_in_near: payload.receiver_id,
                            amount: payload.amount.into(),
                        },
                    }
                }
                PayloadType::Lock => {
                    let payload_result: Result<LockPayload, std::io::Error> =
                        BorshDeserialize::deserialize(&mut &m.payload[..]);
                    let payload = payload_result.unwrap();
                    AppchainMessage {
                        nonce: m.nonce as u32,
                        appchain_event: AppchainEvent::NativeTokenLocked {
                            owner_id_in_appchain: payload.sender,
                            receiver_id_in_near: payload.receiver_id,
                            amount: payload.amount.into(),
                        },
                    }
                }
                PayloadType::PlanNewEra => {
                    let payload_result: Result<PlanNewEraPayload, std::io::Error> =
                        BorshDeserialize::deserialize(&mut &m.payload[..]);
                    let payload = payload_result.unwrap();
                    AppchainMessage {
                        nonce: m.nonce as u32,
                        appchain_event: AppchainEvent::EraSwitchPlaned {
                            era_number: U64::from(payload.era as u64),
                            next_set_id: payload.next_set_id,
                        },
                    }
                }
                PayloadType::EraPayout => {
                    let payload_result: Result<EraPayoutPayload, std::io::Error> =
                        BorshDeserialize::deserialize(&mut &m.payload[..]);
                    let payload = payload_result.unwrap();
                    AppchainMessage {
                        nonce: m.nonce as u32,
                        appchain_event: AppchainEvent::EraRewardConcluded {
                            era_number: U64::from(payload.era as u64),
                            unprofitable_validator_ids: payload.excluded_validators,
                            current_set_id: payload.current_set_id,
                        },
                    }
                }
            })
            .collect()
    }
}
