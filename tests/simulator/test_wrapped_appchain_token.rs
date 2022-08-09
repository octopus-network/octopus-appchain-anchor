use crate::{
    common,
    contract_interfaces::{sudo_actions, wrapped_appchain_token_manager},
};
use appchain_anchor::{AppchainEvent, AppchainMessage};
use near_sdk::{json_types::U128, AccountId};
use std::str::FromStr;

const TOTAL_SUPPLY: u128 = 100_000_000;

#[tokio::test]
async fn test_wrapped_appchain_token_bridging() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (
        root,
        _oct_token,
        wrapped_appchain_token,
        _registry,
        anchor,
        _wat_faucet,
        users,
        mut appchain_message_nonce,
    ) = common::test_normal_actions(&worker, false, true, vec!["0x00".to_string()]).await?;
    //
    let total_supply = common::to_actual_amount(TOTAL_SUPPLY, 18);
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    let user4_id_in_appchain =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da273".to_string();
    //
    // Mint wrapped appchain token for user1 (error)
    //
    let user1_wat_balance =
        common::get_ft_balance_of(&worker, &users[1], &wrapped_appchain_token).await?;
    appchain_message_nonce += 1;
    let appchain_message = AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: AccountId::from_str("unknown.testnet").unwrap(),
            amount: U128::from(total_supply / 10),
        },
        nonce: appchain_message_nonce,
    };
    sudo_actions::stage_appchain_message(&worker, &root, &anchor, appchain_message).await?;
    common::complex_actions::process_appchain_messages(&worker, &users[4], &anchor).await?;
    common::complex_viewer::print_appchain_messages(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_messages_processing_results(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    assert_eq!(
        common::get_ft_balance_of(&worker, &users[1], &wrapped_appchain_token)
            .await?
            .0,
        user1_wat_balance.0
    );
    //
    // Burn wrapped appchain token from user0
    //
    let result = wrapped_appchain_token_manager::burn_wrapped_appchain_token(
        &worker,
        &users[0],
        &anchor,
        user0_id_in_appchain,
        total_supply / 2 - common::to_actual_amount(50000, 18),
    )
    .await?;
    assert!(result.is_success());
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    //
    // Mint wrapped appchain token for user1
    //
    let user1_wat_balance =
        common::get_ft_balance_of(&worker, &users[1], &wrapped_appchain_token).await?;
    let mut appchain_messages = Vec::<AppchainMessage>::new();
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].id().to_string().parse().unwrap(),
            amount: U128::from(common::to_actual_amount(60, 18)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].id().to_string().parse().unwrap(),
            amount: U128::from(common::to_actual_amount(40, 18)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraSwitchPlaned { era_number: 1 },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].id().to_string().parse().unwrap(),
            amount: U128::from(common::to_actual_amount(70, 18)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].id().to_string().parse().unwrap(),
            amount: U128::from(common::to_actual_amount(30, 18)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].id().to_string().parse().unwrap(),
            amount: U128::from(common::to_actual_amount(45, 18)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].id().to_string().parse().unwrap(),
            amount: U128::from(common::to_actual_amount(45, 18)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].id().to_string().parse().unwrap(),
            amount: U128::from(common::to_actual_amount(45, 18)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].id().to_string().parse().unwrap(),
            amount: U128::from(common::to_actual_amount(45, 18)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraRewardConcluded {
            era_number: 0,
            unprofitable_validator_ids: Vec::new(),
            offenders: Vec::new(),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].id().to_string().parse().unwrap(),
            amount: U128::from(common::to_actual_amount(45, 18)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].id().to_string().parse().unwrap(),
            amount: U128::from(common::to_actual_amount(45, 18)),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::NativeTokenLocked {
            owner_id_in_appchain: user4_id_in_appchain.clone(),
            receiver_id_in_near: users[1].id().to_string().parse().unwrap(),
            amount: U128::from(common::to_actual_amount(45, 18)),
        },
        nonce: appchain_message_nonce,
    });
    for appchain_message in appchain_messages {
        sudo_actions::stage_appchain_message(&worker, &root, &anchor, appchain_message).await?;
    }
    common::complex_actions::process_appchain_messages(&worker, &users[3], &anchor).await?;
    common::complex_viewer::print_appchain_messages(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_messages_processing_results(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    assert_eq!(
        common::get_ft_balance_of(&worker, &users[1], &wrapped_appchain_token)
            .await?
            .0,
        user1_wat_balance.0 + common::to_actual_amount(515, 18)
    );
    //
    //
    //
    let mut appchain_messages = Vec::<AppchainMessage>::new();
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraSwitchPlaned { era_number: 2 },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraRewardConcluded {
            era_number: 1,
            unprofitable_validator_ids: Vec::new(),
            offenders: Vec::new(),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraSwitchPlaned { era_number: 3 },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraRewardConcluded {
            era_number: 2,
            unprofitable_validator_ids: Vec::new(),
            offenders: Vec::new(),
        },
        nonce: appchain_message_nonce,
    });
    appchain_message_nonce += 1;
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraRewardConcluded {
            era_number: 1,
            unprofitable_validator_ids: Vec::new(),
            offenders: Vec::new(),
        },
        nonce: appchain_message_nonce,
    });
    for appchain_message in appchain_messages {
        sudo_actions::stage_appchain_message(&worker, &root, &anchor, appchain_message).await?;
    }
    common::complex_actions::process_appchain_messages(&worker, &users[3], &anchor).await?;
    common::complex_viewer::print_appchain_messages(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_messages_processing_results(&worker, &anchor).await?;
    Ok(())
}
