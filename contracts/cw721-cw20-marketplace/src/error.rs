use cosmwasm_std::{OverflowError, StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("NotFound")]
    NotFound {},

    #[error("Native token not in allowed list: {denom}")]
    NativeDenomNotAllowed { denom: String },

    #[error("No support for CW20 and native tokens simultaneously")]
    InvalidTokenType {},

    #[error("The marketplace does not support CW20 tokens")]
    CW20TokenNotSupported {},

    #[error("This CW20 token is not allowed: (current: {sent}, allowed: {need}")]
    CW20TokenNotAllowed { sent: String, need: String },

    #[error("Send single native token type")]
    SendSingleNativeToken {},

    #[error("Sent wrong amount of funds, need: {need} sent: {sent}")]
    SentWrongFundsAmount { need: Uint128, sent: Uint128 },

    #[error("NFT not on sale")]
    NftNotOnSale {},

    #[error("Marketplace contract is not approved as operator")]
    NotApproved {},

    #[error("Approval expired")]
    ApprovalExpired {},

    #[error("Wrong input")]
    WrongInput {},
}
