#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Order, Response, StdResult,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;

use crate::error::ContractError;
use crate::msg::{
    CustomMintMsg, ExecuteMsg, Extension, InstantiateMsg, MembersResponse, QueryMsg,
    WhitelistStateResponse,
};
use crate::state::{State, STATE, WHITELIST};

pub type Cw721Contract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty>;
pub type Cw721ExecuteMsg = cw721_base::ExecuteMsg<Extension>;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:egg-mint";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const PAGINATION_DEFAULT_LIMIT: u32 = 25;
const PAGINATION_MAX_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let res = Cw721Contract::default().instantiate(deps.branch(), env, info, msg.base.clone());

    if res.is_err() {
        return Err(ContractError::NftContractError {
            method: "721 instantiate".to_string(),
        });
    }

    let mut filtered_members = msg.members;

    // remove duplicate members
    filtered_members.sort_unstable();
    filtered_members.dedup();

    let state = State {
        name: msg.base.name,
        symbol: msg.base.symbol,
        minter: msg.base.minter,
        size: filtered_members.len() as u32,
        claimed_dragons: 0,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    //True is set for all members, they can all claim their dragon
    for member in filtered_members.into_iter() {
        let addr = deps.api.addr_validate(&member.clone())?;
        WHITELIST.save(deps.storage, addr, &true)?;
    }
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
        ExecuteMsg::AddMembers { members } => execute_add_members(deps, env, info, members),
        ExecuteMsg::RemoveMembers { members } => execute_remove_members(deps, env, info, members),
    }
}

fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CustomMintMsg,
) -> Result<Response, ContractError> {
    //Only minter can mint
    let state = STATE.load(deps.storage)?;
    if info.sender != state.minter {
        return Err(ContractError::Unauthorized {});
    }

    // if address do not exists in whitelist
    let addr = deps.api.addr_validate(&msg.base.clone().owner)?;
    if !WHITELIST.has(deps.storage, addr.clone()) {
        return Err(ContractError::NoMemberFound(addr.to_string()));
    }

    // if dragon is already claimed
    let can_claim = WHITELIST.load(deps.storage, addr.clone())?;
    if can_claim == false {
        return Err(ContractError::AlreadyClaimed(addr.to_string()));
    }

    let new = State {
        name: state.name,
        symbol: state.symbol,
        minter: state.minter,
        size: state.size,
        claimed_dragons: state.claimed_dragons + 1,
    };

    STATE.save(deps.storage, &new)?;
    WHITELIST.save(deps.storage, addr.clone(), &false)?;

    let mint_msg = Cw721ExecuteMsg::Mint(msg.base.clone());
    let res = Cw721Contract::default().execute(deps, env, info, mint_msg);

    if res.is_err() {
        return Err(ContractError::NftContractError {
            method: "721 mint".to_string(),
        });
    }

    Ok(Response::default().add_attribute("new owner", msg.base.owner.clone()))
}

fn execute_add_members(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    mut members: Vec<String>,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    //if info.sender.to_string() != state.admin || info.sender != state.minter {
      //  return Err(ContractError::Unauthorized {});
    //}

    // remove duplicate members
    members.sort_unstable();
    members.dedup();

    for add in members.into_iter() {
        let addr = deps.api.addr_validate(&add)?;
        if WHITELIST.has(deps.storage, addr.clone()) {
            return Err(ContractError::DuplicateMember(addr.to_string()));
        }
        WHITELIST.save(deps.storage, addr, &true)?;
        state.size += 1;
    }
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "add_members")
        .add_attribute("sender", info.sender))
}

pub fn execute_remove_members(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    members: Vec<String>,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    //if info.sender.to_string() != state.admin || info.sender.to_string() != state.minter {
      //  return Err(ContractError::Unauthorized {});
    //}

    for remove in members.into_iter() {
        let addr = deps.api.addr_validate(&remove)?;
        if !WHITELIST.has(deps.storage, addr.clone()) {
            return Err(ContractError::NoMemberFound(addr.to_string()));
        }
        WHITELIST.remove(deps.storage, addr);
        state.size -= 1;
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("action", "remove_members")
        .add_attribute("sender", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Whitelist {} => to_binary(&query_state(deps)?),
        QueryMsg::Members { start_after, limit } => {
            to_binary(&query_members(deps, start_after, limit)?)
        }
        QueryMsg::IsMember { address } => to_binary(&is_member(deps, address)?),
        _ => Cw721Contract::default().query(deps, env, msg.into()),
    }
}

fn is_member(deps: Deps, address: String) -> StdResult<bool> {
    let addr = deps.api.addr_validate(&address)?;
    let res = WHITELIST.has(deps.storage, addr);
    Ok(res)
}

pub fn query_members(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<MembersResponse> {
    let limit = limit
        .unwrap_or(PAGINATION_DEFAULT_LIMIT)
        .min(PAGINATION_MAX_LIMIT) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.map(Bound::exclusive);
    let members = WHITELIST
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|addr| addr.unwrap().0.to_string())
        .collect::<Vec<String>>();

    Ok(MembersResponse { members })
}

pub fn query_state(deps: Deps) -> StdResult<WhitelistStateResponse> {
    let state = STATE.load(deps.storage)?;

    Ok(WhitelistStateResponse {
        name: state.name,
        symbol: state.symbol,
        minter: state.minter,
        size: state.size,
        claimed_dragons: state.claimed_dragons,
    })
}
