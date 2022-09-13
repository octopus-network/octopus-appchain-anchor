use std::str::FromStr;

use near_contract_standards::non_fungible_token::metadata::NFTContractMetadata;
use near_sdk::json_types::Base58CryptoHash;
use near_sdk::Promise;

use crate::interfaces::WrappedAppchainNFTManager;
use crate::permissionless_actions::AppchainMessagesProcessingContext;
use crate::types::WrappedAppchainNFT;
use crate::*;

trait WrappedAppchainNFTContractResolver {
    /// Resolver for transfer wrapped appchain NFT
    fn resolve_wrapped_appchain_nft_transfer(
        &mut self,
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        class_id: String,
        instance_id: String,
        token_metadata: TokenMetadata,
        appchain_message_nonce: u32,
    );
    /// Resolver for mint wrapped appchain NFT
    fn resolve_wrapped_appchain_nft_mint(
        &mut self,
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        class_id: String,
        instance_id: String,
        token_metadata: TokenMetadata,
        appchain_message_nonce: u32,
    );
}

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
}

#[near_bindgen]
impl WrappedAppchainNFTManager for AppchainAnchor {
    //
    fn register_wrapped_appchain_nft(&mut self, class_id: String, metadata: NFTContractMetadata) {
        self.assert_owner();
        assert!(
            env::storage_has_key(&StorageKey::WrappedAppchainNFTContractWasm.into_bytes()),
            "Wasm file for deployment is not staged yet."
        );
        let internal_wrapped_appchain_nft =
            InternalWrappedAppchainNFT::new(class_id.clone(), metadata.clone());
        let mut wrapped_appchain_nfts = self.wrapped_appchain_nfts.get().unwrap();
        assert!(
            wrapped_appchain_nfts.get(&class_id).is_none(),
            "The given class id has already registered."
        );
        wrapped_appchain_nfts.insert(&class_id, &internal_wrapped_appchain_nft);
        self.wrapped_appchain_nfts.set(&wrapped_appchain_nfts);
        //
        #[derive(near_sdk::serde::Serialize)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            owner_id: AccountId,
            metadata: NFTContractMetadata,
        }
        let args = Input {
            owner_id: env::current_account_id(),
            metadata,
        };
        let args = near_sdk::serde_json::to_vec(&args)
            .expect("Failed to serialize the cross contract args using JSON.");
        Promise::new(internal_wrapped_appchain_nft.contract_account)
            .create_account()
            .transfer(WRAPPED_APPCHAIN_NFT_CONTRACT_INIT_BALANCE)
            .add_full_access_key(self.owner_pk.clone())
            .deploy_contract(
                env::storage_read(&StorageKey::WrappedAppchainNFTContractWasm.into_bytes())
                    .unwrap(),
            )
            .function_call(
                "new".to_string(),
                args,
                0,
                Gas::ONE_TERA.mul(T_GAS_FOR_NFT_CONTRACT_INITIALIZATION),
            );
    }
    //
    fn change_wrapped_appchain_nft_contract_metadata(
        &mut self,
        class_id: String,
        metadata: NFTContractMetadata,
    ) {
        self.assert_owner();
        let mut wrapped_appchain_nfts = self.wrapped_appchain_nfts.get().unwrap();
        if let Some(mut wrapped_appchain_nft) = wrapped_appchain_nfts.get(&class_id) {
            wrapped_appchain_nft.metadata = metadata;
            wrapped_appchain_nfts.insert(&class_id, &wrapped_appchain_nft);
        } else {
            panic!("Unregistered class id.");
        }
    }
    //
    fn open_bridging_of_wrapped_appchain_nft(&mut self, class_id: String) {
        self.assert_owner();
        let mut wrapped_appchain_nfts = self.wrapped_appchain_nfts.get().unwrap();
        if let Some(mut wrapped_appchain_nft) = wrapped_appchain_nfts.get(&class_id) {
            assert!(
                wrapped_appchain_nft
                    .bridging_state
                    .ne(&BridgingState::Active),
                "Bridging is already active."
            );
            wrapped_appchain_nft.bridging_state = BridgingState::Active;
            wrapped_appchain_nfts.insert(&class_id, &wrapped_appchain_nft);
        } else {
            panic!("Unregistered class id.");
        }
    }
    //
    fn close_bridging_of_wrapped_appchain_nft(&mut self, class_id: String) {
        self.assert_owner();
        let mut wrapped_appchain_nfts = self.wrapped_appchain_nfts.get().unwrap();
        if let Some(mut wrapped_appchain_nft) = wrapped_appchain_nfts.get(&class_id) {
            assert!(
                wrapped_appchain_nft
                    .bridging_state
                    .ne(&BridgingState::Closed),
                "Bridging is already closed."
            );
            wrapped_appchain_nft.bridging_state = BridgingState::Closed;
            wrapped_appchain_nfts.insert(&class_id, &wrapped_appchain_nft);
        } else {
            panic!("Unregistered class id.");
        }
    }
}

