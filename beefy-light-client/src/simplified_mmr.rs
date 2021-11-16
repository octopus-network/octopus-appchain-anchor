use beefy_merkle_tree::{Hash, Keccak256};

// Get the value of the bit at the given 'index' in 'n'.
// index should be validated beforehand to make sure it is less than 64
fn bit(n: u64, index: usize) -> bool {
    (n >> index & 1) == 1
}

fn merkle_root(
    leaf_node_hash: Hash,
    merkle_proof_items: Vec<Hash>,
    merkle_proof_order_bit_field: u64,
) -> Hash {
    let mut current_hash = leaf_node_hash;

    for current_position in 0..merkle_proof_items.len() {
        let is_sibling_left = bit(merkle_proof_order_bit_field, current_position);
        let sibling = merkle_proof_items[current_position];

        let mut combined = [0_u8; 64];
        if is_sibling_left {
            combined[0..32].copy_from_slice(&sibling);
            combined[32..64].copy_from_slice(&current_hash);
        } else {
            combined[0..32].copy_from_slice(&current_hash);
            combined[32..64].copy_from_slice(&sibling);
        }
        current_hash = Keccak256::hash(&combined);
    }

    current_hash
}

pub fn verify_proof(root: Hash, leaf_node_hash: Hash, proof: MerkleProof) -> bool {
    if proof.merkle_proof_items.len() < 64 {
        root == merkle_root(
            leaf_node_hash,
            proof.merkle_proof_items,
            proof.merkle_proof_order_bit_field,
        )
    } else {
        false
    }
}

pub struct MerkleProof {
    merkle_proof_items: Vec<Hash>,
    merkle_proof_order_bit_field: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn should_verify_proof_1() {
        let mmr_root = hex!("f85f275b6b06c233fc62ecb5992cd3b3396982ecef9c9508e615c6f528c8fc25");
        let leaf_node_hash =
            hex!("f4aac2fbe33f03554bfeb559ea2690ed8521caa4be961e61c91ac9a1530dce7a");
        let leaf_proof = MerkleProof {
            merkle_proof_items: vec![
                hex!("2fc249826fa000037981cc3446a7e0ad347c8446525dc7958723ea3afc7209de"),
                hex!("b5d6bae5432161e6ce0fdfd28ea26011f581ad68335e77cf68864f4911879257"),
            ],

            merkle_proof_order_bit_field: 0,
        };

        assert!(verify_proof(mmr_root, leaf_node_hash, leaf_proof));
    }

    #[test]
    fn should_verify_proof_2() {
        let mmr_root = hex!("362b201244f8ec314f4995918ac70a19ba818d4d41e78c9634ff6d281af3c4c1");
        let leaf_node_hash =
            hex!("11da6d1f761ddf9bdb4c9d6e5303ebd41f61858d0a5647a1a7bfe089bf921be9");
        let leaf_proof = MerkleProof {
            merkle_proof_items: vec![
                hex!("e12c22d4f162d9a012c9319233da5d3e923cc5e1029b8f90e47249c9ab256b35"),
                hex!("513bf90be61a0fa9099a23510fc22436cf364f837d7d455fc6b13903874e98b9"),
                hex!("c540f6cc8db70e3f37bf564d202563d3d323b761f97bb1bf44b85c48f8f38a16"),
                hex!("6cec581ba72ef0a8b48c0a05fa9dc904775032adadbac83d10b2dbdf05a2f8a7"),
            ],

            merkle_proof_order_bit_field: 8,
        };

        assert!(verify_proof(mmr_root, leaf_node_hash, leaf_proof));
    }
}
