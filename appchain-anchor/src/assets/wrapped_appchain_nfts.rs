use std::str::FromStr;

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

impl InternalWrappedAppchainNFT {
    ///
    pub fn new(class_id: String, metadata: NFTContractMetadata) -> Self {
        let sub_account_id = format!("{}.{}", class_id, env::current_account_id());
        let contract_account = AccountId::from_str(sub_account_id.as_str());
        assert!(contract_account.is_ok(), "Invalid class id.");
        Self {
            metadata,
            contract_account: contract_account.unwrap(),
            bridging_state: BridgingState::Closed,
            locked_token_id_set: UnorderedSet::new(
                StorageKey::WrappedAppchainNFTsLockedTokenIdSet(class_id).into_bytes(),
            ),
        }
    }
    ///
    pub fn is_nft_locked(&self, token_id: &TokenId) -> bool {
        self.locked_token_id_set.contains(token_id)
    }
    ///
    pub fn add_locked_nft(&mut self, token_id: &TokenId) {
        self.locked_token_id_set.insert(token_id);
    }
    ///
    pub fn remove_locked_nft(&mut self, token_id: &TokenId) {
        self.locked_token_id_set.remove(token_id);
    }
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
    pub fn insert(
        &mut self,
        class_id: &String,
        internal_wrapped_appchain_nft: &InternalWrappedAppchainNFT,
    ) {
        self.class_id_set.insert(&class_id);
        self.nfts.insert(class_id, internal_wrapped_appchain_nft);
    }
    ///
    pub fn get(&self, class_id: &String) -> Option<InternalWrappedAppchainNFT> {
        self.nfts.get(class_id)
    }
    ///
    pub fn get_by_contract_account(
        &self,
        account_id: &AccountId,
    ) -> Option<InternalWrappedAppchainNFT> {
        let class_ids = self.class_id_set.to_vec();
        for class_id in class_ids {
            let internal_wrapped_appchain_nft = self.nfts.get(&class_id).unwrap();
            if internal_wrapped_appchain_nft
                .contract_account
                .eq(account_id)
            {
                return Some(internal_wrapped_appchain_nft);
            }
        }
        None
    }
    ///
    pub fn get_class_id_by_contract_account(&self, account_id: &AccountId) -> Option<String> {
        let class_ids = self.class_id_set.to_vec();
        for class_id in class_ids {
            let internal_wrapped_appchain_nft = self.nfts.get(&class_id).unwrap();
            if internal_wrapped_appchain_nft
                .contract_account
                .eq(account_id)
            {
                return Some(class_id);
            }
        }
        None
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
