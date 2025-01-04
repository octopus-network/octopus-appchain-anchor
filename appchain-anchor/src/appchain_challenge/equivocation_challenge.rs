use crate::*;

pub type RoundNumber = u32;
pub type SetId = u32;
pub type BlockNumber = u32;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Hash(pub [u8; 32]);

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PublicKey(pub [u8; 32]);

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct SignatureData(pub Vec<u8>);

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct GrandpaEquivocation<V, S> {
    pub round_number: RoundNumber,
    pub identity: PublicKey,
    pub first: (V, S),
    pub second: (V, S),
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Equivocation {
    Prevote(GrandpaEquivocation<GrandpaPrevote, SignatureData>),
    Precommit(GrandpaEquivocation<GrandpaPrecommit, SignatureData>),
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct EquivocationProof {
    pub set_id: SetId,
    pub equivocation: Equivocation,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct GrandpaPrevote {
    pub target_hash: Hash,
    pub target_number: BlockNumber,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct GrandpaPrecommit {
    pub target_hash: Hash,
    pub target_number: BlockNumber,
}
