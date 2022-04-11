use std::{collections::HashMap, convert::TryInto, str::FromStr};

use appchain_anchor::{
    types::{
        AnchorSettings, AnchorStatus, AppchainCommitment, AppchainSettings, AppchainState,
        MultiTxsOperationProcessingResult, ValidatorProfile, ValidatorSetInfo,
        WrappedAppchainToken,
    },
    AppchainAnchorContract, AppchainEvent, AppchainMessage,
};
use mock_appchain_registry::MockAppchainRegistryContract;
use mock_oct_token::MockOctTokenContract;
use wrapped_appchain_token::WrappedAppchainTokenContract;

use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_sdk::{
    json_types::{U128, U64},
    serde_json, AccountId, Balance,
};
use near_sdk_sim::{
    call, deploy, init_simulator, lazy_static_include, runtime::GenesisConfig, to_yocto, view,
    ContractAccount, ExecutionResult, UserAccount,
};

use num_format::{Locale, ToFormattedString};

use crate::sudo_actions;
use crate::{anchor_viewer, staking_actions, token_viewer};
use crate::{
    lifecycle_actions, permissionless_actions, settings_manager, validator_actions,
    wrapped_appchain_token_manager,
};

const INIT_DEPOSIT_FOR_CONTRACT: Balance = 30_000_000_000_000_000_000_000_000;
const TOTAL_SUPPLY: u128 = 100_000_000;

lazy_static_include::lazy_static_include_bytes! {
    TOKEN_WASM_BYTES => "res/mock_oct_token.wasm",
    REGISTRY_WASM_BYTES => "res/mock_appchain_registry.wasm",
    ANCHOR_WASM_BYTES => "res/appchain_anchor.wasm",
    WAT_WASM_BYTES => "res/mock_wrapped_appchain_token.wasm",
    OLD_ANCHOR_WASM_BYTES => "res/previous_appchain_anchor.wasm",
}

// Register the given `user` to oct_token
fn register_user_to_oct_token(
    account: &UserAccount,
    contract: &ContractAccount<MockOctTokenContract>,
) {
    let result = call!(
        account,
        contract.storage_deposit(Option::from(account.account_id()), Option::None),
        near_sdk::env::storage_byte_cost() * 125,
        near_sdk_sim::DEFAULT_GAS / 2
    );
    print_execution_result("register_user_to_oct_token", &result);
    result.assert_success();
}

fn register_user_to_wat_token(
    account: &UserAccount,
    contract: &ContractAccount<WrappedAppchainTokenContract>,
) {
    let result = call!(
        account,
        contract.storage_deposit(Option::from(account.account_id()), Option::None),
        near_sdk::env::storage_byte_cost() * 125,
        near_sdk_sim::DEFAULT_GAS / 2
    );
    print_execution_result("register_user_to_wat_token", &result);
    result.assert_success();
}

pub fn ft_transfer_oct_token(
    sender: &UserAccount,
    receiver: &UserAccount,
    amount: u128,
    oct_token: &ContractAccount<MockOctTokenContract>,
) {
    let result = call!(
        sender,
        oct_token.ft_transfer(receiver.account_id(), U128::from(amount), Option::None),
        1,
        near_sdk_sim::DEFAULT_GAS
    );
    print_execution_result("ft_transfer_oct_token", &result);
    result.assert_success();
}

pub fn ft_transfer_call_oct_token(
    sender: &UserAccount,
    receiver: &UserAccount,
    amount: u128,
    msg: String,
    oct_token: &ContractAccount<MockOctTokenContract>,
) -> ExecutionResult {
    let result = call!(
        sender,
        oct_token.ft_transfer_call(
            receiver.account_id(),
            U128::from(amount),
            Option::None,
            msg.clone()
        ),
        1,
        near_sdk_sim::DEFAULT_GAS
    );
    print_execution_result("ft_transfer_call_oct_token", &result);
    result.assert_success();
    result
}

fn get_genesis_config() -> GenesisConfig {
    let mut genesis_config = GenesisConfig::default();
    genesis_config.block_prod_time = 86400 * 1_000_000_000;
    genesis_config
}

