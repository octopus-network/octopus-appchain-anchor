use crate::common;
use near_sdk::{json_types::U128, serde_json::json, AccountId};
use std::collections::HashMap;
use workspaces::{network::Sandbox, result::CallExecutionDetails, Account, Contract, Worker};

pub async fn register_validator(
    worker: &Worker<Sandbox>,
    signer: &Account,
    oct_token: &Contract,
    anchor: &Contract,
    account_id_in_appchain: &String,
    amount: u128,
    can_be_delegated_to: bool,
    profile: HashMap<String, String>,
) -> anyhow::Result<CallExecutionDetails> {
    let result = common::call_ft_transfer_call(
        worker,
        signer,
        &anchor.as_account(),
        amount,
        json!({
            "RegisterValidator": {
                "validator_id_in_appchain": account_id_in_appchain,
                "can_be_delegated_to": can_be_delegated_to,
                "profile": profile
            }
        })
        .to_string(),
        oct_token,
    )
    .await;
    println!("Result of 'register_validator': {:?}", result);
    println!();
    result
}

pub async fn register_delegator(
    worker: &Worker<Sandbox>,
    signer: &Account,
    oct_token: &Contract,
    anchor: &Contract,
    validator_id: &AccountId,
    amount: u128,
) -> anyhow::Result<CallExecutionDetails> {
    common::call_ft_transfer_call(
        worker,
        signer,
        &anchor.as_account(),
        amount,
        json!({
            "RegisterDelegator": {
                "validator_id": validator_id
            }
        })
        .to_string(),
        oct_token,
    )
    .await
}

pub async fn increase_stake(
    worker: &Worker<Sandbox>,
    signer: &Account,
    oct_token: &Contract,
    anchor: &Contract,
    amount: u128,
) -> anyhow::Result<CallExecutionDetails> {
    common::call_ft_transfer_call(
        worker,
        signer,
        &anchor.as_account(),
        amount,
        "\"IncreaseStake\"".to_string(),
        oct_token,
    )
    .await
}

pub async fn increase_delegation(
    worker: &Worker<Sandbox>,
    signer: &Account,
    oct_token: &Contract,
    anchor: &Contract,
    validator_id: &AccountId,
    amount: u128,
) -> anyhow::Result<CallExecutionDetails> {
    common::call_ft_transfer_call(
        worker,
        signer,
        &anchor.as_account(),
        amount,
        json!({
            "IncreaseDelegation": {
                "validator_id": validator_id
            }
        })
        .to_string(),
        oct_token,
    )
    .await
}

pub async fn decrease_stake(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    amount: u128,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "decrease_stake")
        .args_json(json!({ "amount": U128::from(amount) }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn unbond_stake(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "unbond_stake")
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn decrease_delegation(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    validator_id: &AccountId,
    amount: u128,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "decrease_delegation")
        .args_json(json!({
            "validator_id": validator_id,
            "amount": U128::from(amount)
        }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn unbond_delegation(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    validator_id: &AccountId,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "unbond_delegation")
        .args_json(json!({ "validator_id": validator_id }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn withdraw_stake(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    account_id: &AccountId,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "withdraw_stake")
        .args_json(json!({ "account_id": account_id }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn withdraw_validator_rewards(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    validator_id: &AccountId,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "withdraw_validator_rewards")
        .args_json(json!({ "validator_id": validator_id }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn withdraw_delegator_rewards(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    delegator_id: &AccountId,
    validator_id: &AccountId,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "withdraw_delegator_rewards")
        .args_json(json!({
            "delegator_id": delegator_id,
            "validator_id": validator_id
        }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}
