use near_sdk::{json_types::U128, serde_json::json, AccountId};
use workspaces::{network::Sandbox, result::CallExecutionDetails, Account, Contract, Worker};

pub async fn set_price_of_wrapped_appchain_token(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    price: u128,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "set_price_of_wrapped_appchain_token")
        .args_json(json!({ "price": U128::from(price) }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn set_account_of_wrapped_appchain_token(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    account_id: AccountId,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "set_account_of_wrapped_appchain_token")
        .args_json(json!({ "contract_account": account_id }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn burn_wrapped_appchain_token(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    receiver_id: String,
    amount: u128,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "burn_wrapped_appchain_token")
        .args_json(json!({
            "receiver_id": receiver_id,
            "amount": U128::from(amount)
        }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}
