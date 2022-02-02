use crate::*;
use codec::{Decode, Encode};
use core::convert::TryFrom;
use ed25519_dalek::Verifier;

pub type RoundNumber = u64;
pub type SetId = u64;
pub type BlockNumber = u32;

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug, Decode, Encode,
)]
#[serde(crate = "near_sdk::serde")]
pub struct Hash(pub [u8; 32]);

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug, Decode, Encode,
)]
#[serde(crate = "near_sdk::serde")]
pub struct PublicKey(pub [u8; 32]);

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug, Decode, Encode,
)]
#[serde(crate = "near_sdk::serde")]
pub struct SignatureData(pub Vec<u8>);

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug, Decode, Encode,
)]
#[serde(crate = "near_sdk::serde")]
pub struct VoteData {
    pub target_hash: Hash,
    pub target_number: BlockNumber,
}

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug, Decode, Encode,
)]
#[serde(crate = "near_sdk::serde")]
pub struct GrandpaEquivocation {
    pub round_number: RoundNumber,
    pub identity: PublicKey,
    pub first: (VoteData, SignatureData),
    pub second: (VoteData, SignatureData),
}

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug, Decode, Encode,
)]
#[serde(crate = "near_sdk::serde")]
pub enum Equivocation {
    Prevote(GrandpaEquivocation),
    Precommit(GrandpaEquivocation),
}

#[derive(
    BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug, Decode, Encode,
)]
#[serde(crate = "near_sdk::serde")]
pub struct EquivocationProof {
    pub set_id: SetId,
    pub equivocation: Equivocation,
}

impl EquivocationProof {
    ///
    pub fn is_valid(&self) -> bool {
        match &self.equivocation {
            Equivocation::Prevote(equivocation) | Equivocation::Precommit(equivocation) => {
                self.check_signature(
                    &equivocation.first.0,
                    &equivocation.round_number,
                    &equivocation.first.1,
                    &equivocation.identity,
                ) && self.check_signature(
                    &equivocation.second.0,
                    &equivocation.round_number,
                    &equivocation.second.1,
                    &equivocation.identity,
                )
            }
        }
    }
    //
    fn check_signature(
        &self,
        message: &VoteData,
        round: &RoundNumber,
        signature: &SignatureData,
        pubkey: &PublicKey,
    ) -> bool {
        let mut buffer = Vec::<u8>::new();
        (message, round, self.set_id).encode_to(&mut buffer);
        if signature.0.len() != 64 {
            return false;
        }
        let mut sig_data: [u8; 64] = [0; 64];
        for i in 0..64 {
            sig_data[i] = signature.0.get(i).unwrap_or(&0).clone();
        }
        if let Ok(signature) = ed25519_dalek::Signature::try_from(sig_data) {
            match ed25519_dalek::PublicKey::from_bytes(&pubkey.0) {
                Ok(pubkey) => pubkey.verify(&buffer, &signature).is_ok(),
                Err(..) => false,
            }
        } else {
            false
        }
    }
}