pub fn init(
    total_supply: u128,
    with_old_anchor: bool,
) -> (
    UserAccount,
    ContractAccount<MockOctTokenContract>,
    ContractAccount<MockAppchainRegistryContract>,
    ContractAccount<AppchainAnchorContract>,
    Vec<UserAccount>,
) {
    let root = init_simulator(Some(get_genesis_config()));
    let mut users: Vec<UserAccount> = Vec::new();
    // Deploy and initialize contracts
    let oct_ft_metadata = FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.to_string(),
        name: "OCTToken".to_string(),
        symbol: "OCT".to_string(),
        icon: None,
        reference: None,
        reference_hash: None,
        decimals: 18,
    };
    let oct_token = deploy! {
        contract: MockOctTokenContract,
        contract_id: "oct_token",
        bytes: &TOKEN_WASM_BYTES,
        signer_account: root,
        init_method: new(root.account_id(), U128::from(total_supply), oct_ft_metadata)
    };
    let registry = deploy! {
        contract: MockAppchainRegistryContract,
        contract_id: "registry",
        bytes: &REGISTRY_WASM_BYTES,
        signer_account: root,
        init_method: new(oct_token.account_id())
    };
    let anchor = match with_old_anchor {
        true => deploy! {
            contract: AppchainAnchorContract,
            contract_id: "anchor",
            bytes: &OLD_ANCHOR_WASM_BYTES,
            signer_account: root,
            deposit: INIT_DEPOSIT_FOR_CONTRACT,
            init_method: new(
                "test_appchain_id".to_string(),
                registry.account_id(),
                oct_token.account_id()
            )
        },
        false => deploy! {
            contract: AppchainAnchorContract,
            contract_id: "anchor",
            bytes: &ANCHOR_WASM_BYTES,
            signer_account: root,
            deposit: INIT_DEPOSIT_FOR_CONTRACT,
            init_method: new(
                "test_appchain_id".to_string(),
                registry.account_id(),
                oct_token.account_id()
            )
        },
    };
    register_user_to_oct_token(&registry.user_account, &oct_token);
    register_user_to_oct_token(&anchor.user_account, &oct_token);
    // Create users and transfer a certain amount of OCT token to them
    let alice = root.create_user(AccountId::from_str("alice").unwrap(), to_yocto("100"));
    register_user_to_oct_token(&alice, &oct_token);
    ft_transfer_oct_token(&root, &alice, total_supply / 10, &oct_token);
    users.push(alice);
    let bob = root.create_user(AccountId::from_str("bob").unwrap(), to_yocto("100"));
    register_user_to_oct_token(&bob, &oct_token);
    ft_transfer_oct_token(&root, &bob, total_supply / 10, &oct_token);
    users.push(bob);
    let charlie = root.create_user(AccountId::from_str("charlie").unwrap(), to_yocto("100"));
    register_user_to_oct_token(&charlie, &oct_token);
    ft_transfer_oct_token(&root, &charlie, total_supply / 10, &oct_token);
    users.push(charlie);
    let dave = root.create_user(AccountId::from_str("dave").unwrap(), to_yocto("100"));
    register_user_to_oct_token(&dave, &oct_token);
    ft_transfer_oct_token(&root, &dave, total_supply / 10, &oct_token);
    users.push(dave);
    let eve = root.create_user(AccountId::from_str("eve").unwrap(), to_yocto("100"));
    register_user_to_oct_token(&eve, &oct_token);
    ft_transfer_oct_token(&root, &eve, total_supply / 10, &oct_token);
    users.push(eve);
    // Print initial storage balance of anchor
    print_anchor_storage_balance(&anchor);
    // Return initialized UserAccounts
    (root, oct_token, registry, anchor, users)
}

pub fn deploy_new_anchor_contract(anchor: &ContractAccount<AppchainAnchorContract>) {
    let transaction = anchor.user_account.create_transaction(anchor.account_id());
    let result = transaction
        .deploy_contract((&ANCHOR_WASM_BYTES).to_vec())
        .submit();
    result.assert_success();
}

pub fn deploy_wrapped_appchain_token_contract(
    root: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    premined_balance: U128,
    users: &Vec<UserAccount>,
) -> ContractAccount<WrappedAppchainTokenContract> {
    let wat_ft_metadata = FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.to_string(),
        name: "WrappedAppchainToken".to_string(),
        symbol: "WAT".to_string(),
        icon: None,
        reference: None,
        reference_hash: None,
        decimals: 18,
    };
    let wrapped_appchain_token = deploy! {
        contract: WrappedAppchainTokenContract,
        contract_id: "wrapped_appchain_token",
        bytes: &WAT_WASM_BYTES,
        signer_account: root,
        init_method: new(
            anchor.account_id(),
            users[0].account_id(),
            premined_balance,
            wat_ft_metadata
        )
    };
    for user in users {
        register_user_to_wat_token(&user, &wrapped_appchain_token);
    }
    wrapped_appchain_token
}

