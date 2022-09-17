use crate::{
    interfaces::NativeNearTokenManager, permissionless_actions::AppchainMessagesProcessingContext,
    *,
};
use near_sdk::json_types::Base58CryptoHash;
use std::str::FromStr;

pub const PREFIX_OF_RECEIVER_SUB_ACCOUNT: &str = "near-vault";
pub const CONTRACT_ACCOUNT_FOR_NATIVE_NEAR_TOKEN: &str = "NEAR";

impl Default for NativeNearToken {
    fn default() -> Self {
        Self {
            locked_balance: U128::from(0),
            bridging_state: BridgingState::Closed,
            price_in_usd: U128::from(0),
        }
    }
}

#[near_bindgen]
impl NativeNearTokenManager for AppchainAnchor {
    ///
    fn deploy_near_vault_contract(&mut self) {
        self.assert_owner();
        assert!(
            env::storage_has_key(&StorageKey::NearVaultContractWasm.into_bytes()),
            "Wasm file for deployment is not staged yet."
        );
        let contract_account = AccountId::from_str(
            format!(
                "{}.{}",
                PREFIX_OF_RECEIVER_SUB_ACCOUNT,
                env::current_account_id()
            )
            .as_str(),
        )
        .unwrap();
        Promise::new(contract_account)
            .create_account()
            .transfer(NATIVE_NEAR_TOKEN_RECEIVER_CONTRACT_INIT_BALANCE)
            .add_full_access_key(self.owner_pk.clone())
            .deploy_contract(
                env::storage_read(&StorageKey::NearVaultContractWasm.into_bytes()).unwrap(),
            )
            .function_call(
                "new".to_string(),
                Vec::new(),
                0,
                Gas::ONE_TERA.mul(T_GAS_FOR_NATIVE_NEAR_RECEIVER_CONTRACT_INITIALIZATION),
            );
    }
    ///
    fn set_price_of_native_near_token(&mut self, price: U128) {
        self.assert_owner();
        let mut native_near_token = self.native_near_token.get().unwrap();
        assert!(
            native_near_token.price_in_usd.0 != price.0,
            "The price is not changed."
        );
        native_near_token.price_in_usd = price;
        self.native_near_token.set(&native_near_token);
    }
    ///
    fn open_bridging_of_native_near_token(&mut self) {
        self.assert_owner();
        let mut native_near_token = self.native_near_token.get().unwrap();
        assert!(
            !native_near_token.bridging_state.eq(&BridgingState::Active),
            "The bridging state is already 'active'."
        );
        native_near_token.bridging_state = BridgingState::Active;
        self.native_near_token.set(&native_near_token);
    }
    ///
    fn close_bridging_of_native_near_token(&mut self) {
        self.assert_owner();
        let mut native_near_token = self.native_near_token.get().unwrap();
        assert!(
            !native_near_token.bridging_state.eq(&BridgingState::Closed),
            "The bridging state is already 'closed'."
        );
        native_near_token.bridging_state = BridgingState::Closed;
        self.native_near_token.set(&native_near_token);
    }
    ///
    fn generate_appchain_notification_for_near_deposit(
        &mut self,
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: U128,
    ) {
        assert!(
            env::predecessor_account_id().eq(&AccountId::from_str(
                format!(
                    "{}.{}",
                    PREFIX_OF_RECEIVER_SUB_ACCOUNT,
                    env::current_account_id()
                )
                .as_str()
            )
            .unwrap()),
            "Can only be called by native near token receiver account."
        );
        self.assert_asset_transfer_is_not_paused();
        let mut native_near_token = self.native_near_token.get().unwrap();
        assert!(
            native_near_token.bridging_state.eq(&BridgingState::Active),
            "Bridging for native NEAR token is closed."
        );
        self.assert_locked_asset_on_near_side(None, &amount);
        native_near_token.locked_balance =
            U128::from(native_near_token.locked_balance.0 + amount.0);
        self.native_near_token.set(&native_near_token);
        let appchain_notification_history = self.internal_append_appchain_notification(
            AppchainNotification::NearFungibleTokenLocked {
                contract_account: String::from_str(CONTRACT_ACCOUNT_FOR_NATIVE_NEAR_TOKEN).unwrap(),
                sender_id_in_near,
                receiver_id_in_appchain: receiver_id_in_appchain.clone(),
                amount,
            },
        );
        log!(
            "Received native NEAR token. Start transfer to '{}' of appchain. Amount: '{}', Crosschain notification index: '{}'.",
            &receiver_id_in_appchain,
            &amount.0,
            &appchain_notification_history.index.0
        );
    }
}

impl NativeNearToken {
    //
    pub fn unlock_near(
        &mut self,
        receiver_id: &AccountId,
        amount: &U128,
        processing_context: &mut AppchainMessagesProcessingContext,
    ) -> MultiTxsOperationProcessingResult {
        //
        #[derive(near_sdk::serde::Serialize)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            receiver_id: AccountId,
            amount: U128,
        }
        let args = Input {
            receiver_id: receiver_id.clone(),
            amount: amount.clone(),
        };
        let args = near_sdk::serde_json::to_vec(&args)
            .expect("Failed to serialize the cross contract args using JSON.");
        let contract_account = AccountId::from_str(
            format!(
                "{}.{}",
                PREFIX_OF_RECEIVER_SUB_ACCOUNT,
                env::current_account_id()
            )
            .as_str(),
        )
        .unwrap();
        Promise::new(contract_account).function_call(
            "unlock_near".to_string(),
            args,
            0,
            Gas::ONE_TERA.mul(T_GAS_FOR_UNLOCK_NATIVE_NEAR),
        );
        processing_context.add_prepaid_gas(Gas::ONE_TERA.mul(T_GAS_FOR_UNLOCK_NATIVE_NEAR));
        self.locked_balance = U128::from(self.locked_balance.0 - amount.0);
        MultiTxsOperationProcessingResult::Ok
    }
}

/// Stores attached data into blob store and returns hash of it.
/// Implemented to avoid loading the data into WASM for optimal gas usage.
#[no_mangle]
pub extern "C" fn store_wasm_of_near_vault_contract() {
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

    env::storage_write(&StorageKey::NearVaultContractWasm.into_bytes(), &input);
    let mut blob_hash = [0u8; 32];
    blob_hash.copy_from_slice(&sha256_hash);
    let blob_hash_str = near_sdk::serde_json::to_string(&Base58CryptoHash::from(blob_hash))
        .unwrap()
        .into_bytes();

    env::value_return(&blob_hash_str);
}
