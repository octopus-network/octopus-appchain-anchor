use std::convert::TryInto;

use appchain_anchor::{
    types::{
        AnchorStatus, AppchainCommitment, AppchainMessageProcessingResult,
        MultiTxsOperationProcessingResult, ValidatorProfile, ValidatorSetInfo,
        ValidatorSetProcessingStatus, WrappedAppchainToken,
    },
    AppchainAnchorContract, AppchainEvent, AppchainMessage,
};
use mock_appchain_registry::MockAppchainRegistryContract;
use mock_oct_token::MockOctTokenContract;
use mock_wrapped_appchain_token::MockWrappedAppchainTokenContract;

use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_sdk::{json_types::U128, serde_json, AccountId, Balance};
use near_sdk_sim::{
    call, deploy, init_simulator, lazy_static_include, runtime::GenesisConfig, to_yocto, view,
    ContractAccount, ExecutionResult, UserAccount,
};

use num_format::{Locale, ToFormattedString};

use crate::permissionless_actions;
use crate::sudo_actions;
use crate::{anchor_viewer, token_viewer};

const INIT_DEPOSIT_FOR_CONTRACT: Balance = 30_000_000_000_000_000_000_000_000;

lazy_static_include::lazy_static_include_bytes! {
    TOKEN_WASM_BYTES => "../res/mock_oct_token.wasm",
    REGISTRY_WASM_BYTES => "../res/mock_appchain_registry.wasm",
    ANCHOR_WASM_BYTES => "../res/appchain_anchor.wasm",
    WAT_WASM_BYTES => "../res/mock_wrapped_appchain_token.wasm",
    OLD_ANCHOR_WASM_BYTES => "../res/previous_appchain_anchor.wasm",
}

// Register the given `user` to oct_token
fn register_user_to_oct_token(
    account: &UserAccount,
    contract: &ContractAccount<MockOctTokenContract>,
) {
    let result = call!(
        account,
        contract.storage_deposit(Option::from(account.valid_account_id()), Option::None),
        near_sdk::env::storage_byte_cost() * 125,
        near_sdk_sim::DEFAULT_GAS / 2
    );
    print_execution_result("register_user_to_oct_token", &result);
    result.assert_success();
}

