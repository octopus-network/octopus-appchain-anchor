use beefy_merkle_tree::{Hash, Keccak256};
use byteorder::{ByteOrder, LittleEndian};
use codec::Decode;
use core::convert::TryInto;

/// A signature (a 512-bit value, plus 8 bits for recovery ID).
#[derive(Debug, Decode, PartialEq, Eq)]
pub struct Signature(pub [u8; 65]);

impl From<&str> for Signature {
    fn from(hex_str: &str) -> Self {
        let data: [u8; 65] =
            hex::decode(&hex_str[2..]).map_or([0; 65], |s| s.try_into().unwrap_or([0; 65]));
        Self(data)
    }
}

#[derive(Debug, Decode)]
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

#[derive(Debug, Decode)]
pub struct SignedCommitment {
    pub commitment: Commitment,
    pub signatures: Vec<Option<Signature>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn signature_from_hex_str_works() {
        let signature_hex_str = "0x34c47a87fd892a2ed56f7f5708722548f7696578731c1119ba554c73c147433722da580d4daf04f5d13e1f4325a9639ad73aced975084982b5a97546cbf7bcc301";
        let signature: Signature = signature_hex_str.into();
        assert_eq!(signature, Signature(hex!("34c47a87fd892a2ed56f7f5708722548f7696578731c1119ba554c73c147433722da580d4daf04f5d13e1f4325a9639ad73aced975084982b5a97546cbf7bcc301").into()));
    }
}
