use crate::BeefyNextAuthoritySet;
use beefy_merkle_tree::{verify_proof, Hash, Keccak256, Leaf, MerkleProof};
use codec::Encode;

#[cfg(not(feature = "std"))]
use core::convert::Into;

#[derive(Debug, Default, Encode)]
pub struct MmrLeafVersion(u8);
impl MmrLeafVersion {
    /// Create new version object from `major` and `minor` components.
    ///
    /// Panics if any of the component occupies more than 4 bits.
    pub fn new(major: u8, minor: u8) -> Self {
        if major > 0b111 || minor > 0b11111 {
            panic!("Version components are too big.");
        }
        let version = (major << 5) + minor;
        Self(version)
    }

    /// Split the version into `major` and `minor` sub-components.
    pub fn split(&self) -> (u8, u8) {
        let major = self.0 >> 5;
        let minor = self.0 & 0b11111;
        (major, minor)
    }
}

#[derive(Debug, Default, Encode)]
pub struct MmrLeaf {
    /// Version of the leaf format.
    ///
    /// Can be used to enable future format migrations and compatibility.
    /// See [`MmrLeafVersion`] documentation for details.
    pub version: MmrLeafVersion,
    /// Current block parent number and hash.
    pub parent_number_and_hash: (u32, Hash),
    /// A merkle root of the next BEEFY authority set.
    pub beefy_next_authority_set: BeefyNextAuthoritySet,
}

impl MmrLeaf {
    pub fn hash(&self) -> Hash {
        Keccak256::hash(&self.encode())
    }
}

impl<'a> From<MmrLeaf> for Leaf<'a> {
    fn from(v: MmrLeaf) -> Self {
        Leaf::Hash(v.hash())
    }
}

/// A MMR proof data for one of the leaves.
#[derive(Debug, Default, Encode)]
pub struct MmrLeafProof {
    /// The index of the leaf the proof is for.
    pub leaf_index: u64,
    /// Number of leaves in MMR, when the proof was generated.
    pub leaf_count: u64,
    /// Proof elements (hashes of siblings of inner nodes on the path to the leaf).
    pub items: Vec<Hash>,
}

/// MMR nodes & size -related utilities.
pub struct NodesUtils {
    no_of_leaves: u64,
}

impl NodesUtils {
    /// Create new instance of MMR nodes utilities for given number of leaves.
    pub fn new(no_of_leaves: u64) -> Self {
        Self { no_of_leaves }
    }

    /// Calculate number of peaks in the MMR.
    pub fn number_of_peaks(&self) -> u64 {
        self.number_of_leaves().count_ones() as u64
    }

    /// Return the number of leaves in the MMR.
    pub fn number_of_leaves(&self) -> u64 {
        self.no_of_leaves
    }

    /// Calculate the total size of MMR (number of nodes).
    pub fn size(&self) -> u64 {
        2 * self.no_of_leaves - self.number_of_peaks()
    }

    /// Calculate maximal depth of the MMR.
    pub fn depth(&self) -> u32 {
        if self.no_of_leaves == 0 {
            return 0;
        }

        64 - self.no_of_leaves.next_power_of_two().leading_zeros()
    }
}
pub fn leaf_index_to_pos(index: u64) -> u64 {
    // mmr_size - H - 1, H is the height(intervals) of last peak
    leaf_index_to_mmr_size(index) - (index + 1).trailing_zeros() as u64 - 1
}

pub fn leaf_index_to_mmr_size(index: u64) -> u64 {
    // leaf index start with 0
    let leaves_count = index + 1;

    // the peak count(k) is actually the count of 1 in leaves count's binary representation
    let peak_count = leaves_count.count_ones() as u64;

    2 * leaves_count - peak_count
}

/// Stateless verification of the leaf proof.
pub fn verify_leaf_proof(
    root: Hash,
    leaf: MmrLeaf,
    proof: MmrLeafProof,
) -> Result<bool, crate::Error> {
    let size = NodesUtils::new(proof.leaf_count).size();
    let leaf_position = leaf_index_to_pos(proof.leaf_index);
    let p = MerkleProof {
        root,
        proof: proof.items,
        number_of_leaves: size as usize,
        leaf_index: leaf_position as usize,
        leaf,
    };

    if !verify_proof::<Keccak256, _, _>(&root, p.proof, p.number_of_leaves, p.leaf_index, p.leaf) {
        return Err(crate::Error::InvalidMmrProof);
    }
    Ok(true)
}
