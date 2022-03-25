use core::convert::TryFrom;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::json_types::I128;

use crate::{
    interfaces::WrappedAppchainTokenManager,
    permissionless_actions::AppchainMessagesProcessingContext, *,
};

pub trait WrappedAppchainTokenContractResolver {
    /// Resolver for burning wrapped appchain token
    fn resolve_wrapped_appchain_token_burning(
        &mut self,
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: U128,
    );
    /// Resolver for minting wrapped appchain token
    fn resolve_wrapped_appchain_token_minting(
        &mut self,
        sender_id_in_appchain: Option<String>,
        receiver_id_in_near: AccountId,
        amount: U128,
        appchain_message_nonce: u32,
    );
}

impl Default for WrappedAppchainToken {
    fn default() -> Self {
        Self {
            metadata: FungibleTokenMetadata {
                spec: "ft-1.0.0".to_string(),
                symbol: String::new(),
                name: String::new(),
                decimals: 0,
                icon: None,
                reference: None,
                reference_hash: None,
            },
            contract_account: AccountId::new(),
            premined_beneficiary: AccountId::new(),
            premined_balance: U128::from(0),
            changed_balance: I128::from(0),
            price_in_usd: U128::from(0),
            total_supply: U128::from(0),
        }
    }
}

impl WrappedAppchainToken {
    ///
    pub fn total_market_value(&self) -> Balance {
        let total_balance: i128 =
            i128::try_from(self.premined_balance.0).unwrap() + self.changed_balance.0;
        u128::try_from(total_balance).unwrap() / u128::pow(10, u32::from(self.metadata.decimals))
            * self.price_in_usd.0
    }
    ///
    pub fn get_market_value_of(&self, amount: u128) -> Balance {
        amount / u128::pow(10, u32::from(self.metadata.decimals)) * self.price_in_usd.0
    }
}

#[near_bindgen]
impl WrappedAppchainTokenManager for AppchainAnchor {
    //
    fn sync_basedata_of_wrapped_appchain_token(
        &mut self,
        metadata: FungibleTokenMetadata,
        premined_beneficiary: AccountId,
        premined_balance: U128,
    ) {
        let mut wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
        assert_eq!(
            env::predecessor_account_id(),
            wrapped_appchain_token.contract_account,
            "Only '{}' can call this function.",
            wrapped_appchain_token.contract_account
        );
        wrapped_appchain_token.metadata = metadata;
        wrapped_appchain_token.premined_beneficiary = premined_beneficiary;
        wrapped_appchain_token.premined_balance = premined_balance;
        wrapped_appchain_token.total_supply = premined_balance;
        self.wrapped_appchain_token.set(&wrapped_appchain_token);
    }
    //
    fn set_account_of_wrapped_appchain_token(&mut self, contract_account: AccountId) {
        self.assert_owner();
        let mut wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
        wrapped_appchain_token.contract_account = contract_account;
        self.wrapped_appchain_token.set(&wrapped_appchain_token);
    }
    //
    fn set_total_supply_of_wrapped_appchain_token(&mut self, total_supply: U128) {
        self.assert_owner();
        let mut wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
        wrapped_appchain_token.total_supply = total_supply;
        self.wrapped_appchain_token.set(&wrapped_appchain_token);
    }
    //
    fn set_price_of_wrapped_appchain_token(&mut self, price: U128) {
        let anchor_settings = self.anchor_settings.get().unwrap();
        assert_eq!(
            env::predecessor_account_id(),
            anchor_settings.token_price_maintainer_account,
            "Only '{}' can call this function.",
            anchor_settings.token_price_maintainer_account
        );
        let mut wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
        wrapped_appchain_token.price_in_usd = price;
        self.wrapped_appchain_token.set(&wrapped_appchain_token);
    }
    //
    fn burn_wrapped_appchain_token(&self, receiver_id: String, amount: U128) {
        self.assert_asset_transfer_is_not_paused();
        let sender_id = env::predecessor_account_id();
        let account_id_in_appchain = AccountIdInAppchain::new(Some(receiver_id.clone()));
        account_id_in_appchain.assert_valid();
        ext_fungible_token::burn(
            sender_id.clone(),
            amount,
            &self.wrapped_appchain_token.get().unwrap().contract_account,
            1,
            GAS_FOR_BURN_FUNGIBLE_TOKEN,
        )
        .then(ext_self::resolve_wrapped_appchain_token_burning(
            sender_id.clone(),
            receiver_id.clone(),
            amount,
            &env::current_account_id(),
            0,
            GAS_FOR_RESOLVER_FUNCTION,
        ));
    }
}

