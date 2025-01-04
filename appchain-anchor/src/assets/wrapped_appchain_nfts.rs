use near_contract_standards::non_fungible_token::metadata::NFTContractMetadata;

use crate::types::WrappedAppchainNFT;
use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InternalWrappedAppchainNFT {
    pub metadata: NFTContractMetadata,
    pub contract_account: AccountId,
    pub bridging_state: BridgingState,
    pub locked_token_id_set: UnorderedSet<String>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct WrappedAppchainNFTs {
    /// The set of class id of wrapped non-fungible tokens.
    class_id_set: UnorderedSet<String>,
    /// The non-fungible token data, mapped by the class id.
    nfts: LookupMap<String, InternalWrappedAppchainNFT>,
}

impl WrappedAppchainNFTs {
    ///
    pub fn new() -> Self {
        WrappedAppchainNFTs {
            class_id_set: UnorderedSet::new(StorageKey::WrappedAppchainNFTsClassIds.into_bytes()),
            nfts: LookupMap::new(StorageKey::WrappedAppchainNFTsNFTs.into_bytes()),
        }
    }
    ///
    pub fn to_vec(&self) -> Vec<WrappedAppchainNFT> {
        let mut results = Vec::<WrappedAppchainNFT>::new();
        let class_ids = self.class_id_set.to_vec();
        for class_id in class_ids {
            let internal_wrapped_appchain_nft = self.nfts.get(&class_id).unwrap();
            results.push(WrappedAppchainNFT {
                class_id: class_id.clone(),
                metadata: internal_wrapped_appchain_nft.metadata,
                contract_account: internal_wrapped_appchain_nft.contract_account,
                bridging_state: internal_wrapped_appchain_nft.bridging_state,
                count_of_locked_tokens: internal_wrapped_appchain_nft
                    .locked_token_id_set
                    .len()
                    .into(),
            });
        }
        results
    }
    //
    pub fn clear(&mut self) {
        let class_ids = self.class_id_set.to_vec();
        for class_id in class_ids {
            self.nfts.remove(&class_id);
        }
        self.class_id_set.clear();
    }
}
