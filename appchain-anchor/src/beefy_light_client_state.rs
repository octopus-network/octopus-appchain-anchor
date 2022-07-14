use crate::*;
use beefy_light_client::{
    commitment::Commitment,
    validator_set::{BeefyNextAuthoritySet, ValidatorSetId},
    verifier_for_external_state_data::LightClientStateData,
};
use near_sdk::collections::UnorderedMap;

///
#[derive(BorshDeserialize, BorshSerialize)]
pub struct BeefyLightClientState {
    //
    authority_set_histories: UnorderedMap<ValidatorSetId, BeefyNextAuthoritySet>,
    //
    commitment_histories: UnorderedMap<(u32, ValidatorSetId), Commitment>,
}

impl LightClientStateData for BeefyLightClientState {
    //
    fn contains_authority_set(&self, set_id: &ValidatorSetId) -> bool {
        self.authority_set_histories.get(set_id).is_some()
    }
    //
    fn get_authority_set(&self, set_id: &ValidatorSetId) -> Option<BeefyNextAuthoritySet> {
        self.authority_set_histories.get(set_id)
    }
    //
    fn store_authority_set(&mut self, validator_set: &BeefyNextAuthoritySet) {
        self.authority_set_histories
            .insert(&validator_set.id, validator_set);
    }
    //
    fn contains_commitment(&self, commitment: &Commitment) -> bool {
        if let Some(stored_commitment) = self
            .commitment_histories
            .get(&(commitment.block_number, commitment.validator_set_id))
        {
            return stored_commitment == commitment.clone();
        }
        false
    }
    //
    fn get_commitment(
        &self,
        block_number: &u32,
        validator_set_id: &ValidatorSetId,
    ) -> Option<Commitment> {
        self.commitment_histories
            .get(&(*block_number, *validator_set_id))
    }
    //
    fn store_commitment(&mut self, commitment: &Commitment) {
        self.commitment_histories.insert(
            &(commitment.block_number, commitment.validator_set_id),
            commitment,
        );
    }
}

impl BeefyLightClientState {
    //
    pub fn new() -> Self {
        Self {
            authority_set_histories: UnorderedMap::new(
                StorageKey::BeefyAuthoritySetHistories.into_bytes(),
            ),
            commitment_histories: UnorderedMap::new(
                StorageKey::BeefyCommitmentHistories.into_bytes(),
            ),
        }
    }
    //
    pub fn status(&self) -> BeefyLightClientStatus {
        BeefyLightClientStatus {
            authority_set_ids: self
                .authority_set_histories
                .keys()
                .into_iter()
                .map(|f| U64::from(f))
                .collect(),
            commitment_keys: self
                .commitment_histories
                .keys()
                .into_iter()
                .map(|f| (f.0, U64::from(f.1)))
                .collect(),
        }
    }
    //
    pub fn clear(&mut self) -> MultiTxsOperationProcessingResult {
        todo!()
    }
}
