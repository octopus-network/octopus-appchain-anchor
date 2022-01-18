use mock_oct_token::MockOctTokenContract;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk_sim::{view, ContractAccount, UserAccount};
use wrapped_appchain_token::WrappedAppchainTokenContract;

pub fn get_oct_balance_of(
    user: &UserAccount,
    oct_token: &ContractAccount<MockOctTokenContract>,
) -> U128 {
    let view_result = view!(oct_token.ft_balance_of(user.valid_account_id()));
    if view_result.is_err() {
        println!("{:#?}", view_result);
    }
    assert!(view_result.is_ok());
    view_result.unwrap_json::<U128>()
}

pub fn get_wat_balance_of(
    account_id: &ValidAccountId,
    wat_token: &ContractAccount<WrappedAppchainTokenContract>,
) -> U128 {
    let view_result = view!(wat_token.ft_balance_of(account_id.clone()));
    if view_result.is_err() {
        println!("{:#?}", view_result);
    }
    assert!(view_result.is_ok());
    view_result.unwrap_json::<U128>()
}
