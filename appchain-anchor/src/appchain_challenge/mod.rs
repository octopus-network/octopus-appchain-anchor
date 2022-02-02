mod equivocation_challenge;

use crate::*;

use self::equivocation_challenge::EquivocationProof;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum AppchainChallenge {
    EquivocationChallenge {
        submitter_account: AccountId,
        proof: EquivocationProof,
    },
}
