use near_sdk::{json_types::U128, serde_json::json, AccountId};
use workspaces::{network::Sandbox, result::CallExecutionDetails, Account, Contract, Worker};

pub async fn register_near_fungible_token(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    symbol: String,
    name: String,
    decimals: u8,
    contract_account: AccountId,
    price: U128,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "register_near_fungible_token")
        .args_json(json!({
            "symbol": symbol,
            "name": name,
            "decimals": decimals,
            "contract_account": contract_account,
            "price": price
        }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}