pub fn to_oct_amount(amount: u128) -> u128 {
    let bt_decimals_base = (10 as u128).pow(18);
    amount * bt_decimals_base
}

pub fn test_normal_actions(
    with_old_anchor: bool,
    to_confirm_view_result: bool,
) -> (
    UserAccount,
    ContractAccount<MockOctTokenContract>,
    ContractAccount<WrappedAppchainTokenContract>,
    ContractAccount<MockAppchainRegistryContract>,
    ContractAccount<AppchainAnchorContract>,
    Vec<UserAccount>,
    u32,
) {
    let total_supply = to_oct_amount(TOTAL_SUPPLY);
    let (root, oct_token, registry, anchor, users) = init(total_supply, with_old_anchor);
    let user0_id_in_appchain =
        "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d".to_string();
    let user1_id_in_appchain =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da270".to_string();
    let user4_id_in_appchain =
        "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da273".to_string();
    let mut user0_profile = HashMap::<String, String>::new();
    user0_profile.insert("key0".to_string(), "value0".to_string());
    let mut user1_profile = HashMap::<String, String>::new();
    user1_profile.insert("key1".to_string(), "value1".to_string());
    let mut user4_profile = HashMap::<String, String>::new();
    user4_profile.insert("key4".to_string(), "value4".to_string());
    //
    // Check initial status
    //
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor),
        AppchainState::Staging
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(anchor_status.total_stake_in_next_era.0, 0);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 0);
    }
    //
    //
    //
    let result = settings_manager::set_price_of_oct_token(&users[4], &anchor, 2_130_000);
    assert!(!result.is_ok());
    let result = wrapped_appchain_token_manager::set_price_of_wrapped_appchain_token(
        &users[4], &anchor, 110_000,
    );
    assert!(!result.is_ok());
    let result = settings_manager::set_token_price_maintainer_account(&root, &anchor, &users[4]);
    result.assert_success();
    //
    // Initialize wrapped appchain token contract.
    //
    let result = wrapped_appchain_token_manager::set_price_of_wrapped_appchain_token(
        &users[4], &anchor, 110,
    );
    result.assert_success();
    let result = wrapped_appchain_token_manager::set_account_of_wrapped_appchain_token(
        &root,
        &anchor,
        AccountId::from_str("wrapped_appchain_token").unwrap(),
    );
    result.assert_success();
    let wrapped_appchain_token = deploy_wrapped_appchain_token_contract(
        &root,
        &anchor,
        U128::from(total_supply / 2),
        &users,
    );
    if to_confirm_view_result {
        print_wrapped_appchain_token_info(&anchor);
    }
    //
    // user0 register validator (error)
    //
    let user0_balance = token_viewer::get_oct_balance_of(&users[0], &oct_token);
    let amount0 = to_oct_amount(9999);
    let result = staking_actions::register_validator(
        &users[0],
        &oct_token,
        &anchor,
        &None,
        amount0,
        true,
        HashMap::new(),
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[0], &oct_token).0,
        user0_balance.0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(anchor_status.total_stake_in_next_era.0, 0);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 0);
    }
    //
    // user0 register validator
    //
    let user0_balance = token_viewer::get_oct_balance_of(&users[0], &oct_token);
    let amount0 = to_oct_amount(23_000);
    let result = staking_actions::register_validator(
        &users[0],
        &oct_token,
        &anchor,
        &None,
        amount0,
        true,
        HashMap::new(),
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[0], &oct_token).0,
        user0_balance.0 - amount0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(anchor_status.total_stake_in_next_era.0, amount0);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 1);
        print_validator_profile(&anchor, &users[0].account_id(), &user0_id_in_appchain);
    }
    //
    // user1 register validator
    //
    let user1_balance = token_viewer::get_oct_balance_of(&users[1], &oct_token);
    let amount1 = to_oct_amount(25_000);
    let result = staking_actions::register_validator(
        &users[1],
        &oct_token,
        &anchor,
        &None,
        amount1,
        false,
        HashMap::new(),
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[1], &oct_token).0,
        user1_balance.0 - amount1
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(anchor_status.total_stake_in_next_era.0, amount0 + amount1);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
        print_validator_profile(&anchor, &users[1].account_id(), &user1_id_in_appchain);
    }
    //
    // user2 register delegator to user0 (error)
    //
    let user2_balance = token_viewer::get_oct_balance_of(&users[2], &oct_token);
    let amount2 = to_oct_amount(499);
    let result = staking_actions::register_delegator(
        &users[2],
        &oct_token,
        &anchor,
        &users[0].account_id(),
        amount2,
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[2], &oct_token).0,
        user2_balance.0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(anchor_status.total_stake_in_next_era.0, amount0 + amount1);
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
    //
    // user2 register delegator to user0
    //
    let user2_balance = token_viewer::get_oct_balance_of(&users[2], &oct_token);
    let amount2_0 = to_oct_amount(1000);
    let result = staking_actions::register_delegator(
        &users[2],
        &oct_token,
        &anchor,
        &users[0].account_id(),
        amount2_0,
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[2], &oct_token).0,
        user2_balance.0 - amount2_0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
    //
    // user2 register delegator to user1 (error)
    //
    let user2_balance = token_viewer::get_oct_balance_of(&users[2], &oct_token);
    let amount2_1 = to_oct_amount(1000);
    let result = staking_actions::register_delegator(
        &users[2],
        &oct_token,
        &anchor,
        &users[1].account_id(),
        amount2_1,
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[2], &oct_token).0,
        user2_balance.0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
    //
    // user3 register delegator to user0
    //
    let user3_balance = token_viewer::get_oct_balance_of(&users[3], &oct_token);
    let amount3_0 = to_oct_amount(2000);
    let result = staking_actions::register_delegator(
        &users[3],
        &oct_token,
        &anchor,
        &users[0].account_id(),
        amount3_0,
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[3], &oct_token).0,
        user3_balance.0 - amount3_0
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0 + amount3_0
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
    //
    // user0 increase stake
    //
    let user0_balance = token_viewer::get_oct_balance_of(&users[0], &oct_token);
    let amount0_p = to_oct_amount(1_200);
    let result = staking_actions::increase_stake(&users[0], &oct_token, &anchor, amount0_p);
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[0], &oct_token).0,
        user0_balance.0 - amount0_p
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0 + amount3_0 + amount0_p
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
    //
    // user2 increase delegation to user0
    //
    let user2_balance = token_viewer::get_oct_balance_of(&users[2], &oct_token);
    let amount2_0_p = to_oct_amount(500);
    let result = staking_actions::increase_delegation(
        &users[2],
        &oct_token,
        &anchor,
        &users[0].account_id(),
        amount2_0_p,
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[2], &oct_token).0,
        user2_balance.0 - amount2_0_p
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0 + amount3_0 + amount0_p + amount2_0_p
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 2);
    }
    //
    // Print anchor status and staking histories
    //
    if to_confirm_view_result {
        print_anchor_status(&anchor);
        print_wrapped_appchain_token_info(&anchor);
        print_staking_histories(&anchor);
        print_validator_list_of(&anchor, None);
    }
    //
    // Try go_booting
    //
    let result = lifecycle_actions::go_booting(&root, &anchor);
    assert!(!result.is_ok());
    //
    // Set appchain settings and try go_booting
    //
    let result = settings_manager::set_rpc_endpoint(&root, &anchor, "rpc_endpoint".to_string());
    result.assert_success();
    let result = settings_manager::set_subql_endpoint(&root, &anchor, "subql_endpoint".to_string());
    result.assert_success();
    let result = settings_manager::set_era_reward(&root, &anchor, to_oct_amount(10));
    result.assert_success();
    let result = lifecycle_actions::go_booting(&root, &anchor);
    assert!(!result.is_ok());
    //
    // Change protocol settings and try go_booting
    //
    let result = settings_manager::change_minimum_validator_count(&root, &anchor, 1);
    result.assert_success();
    let result = lifecycle_actions::go_booting(&root, &anchor);
    assert!(!result.is_ok());
    //
    // Change price of OCT token and try go_booting
    //
    let result = settings_manager::set_price_of_oct_token(&users[4], &anchor, 2_130_000);
    result.assert_success();
    let result = lifecycle_actions::go_booting(&root, &anchor);
    result.assert_success();
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor),
        AppchainState::Booting
    );
    //
    // Check validator set of era0
    //
    let appchain_message_nonce: u32 = 0;
    if to_confirm_view_result {
        print_anchor_status(&anchor);
        print_staking_histories(&anchor);
        print_validator_set_info_of(&anchor, U64::from(0));
        print_validator_list_of(&anchor, Some(0));
        print_delegator_list_of(&anchor, 0, &users[0]);
    }
    //
    // Initialize beefy light client
    //
    let result =
        lifecycle_actions::initialize_beefy_light_client(&root, &anchor, vec!["0x00".to_string()]);
    result.assert_success();
    //
    // Go live
    //
    let result = lifecycle_actions::go_live(&root, &anchor);
    result.assert_success();
    assert_eq!(
        anchor_viewer::get_appchain_state(&anchor),
        AppchainState::Active
    );
    //
    // Change id in appchain and profile of user0, user1
    //
    let result =
        validator_actions::set_validator_id_in_appchain(&users[0], &anchor, &user0_id_in_appchain);
    result.assert_success();
    let result = validator_actions::set_validator_profile(&users[0], &anchor, &user0_profile);
    result.assert_success();
    if to_confirm_view_result {
        print_validator_profile(&anchor, &users[0].account_id(), &user0_id_in_appchain);
    }
    let result =
        validator_actions::set_validator_id_in_appchain(&users[1], &anchor, &user1_id_in_appchain);
    result.assert_success();
    let result = validator_actions::set_validator_profile(&users[1], &anchor, &user1_profile);
    result.assert_success();
    if to_confirm_view_result {
        print_validator_profile(&anchor, &users[1].account_id(), &user1_id_in_appchain);
    }
    //
    // user4 register validator
    //
    let user4_balance = token_viewer::get_oct_balance_of(&users[4], &oct_token);
    let amount4 = to_oct_amount(13_000);
    let result = staking_actions::register_validator(
        &users[4],
        &oct_token,
        &anchor,
        &Some(user4_id_in_appchain.clone()),
        amount4,
        true,
        user4_profile,
    );
    result.assert_success();
    assert_eq!(
        token_viewer::get_oct_balance_of(&users[4], &oct_token).0,
        user4_balance.0 - amount4
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(&anchor);
        assert_eq!(
            anchor_status.total_stake_in_next_era.0,
            amount0 + amount1 + amount2_0 + amount3_0 + amount0_p + amount2_0_p + amount4
        );
        assert_eq!(anchor_status.validator_count_in_next_era.0, 3);
        print_validator_profile(&anchor, &users[4].account_id(), &user4_id_in_appchain);
    }
    //
    // Print staking histories
    //
    if to_confirm_view_result {
        print_staking_histories(&anchor);
    }
    //
    //
    //
    (
        root,
        oct_token,
        wrapped_appchain_token,
        registry,
        anchor,
        users,
        appchain_message_nonce,
    )
}

