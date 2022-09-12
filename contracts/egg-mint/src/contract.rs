#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult,
    Uint64,
};
use cw2::set_contract_version;
use cw_utils::Expiration;
use std::ops::Add;

use crate::error::ContractError;
use crate::msg::{
    CollectionInfoResponse, CustomMintMsg, ExecuteMsg, Extension, InstantiateMsg,
    OwnedEggInfoResponse, QueryMsg,
};
use crate::state::{CollectionInfo, COLLECTION_INFO, OWNED_EGG_COUNT};
pub type Cw721Contract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty>;
pub type Cw721ExecuteMsg = cw721_base::ExecuteMsg<Extension>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:egg-mint";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let minter = deps.api.addr_validate(&msg.base.minter)?;
    let res = Cw721Contract::default().instantiate(deps.branch(), env, info, msg.base.clone());

    if res.is_err() {
        return Err(ContractError::NftContractError {
            method: "instantiate".to_string(),
        });
    }
    let collection_info = CollectionInfo {
        name: msg.base.name,
        symbol: msg.base.symbol,
        minter: minter.to_string(),
        description: "test1".to_string(),
        size: msg.size,
        base_price: msg.base_price,
    };
    COLLECTION_INFO.save(deps.storage, &collection_info)?;
    OWNED_EGG_COUNT.save(deps.storage, &Uint64::zero())?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint(msg) => execute_mint(deps, env, info, msg),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
        ExecuteMsg::Approve {
            spender,
            token_id,
            expires,
        } => execute_approve(deps, env, info, spender, token_id, expires),
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => execute_transfer_nft(deps, env, info, recipient, token_id),
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => execute_send_nft(deps, env, info, contract, token_id, msg),
        ExecuteMsg::Revoke { spender, token_id } => {
            execute_revoke(deps, env, info, spender, token_id)
        }
        ExecuteMsg::ApproveAll { operator, expires } => {
            execute_approve_all(deps, env, info, operator, expires)
        }
        ExecuteMsg::RevokeAll { operator } => execute_revoke_all(deps, env, info, operator),
    }
}

fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CustomMintMsg,
) -> Result<Response, ContractError> {
    OWNED_EGG_COUNT.update::<_, StdError>(deps.storage, |id| Ok(id.add(Uint64::new(1))))?;

    let mint_msg = Cw721ExecuteMsg::Mint(msg.base.clone());
    let mint_res = Cw721Contract::default().execute(deps, env.clone(), info.clone(), mint_msg);
    if mint_res.is_err() {
        return Err(ContractError::NftContractError {
            method: "mint".to_string(),
        });
    }
    Ok(Response::default().add_attribute("new owner", msg.base.owner.clone()))
}

fn execute_approve_all(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    let msg = Cw721ExecuteMsg::ApproveAll { operator, expires };
    let res = Cw721Contract::default().execute(deps, env, info, msg);
    if res.is_err() {
        return Err(ContractError::NftContractError {
            method: "approve all".to_string(),
        });
    }
    Ok(Response::new())
}

fn execute_approve(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    let msg = Cw721ExecuteMsg::Approve {
        spender,
        token_id,
        expires,
    };
    let res = Cw721Contract::default().execute(deps, env, info, msg);
    if res.is_err() {
        return Err(ContractError::NftContractError {
            method: "approve".to_string(),
        });
    }
    Ok(Response::new())
}

fn execute_revoke_all(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
) -> Result<Response, ContractError> {
    let msg = Cw721ExecuteMsg::RevokeAll { operator };
    let res = Cw721Contract::default().execute(deps, env, info, msg);
    if res.is_err() {
        return Err(ContractError::NftContractError {
            method: "revoke all".to_string(),
        });
    }
    Ok(Response::new())
}

fn execute_revoke(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let msg = Cw721ExecuteMsg::Revoke { spender, token_id };
    let res = Cw721Contract::default().execute(deps, env, info, msg);
    if res.is_err() {
        return Err(ContractError::NftContractError {
            method: "revoke".to_string(),
        });
    }
    Ok(Response::new())
}

fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let msg = Cw721ExecuteMsg::Burn { token_id };
    let res = Cw721Contract::default().execute(deps, env, info, msg);

    Ok(Response::new().add_attribute("res", res.is_err().to_string()))
}

fn execute_send_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id: String,
    msg: Binary,
) -> Result<Response, ContractError> {
    let send_msg = Cw721ExecuteMsg::SendNft {
        contract,
        token_id,
        msg,
    };
    let res = Cw721Contract::default().execute(deps, env, info, send_msg);
    if res.is_err() {
        return Err(ContractError::NftContractError {
            method: "send nft".to_string(),
        });
    }
    Ok(Response::new())
}

fn execute_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: String,
) -> Result<Response, ContractError> {
    let msg = Cw721ExecuteMsg::TransferNft {
        recipient,
        token_id,
    };
    let res = Cw721Contract::default().execute(deps, env, info, msg);
    if res.is_err() {
        return Err(ContractError::NftContractError {
            method: "transfer nft".to_string(),
        });
    }
    Ok(Response::default().add_attribute("trasnfer nft", "success"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CollectionInfo {} => to_binary(&query_config(deps)?),
        QueryMsg::OwnedEggCount {} => to_binary(&query_owned_egg_count(deps)?),
        _ => Cw721Contract::default().query(deps, env, msg.into()),
    }
}

fn query_config(deps: Deps) -> StdResult<CollectionInfoResponse> {
    let info = COLLECTION_INFO.load(deps.storage)?;

    Ok(CollectionInfoResponse {
        name: info.name,
        symbol: info.symbol,
        minter: info.minter,
        description: info.description,
        size: info.size,
        base_price: info.base_price,
    })
}

pub fn query_owned_egg_count(deps: Deps) -> StdResult<OwnedEggInfoResponse> {
    let info = COLLECTION_INFO.load(deps.storage)?;
    let owned = OWNED_EGG_COUNT.load(deps.storage)?;

    Ok(OwnedEggInfoResponse {
        owned,
        size: info.size,
    })
}
