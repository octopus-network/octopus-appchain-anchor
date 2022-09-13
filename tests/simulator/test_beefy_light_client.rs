use crate::{
    common,
    contract_interfaces::{
        anchor_viewer, permissionless_actions, settings_manager, staking_actions,
    },
};
use appchain_anchor::types::{MultiTxsOperationProcessingResult, ValidatorMerkleProof};
use beefy_light_client::mmr::{MmrLeaf, MmrLeafProof};
use beefy_light_client::{beefy_ecdsa_to_ethereum, commitment::SignedCommitment};
use codec::Decode;
use hex_literal::hex;
use near_sdk::serde_json;
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
        _wat_faucet,
        users,
        mut appchain_message_nonce,
    ) = common::test_normal_actions(&worker, false, true, initial_public_keys).await?;
    //
    settings_manager::turn_off_beefy_light_client_witness_mode(&worker, &root, &anchor)
        .await
        .expect("Failed to call 'turn_off_beefy_light_client_witness_mode'");
    //
    // Update state of beefy light client
    //
    update_state_of_beefy_light_client_1(&worker, &anchor, &users[4]).await?;
    common::complex_viewer::print_latest_appchain_commitment(&worker, &anchor).await?;
    update_state_of_beefy_light_client_2(&worker, &anchor, &users[1]).await?;
    common::complex_viewer::print_latest_appchain_commitment(&worker, &anchor).await?;
    //
    settings_manager::turn_on_beefy_light_client_witness_mode(&worker, &root, &anchor)
        .await
        .expect("Failed to call 'turn_on_beefy_light_client_witness_mode'");
    //
    // Try start and complete switching era1
    //
    appchain_message_nonce += 1;
    common::complex_actions::switch_era(
        &worker,
        &users[5],
        &anchor,
        1,
        appchain_message_nonce,
        true,
    )
    .await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(1)).await?;
    common::complex_viewer::print_delegator_list_of(&worker, &anchor, 1, &users[0]).await?;
    //
    // Distribut reward of era0
    //
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &users[5],
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
    let result = staking_actions::decrease_stake(
        &worker,
        &users[1],
        &anchor,
        common::to_actual_amount(1000, 18),
    )
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
        common::to_actual_amount(200, 18),
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
    common::complex_actions::switch_era(
        &worker,
        &users[5],
        &anchor,
        2,
        appchain_message_nonce,
        true,
    )
    .await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(2)).await?;
    common::complex_viewer::print_delegator_list_of(&worker, &anchor, 2, &users[0]).await?;
    //
    // Distribute reward of era1
    //
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &users[5],
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
    common::complex_actions::switch_era(
        &worker,
        &users[5],
        &anchor,
        3,
        appchain_message_nonce,
        true,
    )
    .await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(3)).await?;
    common::complex_viewer::print_delegator_list_of(&worker, &anchor, 3, &users[0]).await?;
    //
    // Distribute reward of era2
    //
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &users[5],
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
    common::complex_actions::switch_era(
        &worker,
        &users[5],
        &anchor,
        4,
        appchain_message_nonce,
        true,
    )
    .await?;
    common::complex_viewer::print_validator_list_of(&worker, &anchor, Some(4)).await?;
    common::complex_viewer::print_delegator_list_of(&worker, &anchor, 4, &users[0]).await?;
    //
    // Distribute reward of era3
    //
    appchain_message_nonce += 1;
    common::complex_actions::distribute_reward_of(
        &worker,
        &users[5],
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
    //
    //
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

    let encoded_signed_commitment_1 = hex!("046d688017ffed791fa51b459d3c953a1f8f3e4718bcf8aa571a19bc0327d82761d3257909000000000000000000000004d80500000010c73029d26bba5d549db469b75950c4cb55aaf43de0044a32612acca99445bbf93a1edbc9f5fa5151c1a2e2b6f59968eb1485d001c6b9078c2ed310bad20779b001a4b79f6018e3936a64bd3281dca522fb33bf68720afff458c7ca0db1bfbd270d36c5c3db98abb59d9abbeda7b74b83510120172e7aa6c74f5c9239c85befa85f003bed8b85ff2f466df62569d4cd0169773b4ae4dde1139d4d0721b497f938312803e1885b21f6230ef5a8e44ad3dbbb1cd0e89226a41e35507e91ed62bcf4dc22013f45d94e3a6b97f5208d90d2bf3f2702a440f3f453c438cdd553bf2f2cc02cc23b230b3b12c1e68e39fbaf701e65457a372facba3c530ab56f3eec5e6766eddb01");
    let signed_commitment_1 = SignedCommitment::decode(&mut &encoded_signed_commitment_1[..]);
    println!("signed_commitment_1: {:?}", signed_commitment_1);

    let validator_proofs_1 = vec![
        ValidatorMerkleProof {
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
            proof: vec![
                hex!("2145814fb41496b2881ca364a06e320fd1bf2fa7b94e1e37325cefbe29056519").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 4,
            leaf: eve_pk.clone(),
        },
    ];

    let  encoded_mmr_leaf_1 = hex!("c50100080000005717d626ed925ebf1deaf25cb24ad7bca9384bbe533a938856466cc09fd26292010000000000000005000000304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a20000000000000000000000000000000000000000000000000000000000000000");

    let leaf: Vec<u8> = Decode::decode(&mut &encoded_mmr_leaf_1[..]).unwrap();
    let mmr_leaf_1: MmrLeaf = Decode::decode(&mut &*leaf).unwrap();
    println!("mmr_leaf_1: {:?}", mmr_leaf_1);

    let encoded_mmr_proof_1 =  hex!("0800000000000000090000000000000004effbabd0a9fcade34208684b3d5d69a52b2c9bc9265d872c2590636acd6342a0");
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

    let encoded_signed_commitment_2 = hex!("046d688037d21b14f9701ca2deb9946dbad32de48d8df3ad8988bfaabdbafa329fe07ccd11000000000000000000000004d80500000010b6f60090f011f376a7673d38a810ad15423381fbf6e8e1a88c2d39d58b5473b83dae3750c39be39be17bada861944b2d6f43c7e329b247905eb17dc3ecdb7f8a0062969c39737b7b3101d639ed2bd8aa3a61647bb4569d2a6c78b450e46012879919c90b149493d523d030490e389b3d4ee1e3f2a24f4e0cf5cd4944c03921ed3500389cf1cfe7c117052416db37920594387170fd404f79b98dc39f9b56ede6865a10306bf55a2d8814e36dbb51142f015813acbb1b187fdfefcc1f05b6505dce83019962e14afb83630dffec978b47f52016af699d21d4b1661acf4c01bb4845adcc4fa3e421dca35fb0c4d58d387bdc0d11ec161502e7c6f85c86849f569bc8b4c401");
    let signed_commitment_2 = SignedCommitment::decode(&mut &encoded_signed_commitment_2[..]);
    println!("signed_commitment_2: {:?}", signed_commitment_2);

    let validator_proofs_2 = vec![
        ValidatorMerkleProof {
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
            proof: vec![
                hex!("2145814fb41496b2881ca364a06e320fd1bf2fa7b94e1e37325cefbe29056519").into(),
            ],
            number_of_leaves: 5,
            leaf_index: 4,
            leaf: eve_pk,
        },
    ];

    let encoded_mmr_leaf_2 = hex!("c501001000000027aa6e9a63fe73429eaadc49018eed6d2f6362cdb18744677acfaca8be94838a010000000000000005000000304803fa5a91d9852caafe04b4b867a4ed27a07a5bee3d1507b4b187a68777a20000000000000000000000000000000000000000000000000000000000000000");

    let leaf: Vec<u8> = Decode::decode(&mut &encoded_mmr_leaf_2[..]).unwrap();
    let mmr_leaf_2: MmrLeaf = Decode::decode(&mut &*leaf).unwrap();
    println!("mmr_leaf_2: {:?}", mmr_leaf_2);

    let encoded_mmr_proof_2 =  hex!("10000000000000001100000000000000043b96661a7161a6a760af588ebdefc79401e1c046d889d59f76d824406f713188");
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
