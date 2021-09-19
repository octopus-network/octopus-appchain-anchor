use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};

use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk_sim::{
    deploy, init_simulator, lazy_static_include, to_yocto, ContractAccount, UserAccount,
    DEFAULT_GAS, STORAGE_AMOUNT,
};

// Load in contract bytes at runtime
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    NEP141_WASM_BYTES => "res/test_nep141_token.wasm",
    ANCHOR_WASM_BYTES => "res/appchain_anchor.wasm",
}

const OCT_ID: &str = "oct_token";
const NEP_TOKEN_ID: &str = "nep_token";
const ANCHOR_ID: &str = "appchain_anchor";
const OCT_DECIMALS: u8 = 18;
const NEP_DECIMALS: u8 = 24;

// Register the given `user` with oct_token
pub fn register_user(user: &near_sdk_sim::UserAccount) {
    user.call(
        OCT_ID.to_string(),
        "storage_deposit",
        &json!({
            "account_id": user.valid_account_id()
        })
        .to_string()
        .into_bytes(),
        near_sdk_sim::DEFAULT_GAS,
        near_sdk::env::storage_byte_cost() * 125,
    )
    .assert_success();
    user.call(
        NEP_TOKEN_ID.to_string(),
        "storage_deposit",
        &json!({
            "account_id": user.valid_account_id()
        })
        .to_string()
        .into_bytes(),
        near_sdk_sim::DEFAULT_GAS,
        near_sdk::env::storage_byte_cost() * 125,
    )
    .assert_success();
}

pub fn init(
    initial_balance: u128,
    appchain_minimum_validators: u32,
    minimum_staking_amount: u128,
) -> (
    UserAccount,
    UserAccount,
    UserAccount,
    UserAccount,
    UserAccount,
) {
    let root = init_simulator(None);

    let oct = root.deploy(&NEP141_WASM_BYTES, OCT_ID.into(), 10 * STORAGE_AMOUNT);
    let nep_token = root.deploy(&NEP141_WASM_BYTES, NEP_TOKEN_ID.into(), 10 * STORAGE_AMOUNT);
    let anchor = root.deploy(&ANCHOR_WASM_BYTES, ANCHOR_ID.into(), 10 * STORAGE_AMOUNT);

    oct.call(
        OCT_ID.into(),
        "new",
        &json!({
            "owner_id": root.valid_account_id(),
            "total_supply": U128::from(initial_balance),
            "metadata": FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "OCTToken".to_string(),
                symbol: "OCT".to_string(),
                icon: None,
                reference: None,
                reference_hash: None,
                decimals: OCT_DECIMALS,
            }
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        0,
    )
    .assert_success();

    nep_token
        .call(
            NEP_TOKEN_ID.into(),
            "new",
            &json!({
                "owner_id": root.valid_account_id(),
                "total_supply": U128::from(initial_balance),
                "metadata": FungibleTokenMetadata {
                    spec: FT_METADATA_SPEC.to_string(),
                    name: "NEPToken".to_string(),
                    symbol: "NEP".to_string(),
                    icon: None,
                    reference: None,
                    reference_hash: None,
                    decimals: NEP_DECIMALS,
                }
            })
            .to_string()
            .into_bytes(),
            DEFAULT_GAS,
            0,
        )
        .assert_success();

    anchor
        .call(
            ANCHOR_ID.into(),
            "new",
            &json!({}).to_string().into_bytes(),
            DEFAULT_GAS,
            0,
        )
        .assert_success();

    let alice = root.create_user("alice".to_string(), to_yocto("100"));
    register_user(&alice);

    root.call(
        OCT_ID.into(),
        "ft_transfer",
        &json!({
            "receiver_id": alice.valid_account_id(),
            "amount": U128::from(initial_balance / 10),
        })
        .to_string()
        .into_bytes(),
        DEFAULT_GAS,
        1,
    )
    .assert_success();

    (root, oct, nep_token, anchor, alice)
}
