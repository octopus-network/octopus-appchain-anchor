use beefy_light_client::{
    commitment::{Commitment, SignedCommitment},
    ValidatorProofCollection,
};
use codec::Decode;
use core::convert::TryFrom;

use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct StateCommitment {
    signed_commitment: SignedCommitment,
    verified_signature_count: u32,
    validator_proofs_leaf_set: UnorderedSet<Vec<u8>>,
    validator_proofs: LookupMap<Vec<u8>, ValidatorMerkleProof>,
    mmr_leaf: Vec<u8>,
    mmr_proof: Vec<u8>,
}

impl StateCommitment {
    ///
    pub fn new(
        signed_commitment: SignedCommitment,
        validator_proofs: Vec<ValidatorMerkleProof>,
        mmr_leaf: Vec<u8>,
        mmr_proof: Vec<u8>,
    ) -> Self {
        let mut instance = Self {
            signed_commitment,
            verified_signature_count: 0,
            validator_proofs_leaf_set: UnorderedSet::new(
                StorageKey::BeefyLightClientValidatorProofsLeafSet.into_bytes(),
            ),
            validator_proofs: LookupMap::new(
                StorageKey::BeefyLightClientValidatorProofsMap.into_bytes(),
            ),
            mmr_leaf,
            mmr_proof,
        };
        validator_proofs.iter().for_each(|proof| {
            instance.validator_proofs_leaf_set.insert(&proof.leaf);
            instance.validator_proofs.insert(&proof.leaf, proof);
        });
        instance
    }
    ///
    pub fn is_in_updating(&self) -> bool {
        let verified_signature_count = usize::try_from(self.verified_signature_count).unwrap();
        verified_signature_count < self.signed_commitment.signatures.len()
    }
    ///
    pub fn clear_storage(&mut self) {
        let proof_leaves = self.validator_proofs_leaf_set.to_vec();
        proof_leaves.iter().for_each(|leaf| {
            self.validator_proofs.remove(&leaf);
        });
        self.validator_proofs_leaf_set.clear();
    }
}

impl ValidatorProofCollection for StateCommitment {
    fn get_by_validator_address(&self, validator_address: Vec<u8>) -> Option<MerkleProof<Vec<u8>>> {
        match self.validator_proofs.get(&validator_address) {
            Some(proof) => Some(proof.to_merkle_proof()),
            None => None,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct BeefyLightClientState {
    processing_state_commitment: LazyOption<StateCommitment>,
    light_client: LazyOption<LightClient>,
}

impl BeefyLightClientState {
    ///
    pub fn new() -> Self {
        Self {
            processing_state_commitment: LazyOption::new(
                StorageKey::LightClientStateCommitmentInProcessing.into_bytes(),
                None,
            ),
            light_client: LazyOption::new(StorageKey::BeefyLightClientState.into_bytes(), None),
        }
    }
    ///
    pub fn assert_initialized(&self) {
        assert!(
            self.light_client.is_some(),
            "Beefy light client is not initialized."
        );
    }
    ///
    pub fn initialize(&mut self, public_keys: Vec<String>) {
        assert!(
            self.light_client.is_none(),
            "Beefy light client has already been initialized."
        );
        self.light_client.set(&beefy_light_client::new(public_keys));
    }
    ///
    pub fn get_latest_commitment(&self) -> Option<Commitment> {
        if let Some(light_client) = self.light_client.get() {
            light_client.get_latest_commitment()
        } else {
            None
        }
    }
    ///
    pub fn start_updating_state(
        &mut self,
        signed_commitment: Vec<u8>,
        validator_proofs: Vec<ValidatorMerkleProof>,
        mmr_leaf: Vec<u8>,
        mmr_proof: Vec<u8>,
    ) {
        env::log(format!("Gas used before start updating state: {}", env::used_gas()).as_bytes());
        self.assert_initialized();
        if self.processing_state_commitment.is_some() {
            let mut processing_state_commitment = self.processing_state_commitment.get().unwrap();
            assert!(
                !processing_state_commitment.is_in_updating(),
                "Beefy light client is still processing state updating."
            );
            processing_state_commitment.clear_storage();
        }
        let decoded_signed_commitment = SignedCommitment::decode(&mut &signed_commitment[..]);
        assert!(
            decoded_signed_commitment.is_ok(),
            "Invalid signed commitment."
        );
        let light_client = self.light_client.get().unwrap();
        let signed_commitment = decoded_signed_commitment.unwrap();
        assert!(
            light_client.state_is_older_than(&signed_commitment.commitment),
            "State commitment is too old."
        );
        self.processing_state_commitment.set(&StateCommitment::new(
            signed_commitment,
            validator_proofs,
            mmr_leaf,
            mmr_proof,
        ));
    }
    ///
    pub fn try_complete_updating_state(&mut self) -> MultiTxsOperationProcessingResult {
        self.assert_initialized();
        if self.processing_state_commitment.get().is_none() {
            return MultiTxsOperationProcessingResult::Ok;
        }
        let mut processing_state_commitment = self.processing_state_commitment.get().unwrap();
        if processing_state_commitment.verified_signature_count == u32::MAX {
            return MultiTxsOperationProcessingResult::Ok;
        }
        let mut light_client = self.light_client.get().unwrap();
        let commitment_hash = processing_state_commitment
            .signed_commitment
            .commitment
            .hash();
        let mut verified_signature_count =
            usize::try_from(processing_state_commitment.verified_signature_count).unwrap();
        while env::used_gas() < GAS_CAP_FOR_MULTI_TXS_PROCESSING
            && verified_signature_count
                < processing_state_commitment
                    .signed_commitment
                    .signatures
                    .len()
        {
            let signature = processing_state_commitment
                .signed_commitment
                .signatures
                .get(verified_signature_count)
                .unwrap();
            if let Some(signature) = signature {
                let result = light_client.verify_commitment_signature(
                    &commitment_hash,
                    signature,
                    &processing_state_commitment,
                );
                if result.is_err() {
                    return MultiTxsOperationProcessingResult::Error(format!("{:?}", result));
                }
            }
            verified_signature_count += 1;
        }
        if env::used_gas() >= GAS_CAP_FOR_MULTI_TXS_PROCESSING {
            processing_state_commitment.verified_signature_count =
                verified_signature_count.try_into().unwrap();
            self.processing_state_commitment
                .set(&processing_state_commitment);
            return MultiTxsOperationProcessingResult::NeedMoreGas;
        }
        let result = light_client.update_state(
            processing_state_commitment.signed_commitment.commitment,
            &processing_state_commitment.mmr_leaf,
            &processing_state_commitment.mmr_proof,
        );
        if result.is_err() {
            return MultiTxsOperationProcessingResult::Error(format!("{:?}", result));
        }
        processing_state_commitment.verified_signature_count = u32::MAX;
        self.processing_state_commitment
            .set(&processing_state_commitment);
        self.light_client.set(&light_client);
        MultiTxsOperationProcessingResult::Ok
    }
    ///
    pub fn verify_solochain_messages(
        &self,
        encoded_messages: &Vec<u8>,
        header: &Vec<u8>,
        mmr_leaf: &Vec<u8>,
        mmr_proof: &Vec<u8>,
    ) {
        self.assert_initialized();
        assert!(
            self.processing_state_commitment.is_none(),
            "Beefy light client is still processing state updating."
        );
        let light_client = self.light_client.get().unwrap();
        assert!(
            light_client
                .verify_solochain_messages(encoded_messages, header, mmr_leaf, mmr_proof)
                .is_ok(),
            "Invalid appchain messages."
        );
    }
}