pub fn print_execution_result(function_name: &str, result: &ExecutionResult) {
    println!(
        "Gas burnt of function '{}': {}",
        function_name,
        result.gas_burnt().0.to_formatted_string(&Locale::en)
    );
    let results = result.promise_results();
    for sub_result in results {
        if let Some(sub_result) = sub_result {
            if sub_result.is_ok() {
                let logs = sub_result.logs();
                if logs.len() > 0 {
                    println!("{:#?}", logs);
                }
            } else {
                println!("{:#?}", sub_result.outcome());
            }
        }
    }
}

fn print_anchor_storage_balance(anchor: &ContractAccount<AppchainAnchorContract>) {
    let view_result = view!(anchor.get_storage_balance());
    if view_result.is_err() {
        println!("{:#?}", view_result);
    }
    assert!(view_result.is_ok());
    println!(
        "Anchor storage balance: {}",
        serde_json::to_string::<U128>(&view_result.unwrap_json::<U128>()).unwrap()
    );
}

pub fn print_anchor_status(anchor: &ContractAccount<AppchainAnchorContract>) {
    let anchor_status = anchor_viewer::get_anchor_status(anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
}

pub fn print_appchain_settings(anchor: &ContractAccount<AppchainAnchorContract>) {
    let appchain_settings = anchor_viewer::get_appchain_settings(anchor);
    println!(
        "Appchain settings: {}",
        serde_json::to_string::<AppchainSettings>(&appchain_settings).unwrap()
    );
}

pub fn print_anchor_settings(anchor: &ContractAccount<AppchainAnchorContract>) {
    let anchor_settings = anchor_viewer::get_anchor_settings(anchor);
    println!(
        "Anchor settings: {}",
        serde_json::to_string::<AnchorSettings>(&anchor_settings).unwrap()
    );
}

pub fn print_validator_set_info_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: U64,
) {
    let validator_set_info = anchor_viewer::get_validator_set_info_of(anchor, era_number);
    println!(
        "Validator set {} info: {}",
        era_number.0,
        serde_json::to_string::<ValidatorSetInfo>(&validator_set_info).unwrap()
    );
}

