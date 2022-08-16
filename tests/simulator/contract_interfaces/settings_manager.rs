use near_sdk::{
    json_types::{U128, U64},
    serde_json::json,
};
use workspaces::{network::Sandbox, result::CallExecutionDetails, Account, Contract, Worker};

pub async fn change_minimum_validator_count(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    value: u64,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "change_minimum_validator_count")
        .args_json(json!({ "value": U64::from(value) }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn set_rpc_endpoint(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    value: String,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "set_rpc_endpoint")
        .args_json(json!({ "rpc_endpoint": value }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn set_subql_endpoint(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    value: String,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "set_subql_endpoint")
        .args_json(json!({ "subql_endpoint": value }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn set_era_reward(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    value: u128,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "set_era_reward")
        .args_json(json!({ "era_reward": U128::from(value) }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn set_bonus_for_new_validator(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    value: u128,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "set_bonus_for_new_validator")
        .args_json(json!({ "bonus_amount": U128::from(value) }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn set_token_price_maintainer_account(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    account: &Account,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "set_token_price_maintainer_account")
        .args_json(json!({
            "account_id": account.id()
        }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn set_price_of_oct_token(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    value: u128,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "set_price_of_oct_token")
        .args_json(json!({ "price": U128::from(value) }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn change_unlock_period_of_validator_deposit(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    value: u64,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(
            worker,
            anchor.id(),
            "change_unlock_period_of_validator_deposit",
        )
        .args_json(json!({ "value": U64::from(value) }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn change_unlock_period_of_delegator_deposit(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    value: u64,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(
            worker,
            anchor.id(),
            "change_unlock_period_of_delegator_deposit",
        )
        .args_json(json!({ "value": U64::from(value) }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn turn_on_beefy_light_client_witness_mode(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(
            worker,
            anchor.id(),
            "turn_on_beefy_light_client_witness_mode",
        )
        .gas(200_000_000_000_000)
        .transact()
        .await
}