impl AppchainAnchor {
    //
    pub fn internal_mint_wrapped_appchain_token(
        &mut self,
        sender_id: Option<String>,
        receiver_id: AccountId,
        amount: U128,
        appchain_message_nonce: u32,
        processing_context: &mut AppchainMessagesProcessingContext,
    ) -> MultiTxsOperationProcessingResult {
        let wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
        if let Some(sender_id) = &sender_id {
            if !AccountIdInAppchain::new(Some(sender_id.clone())).is_valid() {
                let message = format!("Invalid sender id in appchain: '{}'", sender_id);
                let result = AppchainMessageProcessingResult::Error {
                    nonce: appchain_message_nonce,
                    message: message.clone(),
                };
                self.record_appchain_message_processing_result(&result);
                return MultiTxsOperationProcessingResult::Error(message);
            }
        }
        ext_fungible_token::mint(
            receiver_id.clone(),
            amount,
            &wrapped_appchain_token.contract_account,
            STORAGE_DEPOSIT_FOR_NEP141_TOEKN,
            GAS_FOR_MINT_FUNGIBLE_TOKEN,
        )
        .then(ext_self::resolve_wrapped_appchain_token_minting(
            sender_id.clone(),
            receiver_id.clone(),
            amount,
            appchain_message_nonce,
            &env::current_account_id(),
            0,
            GAS_FOR_RESOLVER_FUNCTION,
        ));
        processing_context.add_prepaid_gas(GAS_FOR_MINT_FUNGIBLE_TOKEN + GAS_FOR_RESOLVER_FUNCTION);
        MultiTxsOperationProcessingResult::Ok
    }
}

#[near_bindgen]
impl WrappedAppchainTokenContractResolver for AppchainAnchor {
    //
    fn resolve_wrapped_appchain_token_burning(
        &mut self,
        sender_id_in_near: AccountId,
        receiver_id_in_appchain: String,
        amount: U128,
    ) {
        assert_self();
        let mut wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                wrapped_appchain_token.changed_balance = I128::from(
                    wrapped_appchain_token.changed_balance.0 - i128::try_from(amount.0).unwrap(),
                );
                self.wrapped_appchain_token.set(&wrapped_appchain_token);
                self.internal_append_anchor_event(AnchorEvent::WrappedAppchainTokenBurnt {
                    sender_id_in_near: sender_id_in_near.clone(),
                    receiver_id_in_appchain: receiver_id_in_appchain.clone(),
                    amount: U128::from(amount),
                });
                let appchain_notification_history = self.internal_append_appchain_notification(
                    AppchainNotification::WrappedAppchainTokenBurnt {
                        sender_id_in_near: sender_id_in_near.clone(),
                        receiver_id_in_appchain: receiver_id_in_appchain.clone(),
                        amount: U128::from(amount),
                    },
                );
                log!(
                    "Wrapped appchain token burnt in contract '{}' by '{}' for '{}' of appchain. Amount: '{}', Crosschain notification index: '{}'.",
                    &wrapped_appchain_token.contract_account,
                    &sender_id_in_near,
                    &receiver_id_in_appchain,
                    &amount.0,
                    &appchain_notification_history.index.0
                );
            }
            PromiseResult::Failed => {
                log!(
                    "Failed to burn wrapped appchain token in contract '{}' by '{}' for '{}' in appchain. Amount: '{}'",
                    &wrapped_appchain_token.contract_account,
                    &sender_id_in_near,
                    &receiver_id_in_appchain,
                    &amount.0
                );
                self.internal_append_anchor_event(AnchorEvent::FailedToBurnWrappedAppchainToken {
                    sender_id_in_near: sender_id_in_near.clone(),
                    receiver_id_in_appchain,
                    amount: U128::from(amount),
                    reason: format!(
                        "Maybe the balance of '{}' is not enough.",
                        &sender_id_in_near
                    ),
                });
            }
        }
    }
    //
    fn resolve_wrapped_appchain_token_minting(
        &mut self,
        sender_id_in_appchain: Option<String>,
        receiver_id_in_near: AccountId,
        amount: U128,
        appchain_message_nonce: u32,
    ) {
        assert_self();
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {
                self.internal_append_anchor_event(AnchorEvent::WrappedAppchainTokenMinted {
                    sender_id_in_appchain: sender_id_in_appchain.clone(),
                    receiver_id_in_near: receiver_id_in_near.clone(),
                    amount: U128::from(amount),
                    appchain_message_nonce,
                });
                let mut wrapped_appchain_token = self.wrapped_appchain_token.get().unwrap();
                wrapped_appchain_token.changed_balance = I128::from(
                    wrapped_appchain_token.changed_balance.0 + i128::try_from(amount.0).unwrap(),
                );
                self.wrapped_appchain_token.set(&wrapped_appchain_token);
                let message = match sender_id_in_appchain {
                    Some(sender_id) => format!(
                        "Wrapped appchain token minted by '{}' of appchain for '{}' with amount '{}'.",
                        &sender_id, &receiver_id_in_near, &amount.0
                    ),
                    None => format!(
                        "Wrapped appchain token minted by crosschain message for '{}' with amount '{}'.",
                        &receiver_id_in_near, &amount.0
                    ),
                };
                self.record_appchain_message_processing_result(
                    &AppchainMessageProcessingResult::Ok {
                        nonce: appchain_message_nonce,
                        message: Some(message),
                    },
                );
            }
            PromiseResult::Failed => {
                let reason = format!("Maybe the total supply will overflow.");
                let message = format!(
                    "Failed to mint wrapped appchain token for '{}' with amount '{}'. {}",
                    &receiver_id_in_near, &amount.0, &reason
                );
                self.internal_append_anchor_event(AnchorEvent::FailedToMintWrappedAppchainToken {
                    sender_id_in_appchain,
                    receiver_id_in_near,
                    amount: U128::from(amount),
                    appchain_message_nonce,
                    reason,
                });
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
