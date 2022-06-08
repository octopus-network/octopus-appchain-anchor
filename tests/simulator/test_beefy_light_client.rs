use crate::{
    common,
    contract_interfaces::{
        anchor_viewer, permissionless_actions, settings_manager, staking_actions, sudo_actions,
    },
};
use appchain_anchor::types::{MultiTxsOperationProcessingResult, ValidatorMerkleProof};
use beefy_light_client::mmr::{MmrLeaf, MmrLeafProof};
use beefy_light_client::{beefy_ecdsa_to_ethereum, commitment::SignedCommitment};
use codec::Decode;
use hex_literal::hex;
use near_sdk::{json_types::U64, serde_json};
use workspaces::{network::Sandbox, Account, Contract, Worker};

#[tokio::test]
async fn test_beefy_light_client() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let initial_public_keys = vec![
        "0x020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1".to_string(), // Alice
        "0x0390084fdbf27d2b79d26a4f13f0ccd982cb755a661969143c37cbc49ef5b91f27".to_string(), // Bob
        "0x0389411795514af1627765eceffcbd002719f031604fadd7d188e2dc585b4e1afb".to_string(), // Charlie
        "0x03bc9d0ca094bd5b8b3225d7651eac5d18c1c04bf8ae8f8b263eebca4e1410ed0c".to_string(), // Dave
        "0x031d10105e323c4afce225208f71a6441ee327a65b9e646e772500c74d31f669aa".to_string(), // Eve
    ];
    let (
        root,
        oct_token,
        wrapped_appchain_token,
        _registry,
        anchor,
        users,
        mut appchain_message_nonce,
    ) = common::test_normal_actions(&worker, false, true, initial_public_keys).await?;
    //
    // Update state of beefy light client
    //
    update_state_of_beefy_light_client_1(&worker, &anchor, &users[4]).await?;
    common::complex_viewer::print_latest_appchain_commitment(&worker, &anchor).await?;
    update_state_of_beefy_light_client_2(&worker, &anchor, &users[1]).await?;
    common::complex_viewer::print_latest_appchain_commitment(&worker, &anchor).await?;
    //
    // Try start and complete switching era1
    //
    appchain_message_nonce += 1;
    common::complex_actions::switch_era(&worker, &root, &anchor, 1, appchain_message_nonce, true)
        .await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(1)).await?;
    common::complex_viewer::print_delegator_list_of(&worker, &anchor, 1, &users[0]).await?;
    //
    // Distribut reward of era0
    //
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &root,
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        0,
        Vec::new(),
        true,
    )
    .await?;
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[0], 0)
        .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[1], 0)
        .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[2], &users[0], 0,
    )
    .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[3], &users[0], 0,
    )
    .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[4], 0)
        .await?;
    //
    // user1 decrease stake
    //
    let result =
        staking_actions::decrease_stake(&worker, &users[1], &anchor, common::to_oct_amount(1000))
            .await?;
    assert!(result.is_success());
    common::complex_viewer::print_anchor_status(&worker, &anchor).await?;
    let unbonded_stakes =
        anchor_viewer::get_unbonded_stakes_of(&worker, &anchor, &users[1]).await?;
    assert!(unbonded_stakes.len() == 0);
    //
    // user2 decrease delegation
    //
    let result = staking_actions::decrease_delegation(
        &worker,
        &users[2],
        &anchor,
        &users[0].id().to_string().parse().unwrap(),
        common::to_oct_amount(200),
    )
    .await?;
    assert!(result.is_success());
    common::complex_viewer::print_anchor_status(&worker, &anchor).await?;
    let unbonded_stakes =
        anchor_viewer::get_unbonded_stakes_of(&worker, &anchor, &users[2]).await?;
    assert!(unbonded_stakes.len() == 0);
    //
    // Print staking histories
    //
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    //
    // Try start and complete switching era2
    //
    appchain_message_nonce += 1;
    common::complex_actions::switch_era(&worker, &root, &anchor, 2, appchain_message_nonce, true)
        .await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(2)).await?;
    common::complex_viewer::print_delegator_list_of(&worker, &anchor, 2, &users[0]).await?;
    //
    // Distribute reward of era1
    //
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &root,
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        1,
        Vec::new(),
        true,
    )
    .await?;
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[0], 1)
        .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[1], 1)
        .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[2], &users[0], 1,
    )
    .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[3], &users[0], 1,
    )
    .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[4], 1)
        .await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[0]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[1]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[2]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[3]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[4]).await?;
    //
    // Change unlock period for testing
    //
    let result =
        settings_manager::change_unlock_period_of_validator_deposit(&worker, &root, &anchor, 3)
            .await?;
    assert!(result.is_success());
    let result =
        settings_manager::change_unlock_period_of_delegator_deposit(&worker, &root, &anchor, 1)
            .await?;
    assert!(result.is_success());
    //
    // user3 unbond delegation
    //
    let result = staking_actions::unbond_delegation(
        &worker,
        &users[2],
        &anchor,
        &users[0].id().to_string().parse().unwrap(),
    )
    .await?;
    assert!(result.is_success());
    common::complex_viewer::print_anchor_status(&worker, &anchor).await?;
    let unbonded_stakes =
        anchor_viewer::get_unbonded_stakes_of(&worker, &anchor, &users[2]).await?;
    assert!(unbonded_stakes.len() == 1);
    //
    // Print staking histories
    //
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    //
    // Try start and complete switching era3
    //
    appchain_message_nonce += 1;
    common::complex_actions::switch_era(&worker, &root, &anchor, 3, appchain_message_nonce, true)
        .await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(3)).await?;
    common::complex_viewer::print_delegator_list_of(&worker, &anchor, 3, &users[0]).await?;
    //
    // Distribute reward of era2
    //
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &root,
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        2,
        Vec::new(),
        true,
    )
    .await?;
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[0], 2)
        .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[1], 2)
        .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[2], &users[0], 2,
    )
    .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[3], &users[0], 2,
    )
    .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[4], 2)
        .await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[0]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[1]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[2]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[3]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[4]).await?;
    //
    // user0 unbond stake
    //
    let result = staking_actions::unbond_stake(&worker, &users[0], &anchor).await?;
    assert!(result.is_success());
    common::complex_viewer::print_anchor_status(&worker, &anchor).await?;
    let unbonded_stakes =
        anchor_viewer::get_unbonded_stakes_of(&worker, &anchor, &users[0]).await?;
    assert!(unbonded_stakes.len() == 0);
    //
    // Print staking histories
    //
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    //
    // Try start and complete switching era3
    //
    appchain_message_nonce += 1;
    common::complex_actions::switch_era(&worker, &root, &anchor, 4, appchain_message_nonce, true)
        .await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(4)).await?;
    common::complex_viewer::print_delegator_list_of(&worker, &anchor, 4, &users[0]).await?;
    //
    // Distribute reward of era3
    //
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &root,
        &anchor,
        &wrapped_appchain_token,
        appchain_message_nonce,
        3,
        Vec::new(),
        true,
    )
    .await?;
    common::complex_viewer::print_wrapped_appchain_token_info(&worker, &anchor).await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[0], 3)
        .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[1], 3)
        .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[2], &users[0], 3,
    )
    .await?;
    common::complex_viewer::print_delegator_reward_histories(
        &worker, &anchor, &users[3], &users[0], 3,
    )
    .await?;
    common::complex_viewer::print_validator_reward_histories(&worker, &anchor, &users[4], 3)
        .await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[0]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[1]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[2]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[3]).await?;
    common::complex_viewer::print_unbonded_stakes_of(&worker, &anchor, &users[4]).await?;
    //
    // Withdraw validator rewards
    //
    common::complex_actions::withdraw_validator_rewards_of(
        &worker,
        &anchor,
        &users[0],
        &wrapped_appchain_token,
        3,
    )
    .await?;
    common::complex_actions::withdraw_validator_rewards_of(
        &worker,
        &anchor,
        &users[1],
        &wrapped_appchain_token,
        3,
    )
    .await?;
    common::complex_actions::withdraw_validator_rewards_of(
        &worker,
        &anchor,
        &users[4],
        &wrapped_appchain_token,
        3,
    )
    .await?;
    //
    // Withdraw delegator rewards
    //
    common::complex_actions::withdraw_delegator_rewards_of(
        &worker,
        &anchor,
        &users[2],
        &users[0],
        &wrapped_appchain_token,
        3,
    )
    .await?;
    common::complex_actions::withdraw_delegator_rewards_of(
        &worker,
        &anchor,
        &users[3],
        &users[0],
        &wrapped_appchain_token,
        3,
    )
    .await?;
    //
    // Withdraw stake
    //
    common::complex_actions::withdraw_stake_of(&worker, &anchor, &users[0], &oct_token).await?;
    common::complex_actions::withdraw_stake_of(&worker, &anchor, &users[1], &oct_token).await?;
    common::complex_actions::withdraw_stake_of(&worker, &anchor, &users[2], &oct_token).await?;
    common::complex_actions::withdraw_stake_of(&worker, &anchor, &users[3], &oct_token).await?;
    common::complex_actions::withdraw_stake_of(&worker, &anchor, &users[4], &oct_token).await?;
    //
    // Reset history data
    //
    let result =
        sudo_actions::reset_validator_set_histories_to(&worker, &root, &anchor, U64::from(0))
            .await?;
    assert!(result.is_success());
    let result = sudo_actions::clear_anchor_event_histories(&worker, &root, &anchor).await?;
    assert!(result.is_success());
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(0)).await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(1)).await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(2)).await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(3)).await?;
    common::complex_viewer::print_staking_histories(&worker, &anchor).await?;
    common::complex_viewer::print_appchain_notifications(&worker, &anchor).await?;
    Ok(())
}

