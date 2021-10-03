use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct AnchorEvents {
    /// The anchor event data map.
    pub events: LookupMap<u64, AnchorEvent>,
    /// The start index of valid anchor event in `anchor_events`.
    pub start_index: u64,
    /// The end index of valid anchor event in `anchor_events`.
    pub end_index: u64,
}

pub trait AnchorViewer {
    /// Get the start index of anchor events stored in anchor.
    fn get_start_index_of_anchor_event(&self) -> U64;
    /// Get the end index of anchor events stored in anchor.
    fn get_end_index_of_anchor_event(&self) -> U64;
    /// Get anchor event by index.
    /// If the param `index `is omitted, the latest event will be returned.
    /// If the paran `index` is smaller than the start index, or bigger than the end index
    /// stored in anchor, or there is no event in anchor yet, `Option::None` will be returned.
    fn get_anchor_event(&self, index: Option<U64>) -> Option<AnchorEvent>;
    /// Get the validator list of a certain era.
    fn get_validator_list_of_era(&self, era_number: U64) -> Vec<AppchainValidator>;
}

#[near_bindgen]
impl AnchorViewer for AppchainAnchor {
    //
    fn get_start_index_of_anchor_event(&self) -> U64 {
        todo!()
    }
    //
    fn get_end_index_of_anchor_event(&self) -> U64 {
        todo!()
    }
    //
    fn get_anchor_event(&self, index: Option<U64>) -> Option<AnchorEvent> {
        todo!()
    }
    //
    fn get_validator_list_of_era(&self, era_number: U64) -> Vec<AppchainValidator> {
        todo!()
    }
}
