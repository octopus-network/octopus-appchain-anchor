use crate::{common::get_ft_balance_of, contract_interfaces::anchor_viewer};
use appchain_anchor::types::{
    AnchorSettings, AnchorStatus, AppchainCommitment, AppchainSettings, ValidatorProfile,
    ValidatorSetInfo, WrappedAppchainToken,
};
use near_sdk::{json_types::U64, serde_json, AccountId};
use workspaces::network::Sandbox;
use workspaces::{Account, Contract, Worker};

pub async fn print_anchor_status(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<()> {
    let anchor_status = anchor_viewer::get_anchor_status(worker, anchor).await?;
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    println!();
    Ok(())
}

pub async fn print_appchain_settings(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<()> {
    let appchain_settings = anchor_viewer::get_appchain_settings(worker, anchor).await?;
    println!(
        "Appchain settings: {}",
        serde_json::to_string::<AppchainSettings>(&appchain_settings).unwrap()
    );
    println!();
    Ok(())
}

pub async fn print_anchor_settings(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<()> {
    let anchor_settings = anchor_viewer::get_anchor_settings(worker, anchor).await?;
    println!(
        "Anchor settings: {}",
        serde_json::to_string::<AnchorSettings>(&anchor_settings).unwrap()
    );
    println!();
    Ok(())
}

pub async fn print_validator_set_info_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    era_number: U64,
) -> anyhow::Result<()> {
    let validator_set_info =
        anchor_viewer::get_validator_set_info_of(worker, anchor, era_number).await?;
    println!(
        "Validator set {} info: {}",
        era_number.0,
        serde_json::to_string::<ValidatorSetInfo>(&validator_set_info).unwrap()
    );
    println!();
    Ok(())
}

pub async fn print_wrapped_appchain_token_info(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<()> {
    let wrapped_appchain_token_info =
        anchor_viewer::get_wrapped_appchain_token(worker, &anchor).await?;
    println!(
        "Wrapped appchain token: {}",
        serde_json::to_string::<WrappedAppchainToken>(&wrapped_appchain_token_info).unwrap()
    );
    println!();
    Ok(())
}

pub async fn print_near_fungible_tokens(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<()> {
    let near_fungible_tokens = anchor_viewer::get_near_fungible_tokens(worker, &anchor).await?;
    near_fungible_tokens.iter().for_each(|record| {
        println!(
            "Near fungible token: {}",
            serde_json::to_string(&record).unwrap()
        );
        println!();
    });
    Ok(())
}

pub async fn print_validator_profile(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    account_id: &AccountId,
    account_id_in_appchain: &String,
) -> anyhow::Result<()> {
    let result = anchor_viewer::get_validator_profile(worker, &anchor, &account_id).await;
    result
        .as_ref()
        .expect("Failed calling 'get_validator_profile'");
    let validator_profile = result.unwrap();
    println!(
        "Profile of '{}': {}",
        &account_id,
        serde_json::to_string::<ValidatorProfile>(&validator_profile.unwrap()).unwrap()
    );
    println!();
    let result = anchor_viewer::get_validator_profile_by_id_in_appchain(
        worker,
        &anchor,
        &account_id_in_appchain,
    )
    .await;
    result
        .as_ref()
        .expect("Failed calling 'get_validator_profile_by_id_in_appchain'");
    let validator_profile = result.unwrap();
    if validator_profile.is_some() {
        println!(
            "Profile of '{}': {}",
            &account_id_in_appchain,
            serde_json::to_string::<ValidatorProfile>(&validator_profile.unwrap()).unwrap()
        );
        println!();
    }
    Ok(())
}

pub async fn print_appchain_notifications(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<()> {
    let index_range =
        anchor_viewer::get_index_range_of_appchain_notification_history(worker, anchor).await?;
    for i in index_range.start_index.0..index_range.end_index.0 + 1 {
        if let Some(appchain_notification_history) =
            anchor_viewer::get_appchain_notification_history(worker, anchor, i.try_into().unwrap())
                .await?
        {
            println!(
                "Appchain notification history {}: {}",
                i,
                serde_json::to_string(&appchain_notification_history).unwrap()
            );
            println!();
        }
    }
    let records =
        anchor_viewer::get_appchain_notification_histories(worker, anchor, 0, None).await?;
    records.iter().for_each(|record| {
        println!(
            "Appchain notification history {}: {}",
            record.index.0,
            serde_json::to_string(&record).unwrap()
        );
        println!();
    });
    Ok(())
}

pub async fn print_staking_histories(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<()> {
    let index_range = anchor_viewer::get_index_range_of_staking_history(worker, anchor).await?;
    for i in index_range.start_index.0..index_range.end_index.0 + 1 {
        if let Some(staking_history) =
            anchor_viewer::get_staking_history(worker, anchor, i.try_into().unwrap()).await?
        {
            println!(
                "Staking history {}: {}",
                i,
                serde_json::to_string(&staking_history).unwrap()
            );
            println!();
        }
    }
    Ok(())
}

pub async fn print_user_staking_histories_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    user: &Account,
) -> anyhow::Result<()> {
    let staking_histories = anchor_viewer::get_user_staking_histories_of(
        worker,
        anchor,
        user.id().to_string().parse().unwrap(),
    )
    .await?;
    let mut index = 0;
    for staking_history in staking_histories {
        println!(
            "Staking history {} of account {}: {}",
            index,
            &user.id(),
            serde_json::to_string(&staking_history).unwrap()
        );
        index += 1;
        println!();
    }
    Ok(())
}

pub async fn print_validator_list_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    era_number: Option<u64>,
) -> anyhow::Result<()> {
    let validator_list = anchor_viewer::get_validator_list_of(worker, anchor, era_number).await?;
    let mut index = 0;
    for validator in validator_list {
        if let Some(era_number) = era_number {
            println!(
                "Validator {} in era {}: {}",
                index,
                era_number,
                serde_json::to_string(&validator).unwrap()
            );
        } else {
            println!(
                "Validator {} in next era: {}",
                index,
                serde_json::to_string(&validator).unwrap()
            );
        }
        index += 1;
        println!();
    }
    Ok(())
}

pub async fn print_delegator_list_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    era_number: u64,
    validator: &Account,
) -> anyhow::Result<()> {
    let delegator_list =
        anchor_viewer::get_delegators_of_validator_in_era(worker, &anchor, era_number, validator)
            .await?;
    let mut index = 0;
    for delegator in delegator_list {
        println!(
            "Delegator {} of {} in era {}: {}",
            index,
            validator.id(),
            era_number,
            serde_json::to_string(&delegator).unwrap()
        );
        index += 1;
        println!();
    }
    Ok(())
}

pub async fn print_validator_reward_histories(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    validator: &Account,
    end_era: u64,
) -> anyhow::Result<()> {
    let reward_histories =
        anchor_viewer::get_validator_rewards_of(worker, anchor, 0, end_era, validator).await?;
    let mut index = 0;
    for reward_history in reward_histories {
        println!(
            "Reward history {} of {}: {}",
            index,
            validator.id().to_string(),
            serde_json::to_string(&reward_history).unwrap()
        );
        index += 1;
        println!();
    }
    Ok(())
}

pub async fn print_delegator_reward_histories(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    delegator: &Account,
    validator: &Account,
    end_era: u64,
) -> anyhow::Result<()> {
    let reward_histories =
        anchor_viewer::get_delegator_rewards_of(worker, anchor, 0, end_era, delegator, validator)
            .await?;
    let mut index = 0;
    for reward_history in reward_histories {
        println!(
            "Reward history {} of {} to {}: {}",
            index,
            delegator.id().to_string(),
            validator.id().to_string(),
            serde_json::to_string(&reward_history).unwrap()
        );
        index += 1;
        println!();
    }
    Ok(())
}

pub async fn print_unbonded_stakes_of(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    user: &Account,
) -> anyhow::Result<()> {
    let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(worker, anchor, user).await?;
    let mut index = 0;
    for unbonded_stake in unbonded_stakes {
        println!(
            "Unbonded stake {} of {}: {}",
            index,
            user.id(),
            serde_json::to_string(&unbonded_stake).unwrap()
        );
        index += 1;
        println!();
    }
    Ok(())
}

pub async fn print_latest_appchain_commitment(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<()> {
    let appchain_commitment =
        anchor_viewer::get_latest_commitment_of_appchain(worker, &anchor).await?;
    println!(
        "Latest appchain commitment: {}",
        serde_json::to_string::<Option<AppchainCommitment>>(&appchain_commitment).unwrap()
    );
    println!();
    Ok(())
}

pub async fn print_wat_balance_of_anchor(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    wrapped_appchain_token: &Contract,
) -> anyhow::Result<()> {
    let wat_balance_of_anchor =
        get_ft_balance_of(worker, &anchor.as_account(), wrapped_appchain_token).await?;
    println!(
        "Wrapped appchain token balance of anchor contract: {}",
        wat_balance_of_anchor.0
    );
    println!();
    Ok(())
}

pub async fn print_appchain_messages(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<()> {
    let appchain_messages = anchor_viewer::get_appchain_messages(worker, anchor, 0, None).await?;
    for appchain_message in appchain_messages {
        println!(
            "Appchain message '{}': {}",
            appchain_message.nonce,
            serde_json::to_string(&appchain_message).unwrap()
        );
        println!();
    }
    Ok(())
}

pub async fn print_appchain_messages_processing_results(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
) -> anyhow::Result<()> {
    let appchain_messages =
        anchor_viewer::get_appchain_message_processing_results(worker, anchor, 0, None).await?;
    let mut index = 1;
    for appchain_message in appchain_messages {
        println!(
            "Appchain message processing result '{}': {}",
            index,
            serde_json::to_string(&appchain_message).unwrap()
        );
        index += 1;
        println!();
    }
    Ok(())
}
