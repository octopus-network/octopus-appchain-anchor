use crate::common::{self, complex_actions};
use near_sdk::{
    serde_json::{self, json},
    AccountId,
};
use octopus_council::types::{CouncilChangeHistory, ValidatorStake};

#[tokio::test]
async fn test_sync_staking_amount() -> anyhow::Result<()> {
    //
    let worker = workspaces::sandbox().await?;
    let (
        _root,
        _,
        wrapped_appchain_token,
        _registry,
        council,
        anchor,
        _wat_faucet,
        users,
        mut appchain_message_nonce,
    ) = common::test_normal_actions(&worker, false, false, vec!["0x00".to_string()]).await?;
    //
    // Try start and complete switching era1
    //
    appchain_message_nonce += 1;
    complex_actions::switch_era(
        &worker,
        &users[5],
        &anchor,
        1,
        appchain_message_nonce,
        false,
    )
    .await
    .expect("Failed to switch era 1.");
    //
    // Distribut reward of era0
    //
    appchain_message_nonce += 1;
    complex_actions::distribute_reward_of(
        &worker,
        &users[5],
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        0,
        Vec::new(),
        false,
    )
    .await
    .expect("Failed to distribute reward of era 0.");
    //
    // Try start and complete switching era2
    //
    appchain_message_nonce += 1;
    complex_actions::switch_era(
        &worker,
        &users[5],
        &anchor,
        2,
        appchain_message_nonce,
        false,
    )
    .await
    .expect("Failed to switch era 2.");
    //
    //
    //
    let result = council
        .call(&worker, "get_living_appchain_ids")
        .view()
        .await?
        .json::<Vec<String>>()
        .unwrap();
    println!(
        "Living appchain ids: {}",
        serde_json::to_string::<Vec<String>>(&result).unwrap()
    );
    //
    let result = council
        .call(&worker, "get_council_members")
        .view()
        .await?
        .json::<Vec<AccountId>>()
        .unwrap();
    println!(
        "Result of 'get_council_members': {:?}",
        serde_json::to_string::<Vec<AccountId>>(&result).unwrap()
    );
    //
    let result = council
        .call(&worker, "get_ranked_validator_stakes")
        .args_json(json!( {
            "start_index": 0,
            "quantity": null,
        }))?
        .view()
        .await?
        .json::<Vec<ValidatorStake>>()
        .unwrap();
    println!(
        "Result of 'get_ranked_validator_stakes': {:?}",
        serde_json::to_string::<Vec<ValidatorStake>>(&result).unwrap()
    );
    //
    let result = council
        .call(&worker, "get_council_change_histories")
        .args_json(json!( {
            "start_index": "0",
            "quantity": null,
        }))?
        .view()
        .await?
        .json::<Vec<CouncilChangeHistory>>()
        .unwrap();
    println!(
        "Result of 'get_council_change_histories': {:?}",
        serde_json::to_string::<Vec<CouncilChangeHistory>>(&result).unwrap()
    );
    //
    Ok(())
}