async fn update_state_of_beefy_light_client_1(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    user: &Account,
) -> anyhow::Result<()> {
    let alice_pk = beefy_ecdsa_to_ethereum(
        &hex!("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1")[..],
    );
    let bob_pk = beefy_ecdsa_to_ethereum(
        &hex!("0390084fdbf27d2b79d26a4f13f0ccd982cb755a661969143c37cbc49ef5b91f27")[..],
    );
    let charlie_pk = beefy_ecdsa_to_ethereum(
        &hex!("0389411795514af1627765eceffcbd002719f031604fadd7d188e2dc585b4e1afb")[..],
    );
    let dave_pk = beefy_ecdsa_to_ethereum(
        &hex!("03bc9d0ca094bd5b8b3225d7651eac5d18c1c04bf8ae8f8b263eebca4e1410ed0c")[..],
    );
    let eve_pk = beefy_ecdsa_to_ethereum(
        &hex!("031d10105e323c4afce225208f71a6441ee327a65b9e646e772500c74d31f669aa")[..],
    );

    let encoded_signed_commitment_1 = hex!("f45927644a0b5bc6f1ce667330071fbaea498403c084eb0d4cb747114887345d0900000000000000000000001401b9b5b39fb15d7e22710ad06075cf0e20c4b0c1e3d0a6482946e1d0daf86ca2e37b40209316f00a549cdd2a7fd191694fee4f76f698d0525642563e665db85d6300010ee39cb2cb008f7dce753541b5442e98a260250286b335d6048f2dd4695237655ccc93ebcd3d7c04461e0b9d12b81b21a826c5ee3eebcd6ab9e85c8717f6b1ae010001b094279e0bb4442ba07165da47ab9c0d7d0f479e31d42c879564915714e8ea3d42393dc430addc4a5f416316c02e0676e525c56a3d0c0033224ebda4c83052670001f965d806a16c5dfb9d119f78cdbed379bccb071528679306208880ad29a9cf9e00e75f1b284fa3457b7b37223a2272cf2bf90ce4fd7e84e321eddec3cdeb66f801");
    let signed_commitment_1 = SignedCommitment::decode(&mut &encoded_signed_commitment_1[..]);
    println!("signed_commitment_1: {:?}", signed_commitment_1);

    let validator_proofs_1 = vec![
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("f68aec7304bf37f340dae2ea20fb5271ee28a3128812b84a615da4789e458bde").into(),
                hex!("93c6c7e160154c8467b700c291a1d4da94ae9aaf1c5010003a6aa3e9b18657ab").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 0,
            leaf: alice_pk.clone(),
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("aeb47a269393297f4b0a3c9c9cfd00c7a4195255274cf39d83dabc2fcc9ff3d7").into(),
                hex!("93c6c7e160154c8467b700c291a1d4da94ae9aaf1c5010003a6aa3e9b18657ab").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 1,
            leaf: bob_pk.clone(),
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("50bdd3ac4f54a04702a055c33303025b2038446c7334ed3b3341f310f052116f").into(),
                hex!("697ea2a8fe5b03468548a7a413424a6292ab44a82a6f5cc594c3fa7dda7ce402").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 2,
            leaf: charlie_pk.clone(),
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("3eb799651607280e854bd2e42c1df1c8e4a6167772dfb3c64a813e40f6e87136").into(),
                hex!("697ea2a8fe5b03468548a7a413424a6292ab44a82a6f5cc594c3fa7dda7ce402").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 3,
            leaf: dave_pk.clone(),
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("2145814fb41496b2881ca364a06e320fd1bf2fa7b94e1e37325cefbe29056519").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 4,
            leaf: eve_pk.clone(),
        },
    ];

    let  encoded_mmr_leaf_1 = hex!("c501000800000079f0451c096266bee167393545bafc7b27b7d14810084a843955624588ba29c1010000000000000005000000304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a20000000000000000000000000000000000000000000000000000000000000000");

    let leaf: Vec<u8> = Decode::decode(&mut &encoded_mmr_leaf_1[..]).unwrap();
    let mmr_leaf_1: MmrLeaf = Decode::decode(&mut &*leaf).unwrap();
    println!("mmr_leaf_1: {:?}", mmr_leaf_1);

    let encoded_mmr_proof_1 =  hex!("0800000000000000090000000000000004c2d6348aef1ef52e779c59bcc1d87fa0175b59b4fa2ea8fc322e4ceb2bdd1ea2");
    let mmr_proof_1 = MmrLeafProof::decode(&mut &encoded_mmr_proof_1[..]);
    println!("mmr_proof_1: {:?}", mmr_proof_1);
    //
    let result = permissionless_actions::start_updating_state_of_beefy_light_client(
        &worker,
        &user,
        &anchor,
        encoded_signed_commitment_1.to_vec(),
        validator_proofs_1,
        encoded_mmr_leaf_1.to_vec(),
        encoded_mmr_proof_1.to_vec(),
    )
    .await?;
    assert!(result.is_success());
    let result = permissionless_actions::try_complete_updating_state_of_beefy_light_client(
        &worker, &user, &anchor,
    )
    .await?;
    println!(
        "Result of 'try_complete_updating_state_of_beefy_light_client': {}",
        serde_json::to_string::<MultiTxsOperationProcessingResult>(&result).unwrap()
    );
    Ok(())
}

