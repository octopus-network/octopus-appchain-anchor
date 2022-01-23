use crate::*;
use codec::{Decode, Encode};

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
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct LockPayload {
    sender: String,
    receiver_id: AccountId,
    amount: u128,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PlanNewEraPayload {
    pub new_era: u32,
}

#[derive(Clone, Serialize, Deserialize, BorshDeserialize, BorshSerialize)]
#[serde(crate = "near_sdk::serde")]
pub struct EraPayoutPayload {
    pub end_era: u32,
    pub excluded_validators: Vec<String>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
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

#[derive(Encode, Decode, Clone)]
pub struct RawMessage {
    nonce: u64,
    payload_type: PayloadType,
    payload: Vec<u8>,
}

pub fn decode(encoded_message: Vec<u8>) -> Vec<AppchainMessage> {
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
                        contract_account: payload.token_id,
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
                        era_number: payload.new_era,
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
                        era_number: payload.end_era,
                        unprofitable_validator_ids: payload.excluded_validators,
                    },
                }
            }
        })
        .collect()
}