pub fn print_wrapped_appchain_token_info(anchor: &ContractAccount<AppchainAnchorContract>) {
    let wrapped_appchain_token_info = anchor_viewer::get_wrapped_appchain_token(&anchor);
    println!(
        "Wrapped appchain token: {}",
        serde_json::to_string::<WrappedAppchainToken>(&wrapped_appchain_token_info).unwrap()
    );
}

pub fn print_near_fungible_tokens(anchor: &ContractAccount<AppchainAnchorContract>) {
    let near_fungible_tokens = anchor_viewer::get_near_fungible_tokens(&anchor);
    near_fungible_tokens.iter().for_each(|record| {
        println!(
            "Near fungible token: {}",
            serde_json::to_string(&record).unwrap()
        );
    });
}

pub fn print_validator_profile(
    anchor: &ContractAccount<AppchainAnchorContract>,
    account_id: &AccountId,
    account_id_in_appchain: &String,
) {
    let validator_profile = anchor_viewer::get_validator_profile(&anchor, &account_id);
    println!(
        "Profile of '{}': {}",
        &account_id,
        serde_json::to_string::<ValidatorProfile>(&validator_profile.unwrap()).unwrap()
    );
    let validator_profile =
        anchor_viewer::get_validator_profile_by_id_in_appchain(&anchor, &account_id_in_appchain);
    if validator_profile.is_some() {
        println!(
            "Profile of '{}': {}",
            &account_id_in_appchain,
            serde_json::to_string::<ValidatorProfile>(&validator_profile.unwrap()).unwrap()
        );
    }
}

