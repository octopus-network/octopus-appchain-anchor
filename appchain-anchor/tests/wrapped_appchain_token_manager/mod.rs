use appchain_anchor::{AppchainAnchorContract, AppchainMessage};
use mock_wrapped_appchain_token::MockWrappedAppchainTokenContract;
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn set_account_of_wrapped_appchain_token(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    wat_account: &ContractAccount<MockWrappedAppchainTokenContract>,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.set_account_of_wrapped_appchain_token(wat_account.valid_account_id().to_string())
    );
    common::print_execution_result(anchor, "apply_appchain_message", &result);
    result
}
