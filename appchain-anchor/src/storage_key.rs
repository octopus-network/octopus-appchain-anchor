use near_sdk::AccountId;

/// Storage keys for collections of sub-struct in main contract
pub enum StorageKey {
    OctToken,
    WrappedAppchainToken,
    NearFungibleTokenSymbols,
    NearFungibleTokens,
    NextValidatorSet,
    ValidatorSetHistories,
    ValidatorListOfEras,
    UnwithdrawedValidatorRewards,
    UnwithdrawedDelegatorRewards,
    UnbondedStakes,
    LookupMapOfValidatorIdsInAppchain,
    AppchainSettings,
    AnchorSettings,
    ProtocolSettings,
    StakingHistories,
    StakingHistoriesMap,
    TokenBridgingHistories,
    TokenBridgingHistoriesMap,
    AnchorEvents,
    AnchorEventsMap,
    PermissionlessActionsStatus,
    ValidatorIdsInValidatorSet(u64),
    LookupMapOfVToDInValidatorSet(u64),
    LookupMapOfDToVInValidatorSet(u64),
    ValidatorsInValidatorSet(u64),
    DelegatorsInValidatorSet(u64),
    UnprofitableValidatorIdsInValidatorSet(u64),
    DelegatorIdsInLookupMapOfVToDInValidatorSet {
        era_number: u64,
        validator_id: AccountId,
    },
    ValidatorIdsInLookupMapOfDToVInValidatorSet {
        era_number: u64,
        delegator_id: AccountId,
    },
}

impl StorageKey {
    pub fn to_string(&self) -> String {
        match self {
            StorageKey::OctToken => "oct".to_string(),
            StorageKey::WrappedAppchainToken => "wat".to_string(),
            StorageKey::NearFungibleTokenSymbols => "fts".to_string(),
            StorageKey::NearFungibleTokens => "ft".to_string(),
            StorageKey::NextValidatorSet => "nvs".to_string(),
            StorageKey::ValidatorSetHistories => "vsh".to_string(),
            StorageKey::ValidatorListOfEras => "vloe".to_string(),
            StorageKey::UnwithdrawedValidatorRewards => "uwvrs".to_string(),
            StorageKey::UnwithdrawedDelegatorRewards => "uwdrs".to_string(),
            StorageKey::UnbondedStakes => "ubss".to_string(),
            StorageKey::LookupMapOfValidatorIdsInAppchain => "via".to_string(),
            StorageKey::AppchainSettings => "acs".to_string(),
            StorageKey::AnchorSettings => "ans".to_string(),
            StorageKey::ProtocolSettings => "pcs".to_string(),
            StorageKey::StakingHistories => "skh".to_string(),
            StorageKey::StakingHistoriesMap => "skhm".to_string(),
            StorageKey::TokenBridgingHistories => "tbh".to_string(),
            StorageKey::TokenBridgingHistoriesMap => "tbhm".to_string(),
            StorageKey::AnchorEvents => "aes".to_string(),
            StorageKey::AnchorEventsMap => "aesm".to_string(),
            StorageKey::PermissionlessActionsStatus => "pas".to_string(),
            StorageKey::ValidatorIdsInValidatorSet(era_number) => format!("{}vis", era_number),
            StorageKey::LookupMapOfVToDInValidatorSet(era_number) => format!("{}lmvtd", era_number),
            StorageKey::LookupMapOfDToVInValidatorSet(era_number) => format!("{}lmdtv", era_number),
            StorageKey::ValidatorsInValidatorSet(era_number) => format!("{}vs", era_number),
            StorageKey::DelegatorsInValidatorSet(era_number) => format!("{}ds", era_number),
            StorageKey::UnprofitableValidatorIdsInValidatorSet(era_number) => {
                format!("{}upvis", era_number)
            }
            StorageKey::DelegatorIdsInLookupMapOfVToDInValidatorSet {
                era_number,
                validator_id,
            } => format!("{}lmvtd{}", era_number, validator_id),
            StorageKey::ValidatorIdsInLookupMapOfDToVInValidatorSet {
                era_number,
                delegator_id,
            } => format!("{}lmdtv{}", era_number, delegator_id),
        }
    }
    pub fn into_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}
