use appchain_anchor::types::MultiTxsOperationProcessingResult;
use near_sdk::serde_json::json;
use workspaces::{error::Error, result::ExecutionFinalResult, Account, Contract};

pub async fn stage_and_apply_appchain_messages(
    signer: &Account,
    anchor: &Contract,
    encoded_messages: Vec<u8>,
    relayer_tee_signature: Option<Vec<u8>>,
) -> Result<ExecutionFinalResult, Error> {
    let result = signer
        .call(anchor.id(), "stage_and_apply_appchain_messages")
        .gas(300_000_000_000_000)
        .args_json(json!({
            "encoded_messages": encoded_messages,
            "relayer_tee_signature": relayer_tee_signature,
        }))
        .transact()
        .await;
    println!("{:?}", result);
    println!();
    result
}

pub async fn process_appchain_messages(
    signer: &Account,
    anchor: &Contract,
) -> Result<MultiTxsOperationProcessingResult, Error> {
    let result = signer
        .call(anchor.id(), "process_appchain_messages")
        .gas(300_000_000_000_000)
        .transact()
        .await?;
    println!("{:?}", result);
    println!();
    result.json::<MultiTxsOperationProcessingResult>()
}
