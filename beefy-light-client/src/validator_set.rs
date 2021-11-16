use beefy_merkle_tree::Hash;
use borsh::{BorshDeserialize, BorshSerialize};
use codec::Encode;

/// A typedef for validator set id.
pub type ValidatorSetId = u64;

pub type Public = [u8; 33];

#[derive(Debug, Default, Encode, BorshDeserialize, BorshSerialize)]
pub struct BeefyNextAuthoritySet {
    /// Id of the next set.
    ///
    /// Id is required to correlate BEEFY signed commitments with the validator set.
    /// Light Client can easily verify that the commitment witness it is getting is
    /// produced by the latest validator set.
    pub id: ValidatorSetId,
    /// Number of validators in the set.
    ///
    /// Some BEEFY Light Clients may use an interactive protocol to verify only subset
    /// of signatures. We put set length here, so that these clients can verify the minimal
    /// number of required signatures.
    pub len: u32,
    /// Merkle Root Hash build from BEEFY AuthorityIds.
    ///
    /// This is used by Light Clients to confirm that the commitments are signed by the correct
    /// validator set. Light Clients using interactive protocol, might verify only subset of
    /// signatures, hence don't require the full list here (will receive inclusion proofs).
    pub root: Hash,
}