pub fn print_anchor_events(anchor: &ContractAccount<AppchainAnchorContract>) {
    let index_range = anchor_viewer::get_index_range_of_anchor_event_history(anchor);
    for i in index_range.start_index.0..index_range.end_index.0 + 1 {
        if let Some(anchor_event_history) =
            anchor_viewer::get_anchor_event_history(anchor, i.try_into().unwrap())
        {
            println!(
                "Anchor event history {}: {}",
                i,
                serde_json::to_string(&anchor_event_history).unwrap()
            );
        }
    }
    let records = anchor_viewer::get_anchor_event_histories(anchor, 0, None);
    records.iter().for_each(|record| {
        println!(
            "Anchor event history {}: {}",
            record.index.0,
            serde_json::to_string(&record).unwrap()
        );
    });
}

pub fn print_appchain_notifications(anchor: &ContractAccount<AppchainAnchorContract>) {
    let index_range = anchor_viewer::get_index_range_of_appchain_notification_history(anchor);
    for i in index_range.start_index.0..index_range.end_index.0 + 1 {
        if let Some(appchain_notification_history) =
            anchor_viewer::get_appchain_notification_history(anchor, i.try_into().unwrap())
        {
            println!(
                "Appchain notification history {}: {}",
                i,
                serde_json::to_string(&appchain_notification_history).unwrap()
            );
        }
    }
    let records = anchor_viewer::get_appchain_notification_histories(anchor, 0, None);
    records.iter().for_each(|record| {
        println!(
            "Appchain notification history {}: {}",
            record.index.0,
            serde_json::to_string(&record).unwrap()
        );
    });
}

pub fn print_staking_histories(anchor: &ContractAccount<AppchainAnchorContract>) {
    let index_range = anchor_viewer::get_index_range_of_staking_history(anchor);
    for i in index_range.start_index.0..index_range.end_index.0 + 1 {
        if let Some(staking_history) =
            anchor_viewer::get_staking_history(anchor, i.try_into().unwrap())
        {
            println!(
                "Staking history {}: {}",
                i,
                serde_json::to_string(&staking_history).unwrap()
            );
        }
    }
}

pub fn print_user_staking_histories_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    user: &UserAccount,
) {
    let staking_histories = anchor_viewer::get_user_staking_histories_of(anchor, user.account_id());
    let mut index = 0;
    for staking_history in staking_histories {
        println!(
            "Staking history {} of account {}: {}",
            index,
            &user.account_id(),
            serde_json::to_string(&staking_history).unwrap()
        );
        index += 1;
    }
}

pub fn print_validator_list_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: Option<u64>,
) {
    let validator_list = anchor_viewer::get_validator_list_of(anchor, era_number);
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
    }
}

pub fn print_delegator_list_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: u64,
    validator: &UserAccount,
) {
    let delegator_list =
        anchor_viewer::get_delegators_of_validator_in_era(&anchor, era_number, validator);
    let mut index = 0;
    for delegator in delegator_list {
        println!(
            "Delegator {} of {} in era {}: {}",
            index,
            validator.account_id(),
            era_number,
            serde_json::to_string(&delegator).unwrap()
        );
        index += 1;
    }
}

