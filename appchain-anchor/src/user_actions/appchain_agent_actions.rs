use crate::*;

#[derive(near_sdk::serde::Serialize)]
#[serde(crate = "near_sdk::serde")]
struct AppchainAgentMessage {
    sender_id: AccountId,
    message: String,
}

#[near_bindgen]
impl AppchainAnchor {
    pub fn deliver_appchain_agent_message(
        &mut self,
        sender_id_in_near: AccountId,
        message: String,
    ) {
        let appchain_notification_history = self.internal_append_appchain_notification(
            AppchainNotification::AppchainAgentMessageSigned {
                sender_id_in_near: sender_id_in_near.clone(),
                message: message.clone(),
            },
        );
        log!(
            "Agent message '{}' signed by '{}' delivered to appchain. Crosschain notification index: '{}'.",
            &message,
            &sender_id_in_near,
            &appchain_notification_history.index.0
        );
    }
}