impl AppchainAnchor {
    //
    pub fn internal_process_locked_nft_in_appchain(
        &mut self,
        processing_context: &mut AppchainMessagesProcessingContext,
        appchain_message_nonce: u32,
        owner_id_in_appchain: &String,
        receiver_id_in_near: &AccountId,
        class_id: &String,
        instance_id: &String,
        token_metadata: &TokenMetadata,
    ) -> MultiTxsOperationProcessingResult {
        let wrapped_appchain_nfts = self.wrapped_appchain_nfts.get().unwrap();
        if let Some(wrapped_appchain_nft) = wrapped_appchain_nfts.get(&class_id) {
            if wrapped_appchain_nft
                .bridging_state
                .eq(&BridgingState::Closed)
            {
                let message = format!(
                    "Bridging for wrapped appchain NFT in contract '{}' is now closed.",
                    wrapped_appchain_nft.contract_account
                );
                let result = AppchainMessageProcessingResult::Error {
                    nonce: appchain_message_nonce,
                    message: message.clone(),
                };
                self.record_appchain_message_processing_result(&result);
                return MultiTxsOperationProcessingResult::Error(message);
            }
            if wrapped_appchain_nft.is_nft_locked(&instance_id) {
                #[derive(near_sdk::serde::Serialize)]
                #[serde(crate = "near_sdk::serde")]
                struct Args {
                    receiver_id: AccountId,
                    token_id: TokenId,
                    approval_id: Option<u64>,
                    memo: Option<String>,
                }
                let args = Args {
                    receiver_id: receiver_id_in_near.clone(),
                    token_id: instance_id.clone(),
                    approval_id: None,
                    memo: None,
                };
                let args = near_sdk::serde_json::to_vec(&args)
                    .expect("Failed to serialize the cross contract args using JSON.");
                Promise::new(wrapped_appchain_nft.contract_account)
                    .function_call(
                        "nft_tranfser".to_string(),
                        args,
                        1,
                        Gas::ONE_TERA.mul(T_GAS_FOR_NFT_TRANSFER),
                    )
                    .then(
                        ext_self::ext(env::current_account_id())
                            .with_attached_deposit(0)
                            .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_RESOLVER_FUNCTION))
                            .with_unused_gas_weight(0)
                            .resolve_wrapped_appchain_nft_transfer(
                                owner_id_in_appchain.clone(),
                                receiver_id_in_near.clone(),
                                class_id.clone(),
                                instance_id.clone(),
                                token_metadata.clone(),
                                appchain_message_nonce,
                            ),
                    );
                processing_context.add_prepaid_gas(Gas::ONE_TERA.mul(T_GAS_FOR_FT_TRANSFER));
                processing_context.add_prepaid_gas(Gas::ONE_TERA.mul(T_GAS_FOR_RESOLVER_FUNCTION));
                MultiTxsOperationProcessingResult::Ok
            } else {
                #[derive(near_sdk::serde::Serialize)]
                #[serde(crate = "near_sdk::serde")]
                struct Args {
                    token_id: TokenId,
                    token_owner_id: AccountId,
                    token_metadata: TokenMetadata,
                }
                let args = Args {
                    token_id: instance_id.clone(),
                    token_owner_id: receiver_id_in_near.clone(),
                    token_metadata: token_metadata.clone(),
                };
                let args = near_sdk::serde_json::to_vec(&args)
                    .expect("Failed to serialize the cross contract args using JSON.");
                Promise::new(wrapped_appchain_nft.contract_account)
                    .function_call(
                        "nft_mint".to_string(),
                        args,
                        STORAGE_DEPOSIT_FOR_MINT_NFT,
                        Gas::ONE_TERA.mul(T_GAS_FOR_MINT_NFT),
                    )
                    .then(
                        ext_self::ext(env::current_account_id())
                            .with_attached_deposit(0)
                            .with_static_gas(Gas::ONE_TERA.mul(T_GAS_FOR_RESOLVER_FUNCTION))
                            .with_unused_gas_weight(0)
                            .resolve_wrapped_appchain_nft_mint(
                                owner_id_in_appchain.clone(),
                                receiver_id_in_near.clone(),
                                class_id.clone(),
                                instance_id.clone(),
                                token_metadata.clone(),
                                appchain_message_nonce,
                            ),
                    );
                processing_context.add_prepaid_gas(Gas::ONE_TERA.mul(T_GAS_FOR_FT_TRANSFER));
                processing_context.add_prepaid_gas(Gas::ONE_TERA.mul(T_GAS_FOR_RESOLVER_FUNCTION));
                MultiTxsOperationProcessingResult::Ok
            }
        } else {
            let message = format!(
                "Unregistered class id of wrapped appchain NFT: '{}'",
                class_id
            );
            let result = AppchainMessageProcessingResult::Error {
                nonce: appchain_message_nonce,
                message: message.clone(),
            };
            self.record_appchain_message_processing_result(&result);
            return MultiTxsOperationProcessingResult::Error(message);
        }
    }
    //
    pub fn internal_process_nft_transfer(
        &mut self,
        predecessor_account_id: AccountId,
        sender_id: AccountId,
        nft_owner_id: AccountId,
        token_id: TokenId,
        transfer_message: NFTTransferMessage,
    ) -> PromiseOrValue<bool> {
        let mut wrapped_appchain_nfts = self.wrapped_appchain_nfts.get().unwrap();
        if let Some(mut wrapped_appchain_nft) =
            wrapped_appchain_nfts.get_by_contract_account(&predecessor_account_id)
        {
            assert!(
                wrapped_appchain_nft
                    .bridging_state
                    .eq(&BridgingState::Active),
                "Bridging for '{}({})' is closed.",
                wrapped_appchain_nft.metadata.symbol,
                wrapped_appchain_nft.metadata.name
            );
            match transfer_message {
                NFTTransferMessage::BridgeToAppchain {
                    receiver_id_in_appchain,
                } => {
                    AccountIdInAppchain::new(
                        Some(receiver_id_in_appchain.clone()),
                        &self.appchain_template_type,
                    )
                    .assert_valid();
                    wrapped_appchain_nft.add_locked_nft(&token_id);
                    let class_id = wrapped_appchain_nfts
                        .get_class_id_by_contract_account(&predecessor_account_id)
                        .unwrap();
                    wrapped_appchain_nfts.insert(&class_id, &wrapped_appchain_nft);
                    let appchain_notification_history = self.internal_append_appchain_notification(
                        AppchainNotification::WrappedAppchainNFTLocked {
                            class_id,
                            token_id: token_id.clone(),
                            sender_id_in_near: sender_id.clone(),
                            owner_id_in_near: nft_owner_id.clone(),
                            receiver_id_in_appchain: receiver_id_in_appchain.clone(),
                        },
                    );
                    log!(
                        "Received NFT in contract '{}' from '{}'. Start transfer to '{}' of appchain. Crosschain notification index: '{}'.",
                        predecessor_account_id,
                        sender_id.clone(),
                        receiver_id_in_appchain,
                        appchain_notification_history.index.0
                    );
                    return PromiseOrValue::Value(false);
                }
            }
        }
        panic!(
            "Received unregistered token id '{}' of wrapped appchain NFT in contract '{}' from '{}'. Return it.",
            token_id, predecessor_account_id, sender_id,
        );
    }
}

