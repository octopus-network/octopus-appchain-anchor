use appchain_anchor::types::AppchainTemplateType;
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_sdk::json_types::Base64VecU8;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_units::parse_near;
use workspaces::network::Sandbox;
use workspaces::{error::Error, Account, Contract, Worker};

const TEST_APPCHAIN_ID: &str = "test-appchain";

pub async fn initialize_contracts_and_users(
    worker: &Worker<Sandbox>,
    total_supply: u128,
    with_old_anchor: bool,
) -> anyhow::Result<(
    Account,
    Contract,
    Contract,
    Contract,
    Contract,
    Contract,
    Vec<Account>,
)> {
    let root = worker.root_account().unwrap();
    let mut users: Vec<Account> = Vec::new();
    //
    // deploy OCT token contract
    //
    let oct_ft_metadata = FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.to_string(),
        name: "OCTToken".to_string(),
        symbol: "OCT".to_string(),
        icon: Option::<String>::None,
        reference: Option::<String>::None,
        reference_hash: Option::<Base64VecU8>::None,
        decimals: 18,
    };
    let oct_token = root
        .create_subaccount("oct-token")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let oct_token = oct_token
        .deploy(&std::fs::read(format!("res/mock_oct_token.wasm"))?)
        .await?
        .unwrap();
    assert!(oct_token
        .call("new")
        .args_json(json!({
            "owner_id": root.id(),
            "total_supply": U128::from(total_supply),
            "metadata": oct_ft_metadata
        }))
        .gas(300_000_000_000_000)
        .transact()
        .await
        .unwrap()
        .is_success());
    //
    // deploy appchain registry contract
    //
    let appchain_registry = root
        .create_subaccount("appchain-registry")
        .initial_balance(parse_near!("100 N"))
        .transact()
        .await?
        .unwrap();
    let appchain_registry = appchain_registry
        .deploy(&std::fs::read(format!("res/mock_appchain_registry.wasm"))?)
        .await?
        .unwrap();
    assert!(appchain_registry
        .call("new")
        .args_json(json!({
            "oct_token": oct_token.id()
        }))
        .gas(300_000_000_000_000)
        .transact()
        .await
        .unwrap()
        .is_success());
    //
    // create octopus dao account
    //
    let octopus_dao = root
        .create_subaccount("octopus-dao")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    //
    // deploy council keeper contract
    //
    let council_keeper = appchain_registry
        .as_account()
        .create_subaccount("council-keeper")
        .initial_balance(parse_near!("10 N"))
        .transact()
        .await?
        .unwrap();
    let council_keeper = council_keeper
        .deploy(&std::fs::read(format!("res/council_keeper.wasm"))?)
        .await?
        .unwrap();
    assert!(council_keeper
        .call("new")
        .args_json(json!({
            "max_number_of_council_members": 3,
            "dao_contract_account": octopus_dao.id().to_string(),
        }))
        .gas(300_000_000_000_000)
        .transact()
        .await
        .unwrap()
        .is_success());
    //
    // deploy appchain anchor contract
    //
    let appchain_anchor = appchain_registry
        .as_account()
        .create_subaccount(TEST_APPCHAIN_ID)
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    let appchain_anchor = match with_old_anchor {
        true => appchain_anchor
            .deploy(&std::fs::read(format!("res/appchain_anchor_v2.4.1.wasm"))?)
            .await?
            .unwrap(),
        false => appchain_anchor
            .deploy(&std::fs::read(format!("res/appchain_anchor.wasm"))?)
            .await?
            .unwrap(),
    };
    match with_old_anchor {
        true => {
            assert!(root
                .call(appchain_anchor.id(), "new")
                .args_json(json!({
                    "appchain_id": TEST_APPCHAIN_ID.to_string(),
                    "appchain_template_type": AppchainTemplateType::Barnacle,
                    "appchain_registry": appchain_registry.id(),
                    "oct_token": oct_token.id(),
                }))
                .gas(300_000_000_000_000)
                .transact()
                .await
                .unwrap()
                .is_success())
        }
        false => {
            assert!(root
                .call(appchain_anchor.id(), "new")
                .args_json(json!({
                    "appchain_template_type": AppchainTemplateType::Barnacle,
                    "oct_token": oct_token.id(),
                }))
                .gas(300_000_000_000_000)
                .transact()
                .await
                .unwrap()
                .is_success());
        }
    };
    //
    // deploy wrapped appchain token faucet contract
    //
    let wat_faucet = appchain_anchor
        .as_account()
        .create_subaccount("wat-faucet")
        .initial_balance(parse_near!("5 N"))
        .transact()
        .await?
        .unwrap();
    let wat_faucet = wat_faucet
        .deploy(&std::fs::read(format!("res/wat_faucet.wasm"))?)
        .await?
        .unwrap();
    assert!(wat_faucet
        .call("new")
        .gas(300_000_000_000_000)
        .transact()
        .await
        .unwrap()
        .is_success());
    //
    // initialize users' accounts
    //
    register_user_to_ft_contract(appchain_registry.as_account(), &oct_token).await;
    register_user_to_ft_contract(appchain_anchor.as_account(), &oct_token).await;
    // Create users and transfer a certain amount of OCT token to them
    // alice
    let alice = root
        .create_subaccount("alice")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    register_user_to_ft_contract(&alice, &oct_token).await;
    assert!(
        super::call_ft_transfer(&root, &alice, total_supply / 10, &oct_token)
            .await
            .unwrap()
            .is_success()
    );
    users.push(alice);
    // bob
    let bob = root
        .create_subaccount("bob")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    register_user_to_ft_contract(&bob, &oct_token).await;
    assert!(
        super::call_ft_transfer(&root, &bob, total_supply / 10, &oct_token)
            .await
            .unwrap()
            .is_success()
    );
    users.push(bob);
    // charlie
    let charlie = root
        .create_subaccount("charlie")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    register_user_to_ft_contract(&charlie, &oct_token).await;
    assert!(
        super::call_ft_transfer(&root, &charlie, total_supply / 10, &oct_token)
            .await
            .unwrap()
            .is_success()
    );
    users.push(charlie);
    // dave
    let dave = root
        .create_subaccount("dave")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    register_user_to_ft_contract(&dave, &oct_token).await;
    assert!(
        super::call_ft_transfer(&root, &dave, total_supply / 10, &oct_token)
            .await
            .unwrap()
            .is_success()
    );
    users.push(dave);
    // eve
    let eve = root
        .create_subaccount("eve")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    register_user_to_ft_contract(&eve, &oct_token).await;
    assert!(
        super::call_ft_transfer(&root, &eve, total_supply / 10, &oct_token)
            .await
            .unwrap()
            .is_success()
    );
    users.push(eve);
    // relayer
    let relayer = root
        .create_subaccount("relayer")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    users.push(relayer);
    // Return initialized UserAccounts
    Ok((
        root,
        oct_token,
        appchain_registry,
        council_keeper,
        appchain_anchor,
        wat_faucet,
        users,
    ))
}

