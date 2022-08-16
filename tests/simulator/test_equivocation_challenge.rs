use crate::{common, contract_interfaces::anchor_viewer};
use appchain_anchor::appchain_challenge::AppchainChallenge;
use near_sdk::serde_json::{self, json};

#[tokio::test]
async fn test_equivocation_challenge() -> anyhow::Result<()> {
    //
    let worker = workspaces::sandbox().await?;
    let (_root, _, _, _, anchor, _wat_faucet, users, _) =
        common::test_normal_actions(&worker, false, false, vec!["0x00".to_string()]).await?;
    //
    if let Ok(challenge) = serde_json::from_str::<AppchainChallenge>("{\"EquivocationChallenge\":{\"submitter_account\":\"tt.testnet\",\"proof\":{\"set_id\":0,\"equivocation\":{\"Prevote\":{\"round_number\":2,\"identity\":[209,124,45,120,35,235,242,96,253,19,143,45,126,39,209,20,192,20,93,150,139,95,245,0,97,37,242,65,79,173,174,105],\"first\":[{\"target_hash\":[96,43,40,162,148,136,214,36,237,150,130,159,164,176,134,217,188,7,156,28,26,245,153,173,235,220,148,113,142,54,86,172],\"target_number\":2},[160,8,125,180,57,254,58,164,29,247,251,21,218,43,228,81,110,42,54,245,139,100,113,120,8,169,186,72,79,1,10,44,124,214,240,57,158,28,5,246,112,141,249,88,85,136,172,109,27,246,217,212,175,90,35,66,230,60,7,116,132,238,222,10]],\"second\":[{\"target_hash\":[55,65,109,173,2,87,56,21,245,65,225,251,11,255,55,219,64,83,133,115,5,161,227,232,204,172,40,117,127,126,63,225],\"target_number\":1},[182,116,59,76,20,131,229,152,169,61,221,96,84,126,111,231,69,122,21,132,2,242,18,172,118,22,204,130,230,203,228,28,91,196,141,105,180,223,209,205,3,210,217,106,135,148,174,214,169,196,82,106,255,89,109,197,73,142,237,71,179,42,184,7]]}}}}}") {
        let result = users[3]
            .call(&worker, anchor.id(), "commit_appchain_challenge")
            .args_json(json!({
                "appchain_challenge": challenge
            }))?
            .gas(300_000_000_000_000)
            .transact()
            .await?;
        assert!(result.is_success());
        //
        let appchain_challenge = anchor_viewer::get_appchain_challenge(&worker, &anchor, 0).await?;
        println!(
            "Appchain challenge 0: {}",
            serde_json::to_string(&appchain_challenge).unwrap()
        );
    //
        let appchain_challenges = anchor_viewer::get_appchain_challenges(&worker, &anchor, 0, None).await?;
        let mut index = 0;
        for appchain_challenge in appchain_challenges {
            println!(
                "Appchain challenge {}: {}",
                index,
                serde_json::to_string(&appchain_challenge).unwrap()
            );
            index += 1;
        }
    } else {
        panic!("Wrong testing data for equivocation challenge.");
    }
    Ok(())
}