#[near_bindgen]
impl WrappedAppchainNFTContractResolver for AppchainAnchor {
    //
    fn resolve_wrapped_appchain_nft_transfer(
        &mut self,
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        class_id: String,
        instance_id: String,
        token_metadata: TokenMetadata,
        appchain_message_nonce: u32,
    ) {
        assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let mut wrapped_appchain_nfts = self.wrapped_appchain_nfts.get().unwrap();
                if let Some(mut wrapped_appchain_nft) = wrapped_appchain_nfts.get(&class_id) {
                    wrapped_appchain_nft.remove_locked_nft(&instance_id);
                    wrapped_appchain_nfts.insert(&class_id, &wrapped_appchain_nft);
                };
                self.record_appchain_message_processing_result(
                    &AppchainMessageProcessingResult::Ok {
                        nonce: appchain_message_nonce,
                        message: None,
                    },
                );
            }
            PromiseResult::Failed => {
                let wrapped_appchain_nfts = self.wrapped_appchain_nfts.get().unwrap();
                let reason = format!(
                    "Maybe the receiver account '{}' is not registered in contract '{}'.",
                    receiver_id_in_near,
                    wrapped_appchain_nfts
                        .get(&class_id)
                        .unwrap()
                        .contract_account,
                );
                let message = format!(
                    "Failed to transfer NFT from appchain account '{}' with metadata '{}'. {}",
                    owner_id_in_appchain,
                    serde_json::to_string(&token_metadata).unwrap(),
                    reason
                );
                self.record_appchain_message_processing_result(
                    &AppchainMessageProcessingResult::Error {
                        nonce: appchain_message_nonce,
                        message,
                    },
                );
            }
        }
    }
    //
    fn resolve_wrapped_appchain_nft_mint(
        &mut self,
        owner_id_in_appchain: String,
        receiver_id_in_near: AccountId,
        class_id: String,
        instance_id: String,
        token_metadata: TokenMetadata,
        appchain_message_nonce: u32,
    ) {
        assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                let message = format!(
                    "NFT '{}' from appchain account '{}' with metadata '{}' is minted.",
                    instance_id,
                    owner_id_in_appchain,
                    serde_json::to_string(&token_metadata).unwrap(),
                );
                self.record_appchain_message_processing_result(
                    &AppchainMessageProcessingResult::Ok {
                        nonce: appchain_message_nonce,
                        message: Some(message),
                    },
                );
            }
            PromiseResult::Failed => {
                let wrapped_appchain_nfts = self.wrapped_appchain_nfts.get().unwrap();
                let reason = format!(
                    "Maybe the receiver account '{}' is not registered in contract '{}'.",
                    receiver_id_in_near,
                    wrapped_appchain_nfts
                        .get(&class_id)
                        .unwrap()
                        .contract_account,
                );
                let message = format!(
                    "Failed to mint NFT '{}' from appchain account '{}' with metadata '{}'. {}",
                    instance_id,
                    owner_id_in_appchain,
                    serde_json::to_string(&token_metadata).unwrap(),
                    reason
                );
                self.record_appchain_message_processing_result(
                    &AppchainMessageProcessingResult::Error {
                        nonce: appchain_message_nonce,
                        message,
                    },
                );
            }
        }
    }
}

/// Stores attached data into blob store and returns hash of it.
/// Implemented to avoid loading the data into WASM for optimal gas usage.
#[no_mangle]
pub extern "C" fn store_wasm_of_wrapped_appchain_nft_contract() {
    env::setup_panic_hook();
    let contract: AppchainAnchor = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
    contract.assert_owner();
    let input = env::input().expect("ERR_NO_INPUT");
    let sha256_hash = env::sha256(&input);

    let blob_len = input.len();
    let storage_cost = ((blob_len + 32) as u128) * env::storage_byte_cost();
    assert!(
        env::attached_deposit() >= storage_cost,
        "ERR_NOT_ENOUGH_DEPOSIT:{}",
        storage_cost
    );

    env::storage_write(
        &StorageKey::WrappedAppchainNFTContractWasm.into_bytes(),
        &input,
    );
    let mut blob_hash = [0u8; 32];
    blob_hash.copy_from_slice(&sha256_hash);
    let blob_hash_str = near_sdk::serde_json::to_string(&Base58CryptoHash::from(blob_hash))
        .unwrap()
        .into_bytes();

    env::value_return(&blob_hash_str);
}
