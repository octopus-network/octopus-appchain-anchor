use near_sdk::{json_types::U128, serde_json::json, AccountId};
use workspaces::{error::Error, result::ExecutionFinalResult, Account, Contract};

pub async fn set_price_of_wrapped_appchain_token(
    signer: &Account,
    anchor: &Contract,
    price: u128,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "set_price_of_wrapped_appchain_token")
        .args_json(json!({ "price": U128::from(price) }))
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn set_account_of_wrapped_appchain_token(
    signer: &Account,
    anchor: &Contract,
    account_id: AccountId,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "set_account_of_wrapped_appchain_token")
        .args_json(json!({ "contract_account": account_id }))
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn burn_wrapped_appchain_token(
    signer: &Account,
    anchor: &Contract,
    receiver_id: String,
    amount: u128,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "burn_wrapped_appchain_token")
        .args_json(json!({
            "receiver_id": receiver_id,
            "amount": U128::from(amount)
        }))
        .gas(200_000_000_000_000)
        .transact()
        .await
}
