use appchain_anchor::AppchainAnchorContract;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn set_metadata_of_wrapped_appchain_token(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    symbol: String,
    name: String,
    decimals: u8,
    spec: String,
    icon: Option<String>,
    reference: Option<String>,
    reference_hash: Option<Base64VecU8>,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.set_metadata_of_wrapped_appchain_token(FungibleTokenMetadata {
            symbol,
            name,
            decimals,
            spec,
            icon,
            reference,
            reference_hash
        })
    );
    common::print_execution_result("set_metadata_of_wrapped_appchain_token", &result);
    result
}

pub fn set_premined_balance_of_wrapped_appchain_token(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    premined_beneficiary: String,
    premined_balance: u128,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.set_premined_balance_of_wrapped_appchain_token(
            premined_beneficiary,
            U128::from(premined_balance)
        )
    );
    common::print_execution_result("set_premined_balance_of_wrapped_appchain_token", &result);
    result
}

pub fn set_price_of_wrapped_appchain_token(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    price: u128,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.set_price_of_wrapped_appchain_token(U128::from(price))
    );
    common::print_execution_result("set_price_of_wrapped_appchain_token", &result);
    result
}

pub fn set_account_of_wrapped_appchain_token(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    wat_account: String,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.set_account_of_wrapped_appchain_token(wat_account)
    );
    common::print_execution_result("apply_appchain_message", &result);
    result
}

pub fn burn_wrapped_appchain_token(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    receiver_id: String,
    amount: u128,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.burn_wrapped_appchain_token(receiver_id, U128::from(amount))
    );
    common::print_execution_result("burn_wrapped_appchain_token", &result);
    result
}
