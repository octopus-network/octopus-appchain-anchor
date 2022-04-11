use appchain_anchor::AppchainAnchorContract;
use near_contract_standards::non_fungible_token::metadata::NFTContractMetadata;
use near_sdk_sim::{call, ContractAccount, ExecutionResult, UserAccount};

use crate::common;

pub fn register_wrapped_appchain_nft(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    class_id: String,
    metadata: NFTContractMetadata,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.register_wrapped_appchain_nft(class_id, metadata)
    );
    common::print_execution_result("register_wrapped_appchain_nft", &result);
    result
}

pub fn change_wrapped_appchain_nft_contract_metadata(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    class_id: String,
    metadata: NFTContractMetadata,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.change_wrapped_appchain_nft_contract_metadata(class_id, metadata)
    );
    common::print_execution_result("change_wrapped_appchain_nft_contract_metadata", &result);
    result
}

pub fn open_bridging_of_wrapped_appchain_nft(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    class_id: String,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.open_bridging_of_wrapped_appchain_nft(class_id)
    );
    common::print_execution_result("open_bridging_of_wrapped_appchain_nft", &result);
    result
}

pub fn close_bridging_of_wrapped_appchain_nft(
    signer: &UserAccount,
    anchor: &ContractAccount<AppchainAnchorContract>,
    class_id: String,
) -> ExecutionResult {
    let result = call!(
        signer,
        anchor.close_bridging_of_wrapped_appchain_nft(class_id)
    );
    common::print_execution_result("close_bridging_of_wrapped_appchain_nft", &result);
    result
}
