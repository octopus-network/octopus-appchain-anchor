use crate::*;
use codec::{Decode, Encode};
use core::convert::TryFrom;
use ed25519_dalek::Verifier;

pub type RoundNumber = u32;
pub type SetId = u32;
pub type BlockNumber = u32;

#[derive(
    BorshDeserialize,
    BorshSerialize,
    Deserialize,
    Serialize,
    Clone,
    Debug,
    Decode,
    Encode,
    PartialEq,
)]
#[serde(crate = "near_sdk::serde")]
pub struct Hash(pub [u8; 32]);

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PublicKey(pub [u8; 32]);

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
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

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct GrandpaEquivocation {
    pub round_number: RoundNumber,
    pub identity: PublicKey,
    pub first: (VoteData, SignatureData),
    pub second: (VoteData, SignatureData),
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Equivocation {
    Prevote(GrandpaEquivocation),
    Precommit(GrandpaEquivocation),
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, Clone, Debug)]
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
                // if both votes have the same target the equivocation is invalid.
                if equivocation.first.0.target_hash == equivocation.second.0.target_hash
                    && equivocation.first.0.target_number == equivocation.second.0.target_number
                {
                    env::log(b"Votes in equivocation have same targets.");
                    return false;
                }
                // check all signatures are valid
                let valid_first = self.check_signature(
                    &equivocation.first.0,
                    &equivocation.round_number,
                    &equivocation.first.1,
                    &equivocation.identity,
                );
                let valid_second = self.check_signature(
                    &equivocation.second.0,
                    &equivocation.round_number,
                    &equivocation.second.1,
                    &equivocation.identity,
                );
                valid_first && valid_second
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
        // Notice:
        // Need to convert `round` and `set_id` to u64 to match original
        // signing data in appchain side
        (message, u64::from(*round), u64::from(self.set_id)).encode_to(&mut buffer);
        if signature.0.len() != 64 {
            env::log(b"Invalid signature data length.");
            return false;
        }
        let mut sig_data: [u8; 64] = [0; 64];
        for i in 0..64 {
            sig_data[i] = signature.0.get(i).unwrap_or(&0).clone();
        }
        if let Ok(signature) = ed25519_dalek::Signature::try_from(sig_data) {
            match ed25519_dalek::PublicKey::from_bytes(&pubkey.0) {
                Ok(pubkey) => match pubkey.verify(&buffer, &signature) {
                    Ok(()) => true,
                    Err(err) => {
                        env::log(format!("Signature verification failed: {}", err).as_bytes());
                        false
                    }
                },
                Err(err) => {
                    env::log(format!("Invalid ed25519 pubkey: {}", err).as_bytes());
                    false
                }
            }
        } else {
            env::log(b"Invalid ed25519 signature data.");
            false
        }
    }
}
