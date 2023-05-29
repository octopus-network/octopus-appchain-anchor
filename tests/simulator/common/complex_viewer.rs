use crate::{common::get_ft_balance_of, contract_interfaces::anchor_viewer};
use appchain_anchor::types::{
    AnchorSettings, AnchorStatus, AppchainSettings, BeefyLightClientStatus, ValidatorProfile,
    ValidatorSetInfo, WrappedAppchainToken,
};
use near_sdk::{json_types::U64, serde_json, AccountId};
use workspaces::{Account, Contract};

pub async fn print_anchor_status(anchor: &Contract) {
    let anchor_status = anchor_viewer::get_anchor_status(anchor).await.unwrap();
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    println!();
}

pub async fn print_appchain_settings(anchor: &Contract) {
    let appchain_settings = anchor_viewer::get_appchain_settings(anchor).await.unwrap();
    println!(
        "Appchain settings: {}",
        serde_json::to_string::<AppchainSettings>(&appchain_settings).unwrap()
    );
    println!();
}

pub async fn print_anchor_settings(anchor: &Contract) {
    let anchor_settings = anchor_viewer::get_anchor_settings(anchor).await.unwrap();
    println!(
        "Anchor settings: {}",
        serde_json::to_string::<AnchorSettings>(&anchor_settings).unwrap()
    );
    println!();
}

pub async fn print_validator_set_info_of(anchor: &Contract, era_number: U64) {
    let validator_set_info = anchor_viewer::get_validator_set_info_of(anchor, era_number)
        .await
        .unwrap();
    println!(
        "Validator set {} info: {}",
        era_number.0,
        serde_json::to_string::<ValidatorSetInfo>(&validator_set_info).unwrap()
    );
    println!();
}

pub async fn print_wrapped_appchain_token_info(anchor: &Contract) {
    let wrapped_appchain_token_info = anchor_viewer::get_wrapped_appchain_token(&anchor)
        .await
        .unwrap();
    println!(
        "Wrapped appchain token: {}",
        serde_json::to_string::<WrappedAppchainToken>(&wrapped_appchain_token_info).unwrap()
    );
    println!();
}

pub async fn print_near_fungible_tokens(anchor: &Contract) {
    let near_fungible_tokens = anchor_viewer::get_near_fungible_tokens(&anchor)
        .await
        .unwrap();
    near_fungible_tokens.iter().for_each(|record| {
        println!(
            "Near fungible token: {}",
            serde_json::to_string(&record).unwrap()
        );
        println!();
    });
}

pub async fn print_native_near_token(anchor: &Contract) {
    let native_near_token = anchor_viewer::get_native_near_token(&anchor).await.unwrap();
    println!(
        "Native NEAR token: {}",
        serde_json::to_string(&native_near_token).unwrap()
    );
}

pub async fn print_validator_profile(
    anchor: &Contract,
    account_id: &AccountId,
    account_id_in_appchain: &String,
) {
    let result = anchor_viewer::get_validator_profile(&anchor, &account_id).await;
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
    let result =
        anchor_viewer::get_validator_profile_by_id_in_appchain(&anchor, &account_id_in_appchain)
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
}

pub async fn print_appchain_notifications(anchor: &Contract) {
    let index_range = anchor_viewer::get_index_range_of_appchain_notification_history(anchor)
        .await
        .unwrap();
    for i in index_range.start_index.0..index_range.end_index.0 + 1 {
        if let Some(appchain_notification_history) =
            anchor_viewer::get_appchain_notification_history(anchor, i.try_into().unwrap())
                .await
                .unwrap()
        {
            println!(
                "Appchain notification history {}: {}",
                i,
                serde_json::to_string(&appchain_notification_history).unwrap()
            );
            println!();
        }
    }
    let records = anchor_viewer::get_appchain_notification_histories(anchor, 0, None)
        .await
        .unwrap();
    records.iter().for_each(|record| {
        println!(
            "Appchain notification history {}: {}",
            record.index.0,
            serde_json::to_string(&record).unwrap()
        );
        println!();
    });
}

pub async fn print_staking_histories(anchor: &Contract) {
    let index_range = anchor_viewer::get_index_range_of_staking_history(anchor)
        .await
        .unwrap();
    for i in index_range.start_index.0..index_range.end_index.0 + 1 {
        if let Some(staking_history) =
            anchor_viewer::get_staking_history(anchor, i.try_into().unwrap())
                .await
                .unwrap()
        {
            println!(
                "Staking history {}: {}",
                i,
                serde_json::to_string(&staking_history).unwrap()
            );
            println!();
        }
    }
}

