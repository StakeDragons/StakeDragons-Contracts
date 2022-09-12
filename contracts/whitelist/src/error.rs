use cosmwasm_std::StdError;
use cw721_base::ContractError as Cw721ContractError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("721 error : {method}")]
    NftContractError { method: String },

    #[error("DuplicateMember: {0}")]
    DuplicateMember(String),

    #[error("AlreadyClaimed: {0}")]
    AlreadyClaimed(String),

    #[error("NoMemberFound: {0}")]
    NoMemberFound(String),
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
impl From<ContractError> for Cw721ContractError {
    fn from(err: ContractError) -> Cw721ContractError {
        match err {
            ContractError::Unauthorized {} => Cw721ContractError::Unauthorized {},
            _ => unreachable!("cannot connect {:?} to cw721ContractError", err),
        }
    }
}