async fn update_state_of_beefy_light_client_2(
    worker: &Worker<Sandbox>,
    anchor: &Contract,
    user: &Account,
) -> anyhow::Result<()> {
    let alice_pk = beefy_ecdsa_to_ethereum(
        &hex!("020a1091341fe5664bfa1782d5e04779689068c916b04cb365ec3153755684d9a1")[..],
    );
    let bob_pk = beefy_ecdsa_to_ethereum(
        &hex!("0390084fdbf27d2b79d26a4f13f0ccd982cb755a661969143c37cbc49ef5b91f27")[..],
    );
    let charlie_pk = beefy_ecdsa_to_ethereum(
        &hex!("0389411795514af1627765eceffcbd002719f031604fadd7d188e2dc585b4e1afb")[..],
    );
    let dave_pk = beefy_ecdsa_to_ethereum(
        &hex!("03bc9d0ca094bd5b8b3225d7651eac5d18c1c04bf8ae8f8b263eebca4e1410ed0c")[..],
    );
    let eve_pk = beefy_ecdsa_to_ethereum(
        &hex!("031d10105e323c4afce225208f71a6441ee327a65b9e646e772500c74d31f669aa")[..],
    );

    let encoded_signed_commitment_2 = hex!("8d3cb96dca5110aff60423046bbf4a76db0e71158aa5586ffa3423fbaf9ef1da1100000000000000000000001401864ce4553324cc92db4ac622b9dbb031a6a4bd26ee1ab66e0272f567928865ec46847b55f98fa7e1dbafb0256f0a23e2f0a375e4547f5d1819d9b8694f17f6a80101c9ae8aad1b81e2249736324716c09c122889317e4f3e47066c501a839c15312e5c823dd37436d8e3bac8041329c5d0ed5dd94c45b5c1eed13d9111924f0a13c1000159fe06519c672d183de7776b6902a13c098d917721b5600a2296dca3a74a81bc01031a671fdb5e5050ff1f432d72e7a2c144ab38f8401ffd368e693257162a4600014290c6aa5028ceb3a3a773c80beee2821f3a7f5b43f592f7a82b0cbbbfab5ba41363daae5a7006fea2f89a30b4900f85fa82283587df789fd7b5b773ad7e8c410100");
    let signed_commitment_2 = SignedCommitment::decode(&mut &encoded_signed_commitment_2[..]);
    println!("signed_commitment_2: {:?}", signed_commitment_2);

    let validator_proofs_2 = vec![
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("f68aec7304bf37f340dae2ea20fb5271ee28a3128812b84a615da4789e458bde").into(),
                hex!("93c6c7e160154c8467b700c291a1d4da94ae9aaf1c5010003a6aa3e9b18657ab").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 0,
            leaf: alice_pk,
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("aeb47a269393297f4b0a3c9c9cfd00c7a4195255274cf39d83dabc2fcc9ff3d7").into(),
                hex!("93c6c7e160154c8467b700c291a1d4da94ae9aaf1c5010003a6aa3e9b18657ab").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 1,
            leaf: bob_pk,
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("50bdd3ac4f54a04702a055c33303025b2038446c7334ed3b3341f310f052116f").into(),
                hex!("697ea2a8fe5b03468548a7a413424a6292ab44a82a6f5cc594c3fa7dda7ce402").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 2,
            leaf: charlie_pk,
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("3eb799651607280e854bd2e42c1df1c8e4a6167772dfb3c64a813e40f6e87136").into(),
                hex!("697ea2a8fe5b03468548a7a413424a6292ab44a82a6f5cc594c3fa7dda7ce402").into(),
                hex!("55ca68207e72b7a7cd012364e03ac9ee560eb1b26de63f0ee42a649d74f3bf58").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 3,
            leaf: dave_pk,
        },
        ValidatorMerkleProof {
            root: hex!("304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a2").into(),
            proof: vec![
                hex!("2145814fb41496b2881ca364a06e320fd1bf2fa7b94e1e37325cefbe29056519").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 4,
            leaf: eve_pk,
        },
    ];

    let encoded_mmr_leaf_2 = hex!("c5010010000000d0a3a930e5f3b0f997c3794023c86f8ba28c6ba2cacf230d08d46be0fdf29435010000000000000005000000304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a20000000000000000000000000000000000000000000000000000000000000000");

    let leaf: Vec<u8> = Decode::decode(&mut &encoded_mmr_leaf_2[..]).unwrap();
    let mmr_leaf_2: MmrLeaf = Decode::decode(&mut &*leaf).unwrap();
    println!("mmr_leaf_2: {:?}", mmr_leaf_2);

    let encoded_mmr_proof_2 =  hex!("10000000000000001100000000000000048a766e1ab001e2ff796517dcfbff957a751c994aff4c3ba9447a46d88ec2ef15");
    let mmr_proof_2 = MmrLeafProof::decode(&mut &encoded_mmr_proof_2[..]);
    println!("mmr_proof_2: {:?}", mmr_proof_2);
    //
    let result = permissionless_actions::start_updating_state_of_beefy_light_client(
        &worker,
        &user,
        &anchor,
        encoded_signed_commitment_2.to_vec(),
        validator_proofs_2,
        encoded_mmr_leaf_2.to_vec(),
        encoded_mmr_proof_2.to_vec(),
    )
    .await?;
    assert!(result.is_success());
    let result = permissionless_actions::try_complete_updating_state_of_beefy_light_client(
        &worker, &user, &anchor,
    )
    .await?;
    println!(
        "Result of 'try_complete_updating_state_of_beefy_light_client': {}",
        serde_json::to_string::<MultiTxsOperationProcessingResult>(&result).unwrap()
    );
    Ok(())
}
