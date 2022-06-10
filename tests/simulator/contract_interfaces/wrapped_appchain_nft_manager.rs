use near_contract_standards::non_fungible_token::metadata::NFTContractMetadata;
use near_sdk::serde_json::json;
use workspaces::{network::Sandbox, result::CallExecutionDetails, Account, Contract, Worker};

pub async fn register_wrapped_appchain_nft(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    class_id: String,
    metadata: NFTContractMetadata,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "register_wrapped_appchain_nft")
        .args_json(json!({
            "class_id": class_id,
            "metadata": metadata
        }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}

pub async fn open_bridging_of_wrapped_appchain_nft(
    worker: &Worker<Sandbox>,
    signer: &Account,
    anchor: &Contract,
    class_id: String,
) -> anyhow::Result<CallExecutionDetails> {
    signer
        .call(worker, anchor.id(), "open_bridging_of_wrapped_appchain_nft")
        .args_json(json!({ "class_id": class_id }))?
        .gas(200_000_000_000_000)
        .transact()
        .await
}