pub async fn deploy_wrapped_appchain_token_contract(
    root: &Account,
    anchor: &Contract,
    premined_beneficiary: &Account,
    premined_balance: &U128,
    users: &Vec<Account>,
) -> Result<Contract, Error> {
    let wat_ft_metadata = FungibleTokenMetadata {
        spec: FT_METADATA_SPEC.to_string(),
        name: "WrappedAppchainToken".to_string(),
        symbol: "WAT".to_string(),
        icon: None,
        reference: None,
        reference_hash: None,
        decimals: 18,
    };
    let wrapped_appchain_token = root
        .create_subaccount("wrapped_appchain_token")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    let wrapped_appchain_token = wrapped_appchain_token
        .deploy(&std::fs::read(format!("res/wrapped_appchain_token.wasm")).unwrap())
        .await?
        .unwrap();
    assert!(wrapped_appchain_token
        .call("new")
        .args_json(json!({
            "owner_id": anchor.id(),
            "premined_beneficiary": premined_beneficiary.id(),
            "premined_balance": premined_balance,
            "metadata": wat_ft_metadata,
        }))
        .gas(300_000_000_000_000)
        .transact()
        .await
        .unwrap()
        .is_success());
    for user in users {
        register_user_to_ft_contract(&user, &wrapped_appchain_token).await;
    }
    Ok(wrapped_appchain_token)
}

// Register the given `user` to fungible token contract
pub async fn register_user_to_ft_contract(account: &Account, ft_token_contract: &Contract) {
    assert!(ft_token_contract
        .call("storage_deposit")
        .args_json(json!({
            "account_id": Some(account.id()),
            "registration_only": Option::<bool>::None,
        }))
        .gas(20_000_000_000_000)
        .deposit(near_sdk::env::storage_byte_cost() * 125)
        .transact()
        .await
        .unwrap()
        .is_success());
}