pub async fn print_user_staking_histories_of(anchor: &Contract, user: &Account) {
    let staking_histories = anchor_viewer::get_user_staking_histories_of(
        anchor,
        user.id().to_string().parse().unwrap(),
    )
    .await
    .unwrap();
    let mut index = 0;
    for staking_history in staking_histories {
        println!(
            "Staking history {} of account {}: {}",
            index,
            &user.id(),
            serde_json::to_string(&staking_history).unwrap()
        );
        println!();
        index += 1;
    }
}

pub async fn print_validator_list_of(anchor: &Contract, era_number: Option<u64>) {
    let validator_list = anchor_viewer::get_validator_list_of(anchor, era_number)
        .await
        .unwrap();
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
        println!();
        index += 1;
    }
}

pub async fn print_delegator_list_of(anchor: &Contract, era_number: u64, validator: &Account) {
    let delegator_list =
        anchor_viewer::get_delegators_of_validator_in_era(&anchor, era_number, validator)
            .await
            .unwrap();
    let mut index = 0;
    for delegator in delegator_list {
        println!(
            "Delegator {} of {} in era {}: {}",
            index,
            validator.id(),
            era_number,
            serde_json::to_string(&delegator).unwrap()
        );
        println!();
        index += 1;
    }
}

pub async fn print_validator_reward_histories(
    anchor: &Contract,
    validator: &Account,
    end_era: u64,
) {
    let reward_histories = anchor_viewer::get_validator_rewards_of(anchor, 0, end_era, validator)
        .await
        .unwrap();
    let mut index = 0;
    for reward_history in reward_histories {
        println!(
            "Reward history {} of {}: {}",
            index,
            validator.id().to_string(),
            serde_json::to_string(&reward_history).unwrap()
        );
        println!();
        index += 1;
    }
}

pub async fn print_delegator_reward_histories(
    anchor: &Contract,
    delegator: &Account,
    validator: &Account,
    end_era: u64,
) {
    let reward_histories =
        anchor_viewer::get_delegator_rewards_of(anchor, 0, end_era, delegator, validator)
            .await
            .unwrap();
    let mut index = 0;
    for reward_history in reward_histories {
        println!(
            "Reward history {} of {} to {}: {}",
            index,
            delegator.id().to_string(),
            validator.id().to_string(),
            serde_json::to_string(&reward_history).unwrap()
        );
        println!();
        index += 1;
    }
}

pub async fn print_unbonded_stakes_of(anchor: &Contract, user: &Account) {
    let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(anchor, user)
        .await
        .unwrap();
    let mut index = 0;
    for unbonded_stake in unbonded_stakes {
        println!(
            "Unbonded stake {} of {}: {}",
            index,
            user.id(),
            serde_json::to_string(&unbonded_stake).unwrap()
        );
        println!();
        index += 1;
    }
}

pub async fn print_beefy_light_client_status(anchor: &Contract) {
    let status = anchor_viewer::get_beefy_light_client_status(&anchor)
        .await
        .unwrap();
    println!(
        "Beefy light client status: {}",
        serde_json::to_string::<BeefyLightClientStatus>(&status).unwrap()
    );
    println!();
}

pub async fn print_wat_balance_of_anchor(anchor: &Contract, wrapped_appchain_token: &Contract) {
    let wat_balance_of_anchor = get_ft_balance_of(&anchor.as_account(), wrapped_appchain_token)
        .await
        .unwrap();
    println!(
        "Wrapped appchain token balance of anchor contract: {}",
        wat_balance_of_anchor.0
    );
    println!();
}

pub async fn print_appchain_messages(anchor: &Contract) {
    let appchain_messages = anchor_viewer::get_appchain_messages(anchor, 0, None)
        .await
        .unwrap();
    for appchain_message in appchain_messages {
        println!(
            "Appchain message '{}': {}",
            appchain_message.nonce,
            serde_json::to_string(&appchain_message).unwrap()
        );
        println!();
    }
}

pub async fn print_appchain_messages_processing_results(anchor: &Contract) {
    let appchain_messages = anchor_viewer::get_appchain_message_processing_results(anchor, 0, None)
        .await
        .unwrap();
    let mut index = 1;
    for appchain_message in appchain_messages {
        println!(
            "Appchain message processing result '{}': {}",
            index,
            serde_json::to_string(&appchain_message).unwrap()
        );
        println!();
        index += 1;
    }
}
