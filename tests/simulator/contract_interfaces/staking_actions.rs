use crate::common;
use near_sdk::{json_types::U128, serde_json::json, AccountId};
use std::collections::HashMap;
use workspaces::{error::Error, result::ExecutionFinalResult, Account, Contract};

pub async fn register_validator(
    signer: &Account,
    oct_token: &Contract,
    anchor: &Contract,
    account_id_in_appchain: &String,
    amount: u128,
    can_be_delegated_to: bool,
    profile: HashMap<String, String>,
) -> Result<ExecutionFinalResult, Error> {
    let result = common::call_ft_transfer_call(
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
    signer: &Account,
    oct_token: &Contract,
    anchor: &Contract,
    validator_id: &AccountId,
    amount: u128,
) -> Result<ExecutionFinalResult, Error> {
    let result = common::call_ft_transfer_call(
        signer,
        &anchor.as_account(),
        amount,
        json!({
            "RegisterDelegator": {
                "validator_id": validator_id,
                "delegator_id": null,
            }
        })
        .to_string(),
        oct_token,
    )
    .await;
    println!("Result of register delegator: {:?}", result);
    println!();
    result
}

pub async fn increase_stake(
    signer: &Account,
    oct_token: &Contract,
    anchor: &Contract,
    amount: u128,
) -> Result<ExecutionFinalResult, Error> {
    common::call_ft_transfer_call(
        signer,
        &anchor.as_account(),
        amount,
        json!({
            "IncreaseStake": {
                "validator_id": null,
            }
        })
        .to_string(),
        oct_token,
    )
    .await
}

pub async fn increase_delegation(
    signer: &Account,
    oct_token: &Contract,
    anchor: &Contract,
    validator_id: &AccountId,
    amount: u128,
) -> Result<ExecutionFinalResult, Error> {
    common::call_ft_transfer_call(
        signer,
        &anchor.as_account(),
        amount,
        json!({
            "IncreaseDelegation": {
                "validator_id": validator_id,
                "delegator_id": null,
            }
        })
        .to_string(),
        oct_token,
    )
    .await
}

pub async fn decrease_stake(
    signer: &Account,
    anchor: &Contract,
    amount: u128,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "decrease_stake")
        .args_json(json!({ "amount": U128::from(amount) }))
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn unbond_stake(
    signer: &Account,
    anchor: &Contract,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "unbond_stake")
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn decrease_delegation(
    signer: &Account,
    anchor: &Contract,
    validator_id: &AccountId,
    amount: u128,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "decrease_delegation")
        .args_json(json!({
            "validator_id": validator_id,
            "amount": U128::from(amount)
        }))
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn unbond_delegation(
    signer: &Account,
    anchor: &Contract,
    validator_id: &AccountId,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "unbond_delegation")
        .args_json(json!({ "validator_id": validator_id }))
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn withdraw_stake(
    signer: &Account,
    anchor: &Contract,
    account_id: &AccountId,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "withdraw_stake")
        .args_json(json!({ "account_id": account_id }))
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn withdraw_validator_rewards(
    signer: &Account,
    anchor: &Contract,
    validator_id: &AccountId,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "withdraw_validator_rewards")
        .args_json(json!({ "validator_id": validator_id }))
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn withdraw_delegator_rewards(
    signer: &Account,
    anchor: &Contract,
    delegator_id: &AccountId,
    validator_id: &AccountId,
) -> Result<ExecutionFinalResult, Error> {
    signer
        .call(anchor.id(), "withdraw_delegator_rewards")
        .args_json(json!({
            "delegator_id": delegator_id,
            "validator_id": validator_id
        }))
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn change_delegated_validator(
    signer: &Account,
    anchor: &Contract,
    old_validator_id: &AccountId,
    new_validator_id: &AccountId,
) -> Result<ExecutionFinalResult, Error> {
    let result = signer
        .call(anchor.id(), "change_delegated_validator")
        .args_json(json!({
            "old_validator_id": old_validator_id,
            "new_validator_id": new_validator_id
        }))
        .gas(200_000_000_000_000)
        .transact()
        .await;
    println!("Result of 'change_delegated_validator': {:?}", result);
    result
}
