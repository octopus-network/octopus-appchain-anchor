#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
use core::result;

use beefy_merkle_tree::{merkle_root, verify_proof, Hash, Keccak256, MerkleProof};
use borsh::{BorshDeserialize, BorshSerialize};
use commitment::{Commitment, SignedCommitment};
use header::Header;
use mmr::MmrLeaf;
use validator_set::{BeefyNextAuthoritySet, Public, ValidatorSetId};

pub mod commitment;
pub mod header;
pub mod mmr;
pub mod simplified_mmr;
pub mod validator_set;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// [Commitment] can't be imported, cause it's signed by either past or future validator set.
    InvalidValidatorSetId {
        expected: ValidatorSetId,
        got: ValidatorSetId,
    },
    /// [Commitment] can't be imported, cause it's a set transition block and the proof is missing.
    InvalidValidatorProof,
    /// There are too many signatures in the commitment - more than validators.
    InvalidNumberOfSignatures {
        /// Number of validators in the set.
        expected: usize,
        /// Numbers of signatures in the commitment.
        got: usize,
    },
    /// [SignedCommitment] doesn't have enough valid signatures.
    NotEnoughValidSignatures {
        expected: usize,
        got: usize,
        valid: Option<usize>,
    },
    /// Next validator set has not been provided by any of the previous commitments.
    MissingNextValidatorSetData,
    /// Couldn't verify the proof against MMR root of the latest commitment.
    InvalidMmrProof,
    ///
    InvalidSignature,
    ///
    InvalidMessage,
    ///
    InvalidRecoveryId,
    ///
    WrongSignature,
    ///
    InvalidMmrLeafProof,
    ///
    DigestNotFound,
    ///
    DigestNotMatch,
    ///
    HeaderHashNotMatch,
}

#[derive(Debug, Default, BorshDeserialize, BorshSerialize)]
pub struct LightClient {
    mmr_root: Hash,
    validator_set: BeefyNextAuthoritySet,
}

// Initialize light client using the BeefyId of the initial validator set.
pub fn new(initial_public_keys: Vec<Public>) -> LightClient {
    LightClient {
        mmr_root: Hash::default(),
        validator_set: BeefyNextAuthoritySet {
            id: 0,
            len: initial_public_keys.len() as u32,
            root: merkle_root::<Keccak256, _, _>(initial_public_keys),
        },
    }
}

impl LightClient {
    // Import a signed commitment and update the state of light client.
    pub fn update_state(
        &mut self,
        signed_commitment: SignedCommitment,
        validator_proof: Vec<MerkleProof<&Public>>,
        mmr_leaf: MmrLeaf,
        mmr_proof: simplified_mmr::MerkleProof,
    ) -> Result<(), Error> {
        // TODO: check length
        for proof in validator_proof {
            if !verify_proof::<Keccak256, _, _>(
                &self.validator_set.root,
                proof.proof,
                proof.number_of_leaves,
                proof.leaf_index,
                proof.leaf,
            ) {
                return Err(Error::InvalidValidatorProof);
            }
        }
        let commitment = self.verify_commitment(signed_commitment)?;
        self.verify_mmr_leaf(commitment.payload, &mmr_leaf, mmr_proof)?;

        // update mmr_root
        self.mmr_root = commitment.payload;

        // update validator_set
        if mmr_leaf.beefy_next_authority_set.id > self.validator_set.id {
            self.validator_set = mmr_leaf.beefy_next_authority_set;
        }
        Ok(())
    }

    pub fn verify_solochain_messages(
        &self,
        messages: &[u8],
        header: Header,
        leaf: &MmrLeaf,
        proof: simplified_mmr::MerkleProof,
    ) -> Result<(), Error> {
        let header_digest = header.get_other().ok_or(Error::DigestNotFound)?;

        let messages_hash = Keccak256::hash(messages);
        if messages_hash != &header_digest[..] {
            return Err(Error::DigestNotFound);
        }

        let header_hash = header.hash();
        if header_hash != leaf.parent_number_and_hash.1 {
            return Err(Error::HeaderHashNotMatch);
        }

        self.verify_mmr_leaf(self.mmr_root, leaf, proof)?;
        Ok(())
    }

    pub fn verify_parachain_messages(&self) -> Result<(), Error> {
        Ok(())
    }

    fn verify_commitment(&self, signed_commitment: SignedCommitment) -> Result<Commitment, Error> {
        let SignedCommitment {
            commitment,
            signatures,
        } = signed_commitment;
        let commitment_hash = commitment.hash();
        println!("commitment_hash: {:?}", commitment_hash);
        let msg = libsecp256k1::Message::parse_slice(&commitment_hash[..])
            .or(Err(Error::InvalidMessage))?;
        for signature in signatures.into_iter() {
            if let Some(signature) = signature {
                let sig = libsecp256k1::Signature::parse_standard_slice(&signature.0[..64])
                    .or(Err(Error::InvalidSignature))?;
                let recovery_id = libsecp256k1::RecoveryId::parse(signature.0[64])
                    .or(Err(Error::InvalidRecoveryId))?;
                libsecp256k1::recover(&msg, &sig, &recovery_id).or(Err(Error::WrongSignature))?;
            }
        }

        Ok(commitment)
    }

    fn verify_mmr_leaf(
        &self,
        root: Hash,
        leaf: &MmrLeaf,
        proof: simplified_mmr::MerkleProof,
    ) -> Result<(), Error> {
        let leaf_hash = leaf.hash();
        let result = simplified_mmr::verify_proof(root, leaf_hash, proof);
        if !result {
            return Err(Error::InvalidMmrProof);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commitment::{Commitment, Signature, SignedCommitment};
    use hex_literal::hex;

    #[test]
    fn it_works() {
        let public_keys =
            vec![hex!("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1").into()];
        let lc = new(public_keys);
        println!("{:?}", lc);

        let commitment = Commitment {
            payload: hex!("700a2fb21ba1ec2cdf72bb621846a4cc8628ed8e3ed5bb299f9e36406776f84a")
                .into(),
            block_number: 1369,
            validator_set_id: 0,
        };
        let signed_commitment = SignedCommitment { commitment, signatures: vec![Some(Signature(hex!("3a481c251a7aa94b89e8160aa9073f74cc24570da13ec9f697a9a7c989943bed31b969b50c47675c11994fbdacb82707293976927922ec8c2124490e417af73300").into()))] };
        let res = lc.verify_commitment(signed_commitment).unwrap();
        println!("{:?}", res);

        assert_eq!(2 + 2, 4);
        // let pk = hex!("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1");
        // let pk = libsecp256k1::PublicKey::parse_slice(&pk[..], None).unwrap();
    }

    #[test]
    fn recover_works() {
        let msg = libsecp256k1::Message::parse_slice(&hex!(
            "14f213146a362c397545659ac7795926514696ad49565972d64964040394482c"
        ))
        .unwrap();
        let signature =  Signature(hex!("3a481c251a7aa94b89e8160aa9073f74cc24570da13ec9f697a9a7c989943bed31b969b50c47675c11994fbdacb82707293976927922ec8c2124490e417af73300").into());
        let sig = libsecp256k1::Signature::parse_standard_slice(&signature.0[..64]).unwrap();
        let public_key = libsecp256k1::recover(
            &msg,
            &sig,
            &libsecp256k1::RecoveryId::parse(signature.0[64]).unwrap(),
        )
        .unwrap();
        assert_eq!(
            public_key.serialize_compressed(),
            hex!("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1")
        );
    }
}