fn register_user_to_wat_token(
    account: &UserAccount,
    contract: &ContractAccount<MockWrappedAppchainTokenContract>,
) {
    let result = call!(
        account,
        contract.storage_deposit(Option::from(account.valid_account_id()), Option::None),
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
        oct_token.ft_transfer(
            receiver.valid_account_id(),
            U128::from(amount),
            Option::None
        ),
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
            receiver.valid_account_id(),
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
        init_method: new(root.valid_account_id(), U128::from(total_supply), oct_ft_metadata)
    };
    let registry = deploy! {
        contract: MockAppchainRegistryContract,
        contract_id: "registry",
        bytes: &REGISTRY_WASM_BYTES,
        signer_account: root,
        init_method: new(oct_token.valid_account_id().to_string())
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
                registry.valid_account_id().to_string(),
                oct_token.valid_account_id().to_string()
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
                registry.valid_account_id().to_string(),
                oct_token.valid_account_id().to_string()
            )
        },
    };
    register_user_to_oct_token(&registry.user_account, &oct_token);
    register_user_to_oct_token(&anchor.user_account, &oct_token);
    // Create users and transfer a certain amount of OCT token to them
    let alice = root.create_user("alice".to_string(), to_yocto("100"));
    register_user_to_oct_token(&alice, &oct_token);
    ft_transfer_oct_token(&root, &alice, total_supply / 10, &oct_token);
    users.push(alice);
    let bob = root.create_user("bob".to_string(), to_yocto("100"));
    register_user_to_oct_token(&bob, &oct_token);
    ft_transfer_oct_token(&root, &bob, total_supply / 10, &oct_token);
    users.push(bob);
    let charlie = root.create_user("charlie".to_string(), to_yocto("100"));
    register_user_to_oct_token(&charlie, &oct_token);
    ft_transfer_oct_token(&root, &charlie, total_supply / 10, &oct_token);
    users.push(charlie);
    let dave = root.create_user("dave".to_string(), to_yocto("100"));
    register_user_to_oct_token(&dave, &oct_token);
    ft_transfer_oct_token(&root, &dave, total_supply / 10, &oct_token);
    users.push(dave);
    let eve = root.create_user("eve".to_string(), to_yocto("100"));
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
) -> ContractAccount<MockWrappedAppchainTokenContract> {
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
        contract: MockWrappedAppchainTokenContract,
        contract_id: "wrapped_appchain_token",
        bytes: &WAT_WASM_BYTES,
        signer_account: root,
        init_method: new(
            anchor.valid_account_id(),
            users[0].valid_account_id(),
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

pub fn print_execution_result(function_name: &str, result: &ExecutionResult) {
    println!(
        "Gas burnt of function '{}': {}",
        function_name,
        result.gas_burnt().to_formatted_string(&Locale::en)
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
    if result.is_ok() {
        let logs = result.logs();
        if logs.len() > 0 {
            println!("{:#?}", logs);
        }
    } else {
        println!("{:#?}", result.outcome());
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

pub fn print_wrapped_appchain_token_info(anchor: &ContractAccount<AppchainAnchorContract>) {
    let wrapped_appchain_token_info = anchor_viewer::get_wrapped_appchain_token(&anchor);
    println!(
        "Wrapped appchain token: {}",
        serde_json::to_string::<WrappedAppchainToken>(&wrapped_appchain_token_info).unwrap()
    );
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
            validator.valid_account_id().to_string(),
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
            user.valid_account_id().to_string(),
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
    wrapped_appchain_token: &ContractAccount<MockWrappedAppchainTokenContract>,
) {
    let wat_balance_of_anchor =
        token_viewer::get_wat_balance_of(&anchor.valid_account_id(), wrapped_appchain_token);
    println!(
        "Wrapped appchain token balance of anchor contract: {}",
        wat_balance_of_anchor.0
    );
}

pub fn switch_era(
    root: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    era_number: u32,
) {
    if era_number > 0 {
        let mut appchain_messages = Vec::<AppchainMessage>::new();
        appchain_messages.push(AppchainMessage {
            appchain_event: AppchainEvent::EraSwitchPlaned { era_number },
            nonce: (era_number + 1).try_into().unwrap(),
        });
        let results = sudo_actions::apply_appchain_messages(root, anchor, appchain_messages);
        for result in results {
            println!(
                "Appchain message processing result: {}",
                serde_json::to_string::<AppchainMessageProcessingResult>(&result).unwrap()
            )
        }
        let processing_status =
            anchor_viewer::get_processing_status_of(anchor, u64::from(era_number));
        println!(
            "Processing status of era {}: {}",
            era_number,
            serde_json::to_string::<ValidatorSetProcessingStatus>(&processing_status).unwrap()
        );
    }
    loop {
        let result = permissionless_actions::try_complete_switching_era(root, &anchor);
        println!(
            "Try complete switching era: {}",
            serde_json::to_string::<MultiTxsOperationProcessingResult>(&result).unwrap()
        );
        let processing_status =
            anchor_viewer::get_processing_status_of(anchor, u64::from(era_number));
        println!(
            "Processing status of era {}: {}",
            era_number,
            serde_json::to_string::<ValidatorSetProcessingStatus>(&processing_status).unwrap()
        );
        if result.eq(&MultiTxsOperationProcessingResult::Ok) {
            break;
        }
    }
    let anchor_status = anchor_viewer::get_anchor_status(anchor);
    println!(
        "Anchor status: {}",
        serde_json::to_string::<AnchorStatus>(&anchor_status).unwrap()
    );
    let validator_set_info =
        anchor_viewer::get_validator_set_info_of(anchor, u64::from(era_number));
    println!(
        "Validator set info of era {}: {}",
        era_number,
        serde_json::to_string::<ValidatorSetInfo>(&validator_set_info).unwrap()
    );
}
