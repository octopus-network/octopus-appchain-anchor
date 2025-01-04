use crate::*;

impl Default for NativeNearToken {
    fn default() -> Self {
        Self {
            locked_balance: U128::from(0),
            bridging_state: BridgingState::Closed,
            price_in_usd: U128::from(0),
        }
    }
}
