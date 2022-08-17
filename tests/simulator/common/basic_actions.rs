use appchain_anchor::types::AppchainTemplateType;
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_sdk::json_types::Base64VecU8;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_units::parse_near;
use workspaces::network::Sandbox;
use workspaces::{Account, Contract, Worker};

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
    Vec<Account>,
)> {
    let root = worker.root_account();
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
        .create_subaccount(worker, "oct_token")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    let oct_token = oct_token
        .deploy(&worker, &std::fs::read(format!("res/mock_oct_token.wasm"))?)
        .await?
        .unwrap();
    oct_token
        .call(worker, "new")
        .args_json(json!({
            "owner_id": root.id(),
            "total_supply": U128::from(total_supply),
            "metadata": oct_ft_metadata
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    //
    // deploy appchain registry contract
    //
    let appchain_registry = root
        .create_subaccount(worker, "appchain_registry")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    let appchain_registry = appchain_registry
        .deploy(
            worker,
            &std::fs::read(format!("res/mock_appchain_registry.wasm"))?,
        )
        .await?
        .unwrap();
    appchain_registry
        .call(worker, "new")
        .args_json(json!({
            "oct_token": oct_token.id()
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    //
    // deploy appchain anchor contract
    //
    let appchain_anchor = root
        .create_subaccount(worker, "appchain_anchor")
        .initial_balance(parse_near!("100 N"))
        .transact()
        .await?
        .unwrap();
    let appchain_anchor = match with_old_anchor {
        true => appchain_anchor
            .deploy(
                worker,
                &std::fs::read(format!("res/appchain_anchor_v2.1.1.wasm"))?,
            )
            .await?
            .unwrap(),
        false => appchain_anchor
            .deploy(worker, &std::fs::read(format!("res/appchain_anchor.wasm"))?)
            .await?
            .unwrap(),
    };
    match with_old_anchor {
        true => {
            root.call(worker, appchain_anchor.id(), "new")
                .args_json(json!({
                    "appchain_id": "test_appchain_id".to_string(),
                    "appchain_registry": appchain_registry.id(),
                    "oct_token": oct_token.id(),
                }))?
                .gas(300_000_000_000_000)
                .transact()
                .await?
        }
        false => {
            root.call(worker, appchain_anchor.id(), "new")
                .args_json(json!({
                    "appchain_id": "test_appchain_id".to_string(),
                    "appchain_template_type": AppchainTemplateType::Barnacle,
                    "appchain_registry": appchain_registry.id(),
                    "oct_token": oct_token.id(),
                }))?
                .gas(300_000_000_000_000)
                .transact()
                .await?
        }
    };
    //
    // deploy wrapped appchain token faucet contract
    //
    let wat_faucet = appchain_anchor
        .as_account()
        .create_subaccount(worker, "wat-faucet")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    let wat_faucet = wat_faucet
        .deploy(worker, &std::fs::read(format!("res/wat_faucet.wasm"))?)
        .await?
        .unwrap();
    wat_faucet
        .call(worker, "new")
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    //
    // initialize users' accounts
    //
    register_user_to_ft_contract(worker, appchain_registry.as_account(), &oct_token).await?;
    register_user_to_ft_contract(worker, appchain_anchor.as_account(), &oct_token).await?;
    // Create users and transfer a certain amount of OCT token to them
    // alice
    let alice = root
        .create_subaccount(worker, "alice")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    register_user_to_ft_contract(worker, &alice, &oct_token).await?;
    super::call_ft_transfer(worker, &root, &alice, total_supply / 10, &oct_token).await?;
    users.push(alice);
    // bob
    let bob = root
        .create_subaccount(worker, "bob")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    register_user_to_ft_contract(worker, &bob, &oct_token).await?;
    super::call_ft_transfer(worker, &root, &bob, total_supply / 10, &oct_token).await?;
    users.push(bob);
    // charlie
    let charlie = root
        .create_subaccount(worker, "charlie")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    register_user_to_ft_contract(worker, &charlie, &oct_token).await?;
    super::call_ft_transfer(worker, &root, &charlie, total_supply / 10, &oct_token).await?;
    users.push(charlie);
    // dave
    let dave = root
        .create_subaccount(worker, "dave")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    register_user_to_ft_contract(worker, &dave, &oct_token).await?;
    super::call_ft_transfer(worker, &root, &dave, total_supply / 10, &oct_token).await?;
    users.push(dave);
    // eve
    let eve = root
        .create_subaccount(worker, "eve")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    register_user_to_ft_contract(worker, &eve, &oct_token).await?;
    super::call_ft_transfer(worker, &root, &eve, total_supply / 10, &oct_token).await?;
    users.push(eve);
    // Return initialized UserAccounts
    Ok((
        root,
        oct_token,
        appchain_registry,
        appchain_anchor,
        wat_faucet,
        users,
    ))
}

pub async fn deploy_wrapped_appchain_token_contract(
    worker: &Worker<Sandbox>,
    root: &Account,
    anchor: &Contract,
    premined_beneficiary: &Account,
    premined_balance: &U128,
    users: &Vec<Account>,
) -> anyhow::Result<Contract> {
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
        .create_subaccount(worker, "wrapped_appchain_token")
        .initial_balance(parse_near!("50 N"))
        .transact()
        .await?
        .unwrap();
    let wrapped_appchain_token = wrapped_appchain_token
        .deploy(
            worker,
            &std::fs::read(format!("res/wrapped_appchain_token.wasm"))?,
        )
        .await?
        .unwrap();
    wrapped_appchain_token
        .call(worker, "new")
        .args_json(json!({
            "owner_id": anchor.id(),
            "premined_beneficiary": premined_beneficiary.id(),
            "premined_balance": premined_balance,
            "metadata": wat_ft_metadata,
        }))?
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    for user in users {
        register_user_to_ft_contract(worker, &user, &wrapped_appchain_token).await?;
    }
    Ok(wrapped_appchain_token)
}

// Register the given `user` to fungible token contract
pub async fn register_user_to_ft_contract(
    worker: &Worker<Sandbox>,
    account: &Account,
    ft_token_contract: &Contract,
) -> anyhow::Result<()> {
    ft_token_contract
        .call(worker, "storage_deposit")
        .args_json(json!({
            "account_id": Some(account.id()),
            "registration_only": Option::<bool>::None,
        }))?
        .gas(20_000_000_000_000)
        .deposit(near_sdk::env::storage_byte_cost() * 125)
        .transact()
        .await?;
    Ok(())
}
