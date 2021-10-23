use appchain_anchor::AppchainAnchorContract;
use mock_appchain_registry::MockAppchainRegistryContract;
use mock_oct_token::MockOctTokenContract;
use mock_wrapped_appchain_token::{MockWrappedAppchainToken, MockWrappedAppchainTokenContract};
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};

use near_sdk::{json_types::U128, serde_json, Balance};
use near_sdk_sim::{
    call, deploy, init_simulator, lazy_static_include, runtime::GenesisConfig, to_yocto, view,
    ContractAccount, ExecutionResult, UserAccount,
};

use num_format::{Locale, ToFormattedString};

const INIT_DEPOSIT_FOR_CONTRACT: Balance = 30_000_000_000_000_000_000_000_000;

lazy_static_include::lazy_static_include_bytes! {
    TOKEN_WASM_BYTES => "../res/mock_oct_token.wasm",
    REGISTRY_WASM_BYTES => "../res/mock_appchain_registry.wasm",
    ANCHOR_WASM_BYTES => "../res/appchain_anchor.wasm",
    WAT_WASM_BYTES => "../res/mock_wrapped_appchain_token.wasm"
}

// Register the given `user` to oct_token
fn register_user_to_oct_token(
    account: &UserAccount,
    contract: &ContractAccount<MockOctTokenContract>,
) {
    let outcome = call!(
        account,
        contract.storage_deposit(Option::from(account.valid_account_id()), Option::None),
        near_sdk::env::storage_byte_cost() * 125,
        near_sdk_sim::DEFAULT_GAS / 2
    );
    outcome.assert_success();
}

fn register_user_to_wat_token(
    account: &UserAccount,
    contract: &ContractAccount<MockWrappedAppchainTokenContract>,
) {
    let outcome = call!(
        account,
        contract.storage_deposit(Option::from(account.valid_account_id()), Option::None),
        near_sdk::env::storage_byte_cost() * 125,
        near_sdk_sim::DEFAULT_GAS / 2
    );
    outcome.assert_success();
}

pub fn ft_transfer_oct_token(
    sender: &UserAccount,
    receiver: &UserAccount,
    amount: u128,
    oct_token: &ContractAccount<MockOctTokenContract>,
) {
    let outcome = call!(
        sender,
        oct_token.ft_transfer(
            receiver.valid_account_id(),
            U128::from(amount),
            Option::None
        ),
        1,
        near_sdk_sim::DEFAULT_GAS
    );
    outcome.assert_success();
}

pub fn ft_transfer_call_oct_token(
    sender: &UserAccount,
    receiver: &UserAccount,
    amount: u128,
    msg: String,
    oct_token: &ContractAccount<MockOctTokenContract>,
) -> ExecutionResult {
    let outcome = call!(
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
    outcome.assert_success();
    outcome
}

fn get_genesis_config() -> GenesisConfig {
    let mut genesis_config = GenesisConfig::default();
    genesis_config.block_prod_time = 86400 * 1_000_000_000;
    genesis_config
}

pub fn init(
    total_supply: u128,
) -> (
    UserAccount,
    ContractAccount<MockOctTokenContract>,
    ContractAccount<MockAppchainRegistryContract>,
    ContractAccount<AppchainAnchorContract>,
    ContractAccount<MockWrappedAppchainTokenContract>,
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
    let wat_ft_metadata = FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.to_string(),
        name: "WrappedAppchainToken".to_string(),
        symbol: "WAT".to_string(),
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
    let anchor = deploy! {
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
    };
    let wrapped_appchain_token = deploy! {
        contract: MockWrappedAppchainTokenContract,
        contract_id: "wrapped_appchain_token",
        bytes: &WAT_WASM_BYTES,
        signer_account: root,
        init_method: new(
            anchor.valid_account_id(),
            root.valid_account_id(),
            U128::from(total_supply / 2),
            wat_ft_metadata
        )
    };
    register_user_to_oct_token(&registry.user_account, &oct_token);
    register_user_to_oct_token(&anchor.user_account, &oct_token);
    // Create users and transfer a certain amount of OCT token to them
    let alice = root.create_user("alice".to_string(), to_yocto("100"));
    register_user_to_oct_token(&alice, &oct_token);
    register_user_to_wat_token(&alice, &wrapped_appchain_token);
    ft_transfer_oct_token(&root, &alice, total_supply / 10, &oct_token);
    users.push(alice);
    let bob = root.create_user("bob".to_string(), to_yocto("100"));
    register_user_to_oct_token(&bob, &oct_token);
    register_user_to_wat_token(&bob, &wrapped_appchain_token);
    ft_transfer_oct_token(&root, &bob, total_supply / 10, &oct_token);
    users.push(bob);
    let charlie = root.create_user("charlie".to_string(), to_yocto("100"));
    register_user_to_oct_token(&charlie, &oct_token);
    register_user_to_wat_token(&charlie, &wrapped_appchain_token);
    ft_transfer_oct_token(&root, &charlie, total_supply / 10, &oct_token);
    users.push(charlie);
    let dave = root.create_user("dave".to_string(), to_yocto("100"));
    register_user_to_oct_token(&dave, &oct_token);
    register_user_to_wat_token(&dave, &wrapped_appchain_token);
    ft_transfer_oct_token(&root, &dave, total_supply / 10, &oct_token);
    users.push(dave);
    let eve = root.create_user("eve".to_string(), to_yocto("100"));
    register_user_to_oct_token(&eve, &oct_token);
    register_user_to_wat_token(&eve, &wrapped_appchain_token);
    ft_transfer_oct_token(&root, &eve, total_supply / 10, &oct_token);
    users.push(eve);
    // Print initial storage balance of anchor
    print_anchor_storage_balance(&anchor);
    // Return initialized UserAccounts
    (
        root,
        oct_token,
        registry,
        anchor,
        wrapped_appchain_token,
        users,
    )
}

pub fn to_oct_amount(amount: u128) -> u128 {
    let bt_decimals_base = (10 as u128).pow(18);
    amount * bt_decimals_base
}

pub fn print_execution_result(
    anchor: &ContractAccount<AppchainAnchorContract>,
    function_name: &str,
    result: &ExecutionResult,
) {
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
        print_anchor_storage_balance(anchor);
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
