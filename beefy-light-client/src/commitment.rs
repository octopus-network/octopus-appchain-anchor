use beefy_merkle_tree::{Hash, Keccak256};
use byteorder::{ByteOrder, LittleEndian};

/// A signature (a 512-bit value, plus 8 bits for recovery ID).
#[derive(Debug)]
pub struct Signature(pub [u8; 65]);

#[derive(Debug)]
pub struct Commitment {
    pub payload: Hash,
    pub block_number: u64,
    pub validator_set_id: u32,
}

impl Commitment {
    pub fn hash(&self) -> Hash {
        let mut buf = [0_u8; 44];
        buf[0..32].copy_from_slice(&self.payload);
        LittleEndian::write_u64(&mut buf[32..40], self.block_number);
        LittleEndian::write_u32(&mut buf[40..44], self.validator_set_id);
        Keccak256::hash(&buf)
    }
}

pub struct SignedCommitment {
    pub commitment: Commitment,
    pub signatures: Vec<Option<Signature>>,
}
