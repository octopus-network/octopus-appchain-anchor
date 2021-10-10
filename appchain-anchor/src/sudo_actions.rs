use crate::*;

pub trait SudoActions {
    /// Apply a certain `AppchainMessage`
    fn apply_appchain_message(&mut self, appchain_message: AppchainMessage);
}

#[near_bindgen]
impl SudoActions for AppchainAnchor {
    //
    fn apply_appchain_message(&mut self, appchain_message: AppchainMessage) {
        match appchain_message.appchain_event {
            permissionless_actions::AppchainEvent::NearFungibleTokenBurnt { symbol, amount } => {
                todo!()
            }
            permissionless_actions::AppchainEvent::NativeTokenLocked { amount } => todo!(),
            permissionless_actions::AppchainEvent::EraSwitchPlaned { era_number } => {
                self.assert_owner();
                self.start_switching_era(era_number.0);
            }
            permissionless_actions::AppchainEvent::EraRewardConcluded {
                era_number,
                unprofitable_validator_ids,
            } => {
                self.assert_owner();
                self.start_distributing_reward_of_era(era_number.0, unprofitable_validator_ids);
            }
            permissionless_actions::AppchainEvent::EraRewardChanged {
                era_number,
                era_reward,
            } => todo!(),
        }
    }
}
