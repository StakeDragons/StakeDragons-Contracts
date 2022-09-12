#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg, Uint128,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetStateResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};
use cw20::{Cw20Contract, Cw20ExecuteMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:stake-reward";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: info.sender.into_string(),
        dragon_contract: msg.dragon_contract,
        cw20_contract: msg.cw20_contract,
        admin: msg.admin,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Claim { recipient, amount } => {
            execute_distribute_reward(deps, info, recipient, amount)
        }
        ExecuteMsg::EditState {
            admin,
            dragon_contract,
            cw20_contract,
            owner,
        } => execute_edit_state(deps, info, admin, dragon_contract, owner, cw20_contract),
    }
}

pub fn execute_distribute_reward(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    //Only owner and dragon contract can execute this message
    if info.sender != state.owner || info.sender != state.dragon_contract {
        return Err(ContractError::Unauthorized {});
    }

    let cw20_execute_send = Cw20ExecuteMsg::Transfer { recipient, amount };
    let reward_send_msg = Cw20Contract(state.cw20_contract)
        .call(cw20_execute_send)
        .map_err(ContractError::Std)?;

    Ok(Response::new()
        .add_submessages(vec![SubMsg::new(reward_send_msg)])
        .add_attribute("method", "reset"))
}

pub fn execute_edit_state(
    deps: DepsMut,
    info: MessageInfo,
    admin: String,
    dragon_contract: String,
    owner: String,
    cw20_contract: Addr,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    //Only owner and dragon contract can execute this message
    if info.sender != state.owner || info.sender != state.admin {
        return Err(ContractError::Unauthorized {});
    }

    let new_state = State {
        owner,
        dragon_contract,
        cw20_contract,
        admin,
    };
    STATE.save(deps.storage, &new_state)?;

    Ok(Response::new().add_attribute("method", "reset"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_binary(&query_state(deps)?),
    }
}

fn query_state(deps: Deps) -> StdResult<GetStateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(GetStateResponse {
        owner: state.owner,
        admin: state.admin,
        dragon_contract: state.dragon_contract,
        cw20_contract: state.cw20_contract,
    })
}
