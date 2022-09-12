use cosmwasm_std::{OverflowError, StdError};
use cw721_base::ContractError as Cw721ContractError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("721 error : {method}")]
    NftContractError { method: String },

    #[error("Invalid size")]
    InvalidSize {},

    #[error("InvalidCreationFee")]
    InvalidCreationFee {},

    #[error("token_id already claimed")]
    Claimed {},

    #[error("there are no unclaimed eggs in the collection")]
    NoEggAvailable {},

    #[error("Approval not found for: {spender}")]
    ApprovalNotFound { spender: String },

    #[error("Unexpected trait type: {trait_type}")]
    UnexpectedTraitType { trait_type: String },

    #[error("Staked dragon cannot be transferred")]
    StakedDragonCantBeTransferred {},

    #[error("Dragon must be staked to be hatch")]
    DragonNotStaked {},

    #[error("Minimum stake time required for unstaking")]
    MinStakeTimeRequired {},

    #[error("Kind not found")]
    KindNotFound {},

    #[error("Ovulation period has not ended.")]
    OvulationInProgress {},

    #[error("{0}")]
    Payment(#[from] PaymentError),

    #[error("Dragon is already staked")]
    DragonAlreadyStaked {},

    #[error("Min unstaking time required")]
    MinUnstakingTimeRequired {},

    #[error("Unstaking process must be started first")]
    UnstakingProcessIsNotStarted {},

    #[error("Unstaking process is ongoing")]
    OngoingUnstakingProcess {},

    #[error("{0}")]
    Overflow(#[from] OverflowError),
}

impl From<ContractError> for Cw721ContractError {
    fn from(err: ContractError) -> Cw721ContractError {
        match err {
            ContractError::Unauthorized {} => Cw721ContractError::Unauthorized {},
            ContractError::Claimed {} => Cw721ContractError::Claimed {},
            _ => unreachable!("cannot connect {:?} to cw721ContractError", err),
        }
    }
}