pub fn print_validator_reward_histories(
    anchor: &ContractAccount<AppchainAnchorContract>,
    validator: &UserAccount,
    end_era: u64,
) {
    let reward_histories = anchor_viewer::get_validator_rewards_of(anchor, 0, end_era, validator);
    let mut index = 0;
    for reward_history in reward_histories {
        println!(
            "Reward history {} of {}: {}",
            index,
            validator.account_id().to_string(),
            serde_json::to_string(&reward_history).unwrap()
        );
        index += 1;
    }
}

pub fn print_delegator_reward_histories(
    anchor: &ContractAccount<AppchainAnchorContract>,
    delegator: &UserAccount,
    validator: &UserAccount,
    end_era: u64,
) {
    let reward_histories =
        anchor_viewer::get_delegator_rewards_of(anchor, 0, end_era, delegator, validator);
    let mut index = 0;
    for reward_history in reward_histories {
        println!(
            "Reward history {} of {} to {}: {}",
            index,
            delegator.account_id().to_string(),
            validator.account_id().to_string(),
            serde_json::to_string(&reward_history).unwrap()
        );
        index += 1;
    }
}

pub fn print_unbonded_stakes_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    user: &UserAccount,
) {
    let unbonded_stakes = anchor_viewer::get_unbonded_stakes_of(anchor, user);
    let mut index = 0;
    for unbonded_stake in unbonded_stakes {
        println!(
            "Unbonded stake {} of {}: {}",
            index,
            user.account_id(),
            serde_json::to_string(&unbonded_stake).unwrap()
        );
        index += 1;
    }
}

pub fn print_latest_appchain_commitment(anchor: &ContractAccount<AppchainAnchorContract>) {
    let appchain_commitment = anchor_viewer::get_latest_commitment_of_appchain(&anchor);
    println!(
        "Latest appchain commitment: {}",
        serde_json::to_string::<Option<AppchainCommitment>>(&appchain_commitment).unwrap()
    );
}

pub fn print_wat_balance_of_anchor(
    anchor: &ContractAccount<AppchainAnchorContract>,
    wrapped_appchain_token: &ContractAccount<WrappedAppchainTokenContract>,
) {
    let wat_balance_of_anchor =
        token_viewer::get_wat_balance_of(&anchor.account_id(), wrapped_appchain_token);
    println!(
        "Wrapped appchain token balance of anchor contract: {}",
        wat_balance_of_anchor.0
    );
}

pub fn print_appchain_messages(anchor: &ContractAccount<AppchainAnchorContract>) {
    let appchain_messages = anchor_viewer::get_appchain_messages(anchor, 0, None);
    for appchain_message in appchain_messages {
        println!(
            "Appchain message '{}': {}",
            appchain_message.nonce,
            serde_json::to_string(&appchain_message).unwrap()
        );
    }
}

pub fn print_appchain_messages_processing_results(
    anchor: &ContractAccount<AppchainAnchorContract>,
) {
    let appchain_messages = anchor_viewer::get_appchain_message_processing_results(anchor, 0, None);
    let mut index = 1;
    for appchain_message in appchain_messages {
        println!(
            "Appchain message processing result '{}': {}",
            index,
            serde_json::to_string(&appchain_message).unwrap()
        );
        index += 1;
    }
}

pub fn process_appchain_messages(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
) {
    loop {
        let result = permissionless_actions::process_appchain_messages(signer, anchor);
        println!(
            "Process appchain messages: {}",
            serde_json::to_string::<MultiTxsOperationProcessingResult>(&result).unwrap()
        );
        print_anchor_status(anchor);
        match result {
            MultiTxsOperationProcessingResult::Ok => break,
            MultiTxsOperationProcessingResult::NeedMoreGas => (),
            MultiTxsOperationProcessingResult::Error(message) => {
                panic!("Failed to process appchain messages: {}", &message);
            }
        }
    }
}

pub fn switch_era(
    root: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: u32,
    appchain_message_nonce: u32,
    to_confirm_view_result: bool,
) {
    if era_number > 0 {
        let mut appchain_messages = Vec::<AppchainMessage>::new();
        appchain_messages.push(AppchainMessage {
            appchain_event: AppchainEvent::EraSwitchPlaned { era_number },
            nonce: appchain_message_nonce,
        });
        sudo_actions::stage_appchain_messages(root, anchor, appchain_messages);
    }
    process_appchain_messages(root, anchor);
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(anchor);
        println!(
            "Anchor status: {}",
            serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
        );
        let validator_set_info =
            anchor_viewer::get_validator_set_info_of(anchor, U64::from(u64::from(era_number)));
        println!(
            "Validator set info of era {}: {}",
            era_number,
            serde_json::to_string::<ValidatorSetInfo>(&validator_set_info).unwrap()
        );
    }
}

pub fn distribute_reward_of(
    root: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    wrapped_appchain_token: &ContractAccount<WrappedAppchainTokenContract>,
    nonce: u32,
    era_number: u32,
    unprofitable_validator_ids: Vec<String>,
    to_confirm_view_result: bool,
) {
    let anchor_balance_of_wat =
        token_viewer::get_wat_balance_of(&anchor.account_id(), &wrapped_appchain_token);
    let mut appchain_messages = Vec::<AppchainMessage>::new();
    appchain_messages.push(AppchainMessage {
        appchain_event: AppchainEvent::EraRewardConcluded {
            era_number,
            unprofitable_validator_ids,
        },
        nonce,
    });
    sudo_actions::stage_appchain_messages(root, anchor, appchain_messages);
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(anchor);
        println!(
            "Anchor status: {}",
            serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
        );
    }
    process_appchain_messages(root, anchor);
    assert_eq!(
        token_viewer::get_wat_balance_of(&anchor.account_id(), &wrapped_appchain_token).0,
        anchor_balance_of_wat.0 + to_oct_amount(10)
    );
    if to_confirm_view_result {
        let anchor_status = anchor_viewer::get_anchor_status(anchor);
        println!(
            "Anchor status: {}",
            serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
        );
        let validator_set_info =
            anchor_viewer::get_validator_set_info_of(anchor, U64::from(u64::from(era_number)));
        println!(
            "Validator set info of era {}: {}",
            era_number,
            serde_json::to_string::<ValidatorSetInfo>(&validator_set_info).unwrap()
        );
        print_anchor_events(&anchor);
        print_appchain_notifications(&anchor);
    }
}

pub fn withdraw_validator_rewards_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    user: &UserAccount,
    wrapped_appchain_token: &ContractAccount<WrappedAppchainTokenContract>,
    end_era: u64,
) {
    print_wat_balance_of_anchor(anchor, wrapped_appchain_token);
    let wat_balance_before_withdraw =
        token_viewer::get_wat_balance_of(&user.account_id(), wrapped_appchain_token);
    let result = staking_actions::withdraw_validator_rewards(user, anchor, &user.account_id());
    result.assert_success();
    println!(
        "User '{}' withdrawed rewards: {}",
        &user.account_id(),
        token_viewer::get_wat_balance_of(&user.account_id(), wrapped_appchain_token).0
            - wat_balance_before_withdraw.0
    );
    print_validator_reward_histories(anchor, user, end_era);
}

pub fn withdraw_delegator_rewards_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    user: &UserAccount,
    validator: &UserAccount,
    wrapped_appchain_token: &ContractAccount<WrappedAppchainTokenContract>,
    end_era: u64,
) {
    print_wat_balance_of_anchor(anchor, wrapped_appchain_token);
    let wat_balance_before_withdraw =
        token_viewer::get_wat_balance_of(&user.account_id(), wrapped_appchain_token);
    let result = staking_actions::withdraw_delegator_rewards(
        user,
        anchor,
        &user.account_id(),
        &validator.account_id(),
    );
    result.assert_success();
    println!(
        "User '{}' withdrawed delegator rewards: {}",
        &user.account_id(),
        token_viewer::get_wat_balance_of(&user.account_id(), wrapped_appchain_token).0
            - wat_balance_before_withdraw.0
    );
    print_delegator_reward_histories(anchor, user, validator, end_era);
}

pub fn withdraw_stake_of(
    anchor: &ContractAccount<AppchainAnchorContract>,
    user: &UserAccount,
    oct_token: &ContractAccount<MockOctTokenContract>,
) {
    let oct_balance_before_withdraw = token_viewer::get_oct_balance_of(&user, oct_token);
    let result = staking_actions::withdraw_stake(user, anchor, &user.account_id());
    result.assert_success();
    println!(
        "User '{}' withdrawed stake: {}",
        &user.account_id(),
        token_viewer::get_oct_balance_of(user, oct_token).0 - oct_balance_before_withdraw.0
    );
    print_unbonded_stakes_of(anchor, user);
}
