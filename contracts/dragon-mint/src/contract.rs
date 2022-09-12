use crate::error::ContractError;
use crate::msg::{
    CollectionInfoResponse, CustomMintMsg, ExecuteMsg, Extension, InstantiateMsg, QueryMsg, ClaimMessage, Claim, StateResponse
};
use crate::state::{
    CollectionInfo, State, Dragon, DragonListResponse, DragonResponse, COLLECTION_INFO, DRAGON_INFO,
    DRAGON_INFO_SEQ, STATE, MIN_STAKE_TIME,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::WasmMsg::Execute;
use cosmwasm_std::{to_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Order, Response, StdError, StdResult, Uint64, Uint128, SubMsg};
use cw2::set_contract_version;
use std::ops::Add;
use std::ptr::null;

pub type Cw721Contract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty>;
pub type Cw721ExecuteMsg = cw721_base::ExecuteMsg<Extension>;
use crate::helper::generate_dragon_birth_msg;
use cw_storage_plus::Bound;
use cw_utils::Expiration;
use schemars::_serde_json::Value;
use schemars::_serde_json::Value::Null;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:dragon-mint";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let minter = deps.api.addr_validate(&msg.base.minter)?;
    Cw721Contract::default().instantiate(deps.branch(), env, info.clone(), msg.base.clone())?;

    let state = State {
        owner: String::from(info.sender.clone()),
        reward_contract_address: msg.reward_contract_address,
    };

    let collection_info = CollectionInfo {
        name: msg.base.name,
        symbol: msg.base.symbol,
        minter: minter.to_string(),
        description: "stake_dragons".to_string(),
        size: msg.size,
        base_price: msg.base_price,
    };

    STATE.save(deps.storage, &state)?;
    COLLECTION_INFO.save(deps.storage, &collection_info)?;
    DRAGON_INFO_SEQ.save(deps.storage, &Uint64::zero())?;
    MIN_STAKE_TIME.save(deps.storage, &Uint64::new(1209600))?;
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
        ExecuteMsg::UpdateOwner {new_owner} => execute_update_owner(deps, info, new_owner),
        ExecuteMsg::UpdateRewardContractAddress {new_address} => execute_update_reward_contract_address(deps, info, new_address),
        ExecuteMsg::UpdateMinStakeTime {time} => execute_update_min_stake_time(deps, info, time),
        ExecuteMsg::Mint(msg) => execute_mint(deps, env, info, msg),
        ExecuteMsg::PlantEgg { token_id } => execute_plant_egg(deps, info, env, token_id),
        ExecuteMsg::StakeDragon { token_id } => execute_stake_dragon(deps, info, env, token_id),
        ExecuteMsg::StartUnstakingProcess { token_id } => {
            execute_start_unstake_process(deps, info, env, token_id)
        }
        ExecuteMsg::UnstakeDragon { token_id } => execute_unstake_dragon(deps, info, env, token_id),
        ExecuteMsg::ClaimReward { token_id } => execute_claim_reward(deps, info, env, token_id),
        ExecuteMsg::Claim { token_id } => execute_claim(deps, info, env, token_id),
        ExecuteMsg::Burn { token_id } => execute_burn(deps, env, info, token_id),
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => execute_transfer_nft(deps, env, info, recipient, token_id),
        ExecuteMsg::Approve {
            spender,
            token_id,
            expires,
        } => execute_approve(deps, env, info, spender, token_id, expires),
        ExecuteMsg::ApproveAll { operator, expires } => {
            execute_approve_all(deps, env, info, operator, expires)
        }
        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => execute_send_nft(deps, env, info, contract, token_id, msg),
        ExecuteMsg::Revoke { spender, token_id } => {
            execute_revoke(deps, env, info, spender, token_id)
        }
        ExecuteMsg::RevokeAll { operator } => execute_revoke_all(deps, env, info, operator),
    }
}

fn execute_update_owner(
    deps: DepsMut,
    info: MessageInfo,
    new_owner: String,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    if info.sender.to_string() != state.owner {
        return Err(ContractError::Unauthorized {});
    }
    state.owner = new_owner;
    STATE.save(deps.storage, &state)?;
    Ok(Response::default().add_attribute("new_owner", state.owner))
}

fn execute_update_reward_contract_address(
    deps: DepsMut,
    info: MessageInfo,
    new_address: String,
) -> Result<Response, ContractError> {
    let mut state = STATE.load(deps.storage)?;
    if info.sender.to_string() != state.owner {
        return Err(ContractError::Unauthorized {});
    }
    state.reward_contract_address = new_address;
    STATE.save(deps.storage, &state)?;
    Ok(Response::default().add_attribute("new_reward_contract_address", state.reward_contract_address))
}

fn execute_update_min_stake_time(
    deps: DepsMut,
    info: MessageInfo,
    time: Uint64,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;
    if info.sender.to_string() != state.owner {
        return Err(ContractError::Unauthorized {})
    }
    MIN_STAKE_TIME.update::<_, StdError>(deps.storage, |min_stake_time| Ok(time))?;

    Ok(Response::default().add_attribute("min_stake_time", time))
}

fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    let msg = Cw721ExecuteMsg::Burn { token_id };
    Cw721Contract::default()
        .execute(deps, env, info, msg)
        .unwrap();
    Ok(Response::new())
}

fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut msg: CustomMintMsg,
) -> Result<Response, ContractError> {
    let mut kind = String::new();
    let mut ovulation_period: u64 = 0;
    let mut daily_income: String = String::from("");
    for item in &msg.extension {
        // iterate immutably
        let trait_type: String = item.clone().trait_type;
        let value: String = item.clone().value;

        match &trait_type[..] {
            "kind" => kind = value,
            "ovulation_period" => ovulation_period = value.parse::<u64>().unwrap(),
            "daily_income" => daily_income = value,
            _ => return Err(ContractError::UnexpectedTraitType { trait_type }),
        }
    }
    let id =
        DRAGON_INFO_SEQ.update::<_, StdError>(deps.storage, |id| Ok(id.add(Uint64::new(1))))?;
    let dragon = Dragon {
        token_id: id.to_string(),
        owner: msg.clone().base.owner,
        kind,
        ovulation_period,
        daily_income,
        hatch: Uint64::zero(),
        is_staked: false,
        stake_start_time: Uint64::zero(),
        reward_start_time: Uint64::zero(),
        unstaking_start_time: Uint64::zero(),
        unstaking_process: false,
        reward_end_time: Uint64::zero(),
    };
    DRAGON_INFO.save(deps.storage, id.u64(), &dragon)?;
    msg.base.token_id = id.to_string();
    let mint_msg = Cw721ExecuteMsg::Mint(msg.base.clone());
    Cw721Contract::default()
        .execute(deps, env, info, mint_msg)
        .unwrap();
    Ok(Response::default()
        .add_attribute("new owner", dragon.owner.clone())
        .add_attribute("dragon id", dragon.token_id)
        .add_attribute("dragon kind", dragon.kind)
        .add_attribute(
            "dragon ovulation_period",
            dragon.ovulation_period.to_string(),
        )
        .add_attribute("dragon daily income", dragon.daily_income)
        .add_attribute("hatch", dragon.hatch.to_string())
        .add_attribute("stake_start_time", dragon.stake_start_time.to_string())
        .add_attribute("reward_start_time", dragon.reward_start_time.to_string())
        .add_attribute("is_staked, {}", dragon.is_staked.to_string())
        .add_attribute("unstaking_start_time, {}", dragon.unstaking_start_time.to_string())
        .add_attribute("unstaking_process, {}", dragon.unstaking_start_time.to_string())
        .add_attribute("reward_end_time, {}", dragon.unstaking_start_time.to_string()))
}

fn execute_plant_egg(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    token_id: Uint64,
) -> Result<Response, ContractError> {
    let mut dragon = DRAGON_INFO.load(deps.storage, token_id.u64())?;
    if dragon.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    if dragon.unstaking_process {
        return Err(ContractError::OngoingUnstakingProcess {});
    }
    if !dragon.hatch.is_zero() && dragon.hatch.u64() <= env.block.time.seconds() {
        dragon.hatch = Uint64::zero();
    } else {
        return Err(ContractError::OvulationInProgress {});
    }
    DRAGON_INFO.save(deps.storage, token_id.u64(), &dragon)?;

    let collection = COLLECTION_INFO.load(deps.storage)?;
    let msg = generate_dragon_birth_msg(token_id.to_string(), info.sender.to_string())?;
    Ok(Response::default()
        .add_attribute("resetted hatch value", dragon.hatch)
        .add_message(CosmosMsg::Wasm(Execute {
            contract_addr: collection.minter,
            msg: to_binary(&msg)?,
            funds: vec![],
        })))
}

fn execute_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: Uint64,
) -> Result<Response, ContractError> {
    let msg = Cw721ExecuteMsg::TransferNft {
        recipient: recipient.clone(),
        token_id: token_id.to_string(),
    };
    let mut dragon = DRAGON_INFO.load(deps.storage, token_id.u64())?;
    let is_staked = dragon.clone().is_staked;
    if is_staked {
        return Err(ContractError::StakedDragonCantBeTransferred {});
    }
    let _is_owner = dragon.clone().is_owner(info.sender.to_string())?;
    let valid_recipient = deps.api.addr_validate(&*recipient)?;
    dragon.owner = valid_recipient.to_string();
    DRAGON_INFO.save(deps.storage, token_id.u64(), &dragon)?;
    Cw721Contract::default()
        .execute(deps, env, info.clone(), msg)
        .unwrap();

    Ok(Response::default()
        .add_attribute("old owner", info.sender.to_string())
        .add_attribute("new owner", dragon.owner))
}

pub fn execute_stake_dragon(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    token_id: Uint64,
) -> Result<Response, ContractError> {
    let mut dragon = DRAGON_INFO.load(deps.storage, token_id.u64())?;
    if dragon.is_staked {
        return Err(ContractError::DragonAlreadyStaked {});
    }
    let _is_owner = dragon.clone().is_owner(info.sender.to_string())?;
    dragon.is_staked = true;
    let now = Uint64::new(env.block.time.seconds());
    dragon.stake_start_time = now;
    dragon.reward_start_time = now;
    //get today as seconds
    let today_in_seconds = Uint64::new(env.block.time.seconds());
    //calculate the second that it will take using the ovulation period
    //1 day -> 86400 seconds
    let ovulation_period_in_seconds: Uint64 = Uint64::new(dragon.ovulation_period);
    let total_seconds_to_add = ovulation_period_in_seconds.checked_mul(Uint64::new(86400))?;
    let hatch_time: Uint64 = total_seconds_to_add.checked_add(Uint64::from(today_in_seconds))?;
    dragon.hatch = hatch_time;
    DRAGON_INFO.save(deps.storage, token_id.u64(), &dragon)?;
    Ok(Response::default()
        .add_attribute("token_id", dragon.token_id.to_string())
        .add_attribute("is_staked", dragon.is_staked.to_string())
        .add_attribute("start_time", dragon.stake_start_time)
        .add_attribute("reward_start_time", dragon.reward_start_time)
        .add_attribute("hatched dragon", token_id.to_string())
        .add_attribute("hatch value", dragon.hatch.to_string()))
}

fn execute_start_unstake_process(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    token_id: Uint64,
) -> Result<Response, ContractError> {
    let mut dragon = DRAGON_INFO.load(deps.storage, token_id.u64())?;
    if dragon.unstaking_process {
        return Err(ContractError::OngoingUnstakingProcess {});
    }
    if !dragon.is_staked {
        return Err(ContractError::DragonNotStaked {});
    }
    if dragon.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    dragon.unstaking_process = true;
    dragon.unstaking_start_time = Uint64::new(env.block.time.seconds());
    dragon.reward_end_time = Uint64::new(env.block.time.seconds());
    DRAGON_INFO.save(deps.storage, token_id.u64(), &dragon)?;
    Ok(Response::default()
        .add_attribute("token_id", dragon.clone().token_id.to_string())
        .add_attribute("unstaking_start_time", dragon.unstaking_start_time)
        .add_attribute("reward_end_time", dragon.reward_end_time))
}

fn execute_unstake_dragon(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    token_id: Uint64,
) -> Result<Response, ContractError> {
    let mut dragon = DRAGON_INFO.load(deps.storage, token_id.u64())?;
    if !dragon.is_staked {
        return Err(ContractError::DragonNotStaked {});
    }
    if dragon.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    if !dragon.unstaking_process {
        return Err(ContractError::UnstakingProcessIsNotStarted {});
    }
    let now = Uint64::new(env.block.time.seconds());
    let min_stake_time = MIN_STAKE_TIME.load(deps.storage)?;
    if now.checked_sub(dragon.unstaking_start_time)? < min_stake_time
        || dragon.unstaking_start_time.is_zero()
    {
        return Err(ContractError::MinUnstakingTimeRequired {});
    }
    //Hatch
    dragon.hatch = Uint64::zero();
    dragon.is_staked = false;
    dragon.stake_start_time = Uint64::zero();
    dragon.reward_start_time = Uint64::zero();
    dragon.unstaking_process = false;
    dragon.unstaking_start_time = Uint64::zero();
    DRAGON_INFO.save(deps.storage, token_id.u64(), &dragon)?;
    Ok(Response::default()
        .add_attribute("token_id", dragon.clone().token_id.to_string())
        .add_attribute("is_staked", dragon.clone().is_staked.to_string())
        .add_attribute(
            "stake_start_time",
            dragon.clone().stake_start_time.to_string(),
        )
        .add_attribute(
            "reward_start_time",
            dragon.clone().reward_start_time.to_string(),
        )
        .add_attribute(
            "unstaking_process",
            dragon.clone().unstaking_process.to_string(),
        )
        .add_attribute(
            "unstaking_start_time",
            dragon.clone().unstaking_start_time.to_string(),
        ))
}

fn execute_claim_reward(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    token_id: Uint64,
) -> Result<Response, ContractError> {
    let mut dragon = DRAGON_INFO.load(deps.storage, token_id.u64())?;
    if !dragon.is_staked {
        return Err(ContractError::DragonNotStaked {});
    }
    if dragon.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    //Reward calculation
    let now = env.block.time.seconds();
    let mut reward = Uint128::zero();
    if dragon.unstaking_process {
        if dragon.kind == "common".to_string() {
            let second_difference = Uint128::new(dragon.reward_end_time.u64() as u128).checked_sub(Uint128::new(dragon.reward_start_time.u64() as u128))?;
            let second_difference_multiplied = second_difference.checked_mul(Uint128::new(500000))?;
            reward = second_difference_multiplied.checked_div(Uint128::new(86400)).unwrap_or(Uint128::zero());
        } else {
            let second_difference = Uint128::new(dragon.reward_end_time.u64() as u128).checked_sub(Uint128::new(dragon.reward_start_time.u64() as u128))?;
            let second_difference_multiplied = second_difference.checked_mul(Uint128::new(1000000))?;
            let second_difference_multiplied_daily_income = second_difference_multiplied.checked_mul(Uint128::new(dragon.daily_income.parse::<u128>().unwrap()))?;
            reward = second_difference_multiplied_daily_income.checked_div(Uint128::new(86400)).unwrap_or(Uint128::zero());
        }
    } else {
        if dragon.kind == "common".to_string() {
            let second_difference = Uint128::new(now as u128).checked_sub(Uint128::new(dragon.reward_start_time.u64() as u128))?;
            let second_difference_multiplied = second_difference.checked_mul(Uint128::new(500000))?;
            reward = second_difference_multiplied.checked_div(Uint128::new(86400)).unwrap_or(Uint128::zero());
        } else {
            let second_difference = Uint128::new(now as u128).checked_sub(Uint128::new(dragon.reward_start_time.u64() as u128))?;
            let second_difference_multiplied = second_difference.checked_mul(Uint128::new(1000000))?;
            let second_difference_multiplied_daily_income = second_difference_multiplied.checked_mul(Uint128::new(dragon.daily_income.parse::<u128>().unwrap()))?;
            reward = second_difference_multiplied_daily_income.checked_div(Uint128::new(86400)).unwrap_or(Uint128::zero());
        }
    }
    let state = STATE.load(deps.storage)?;
    let msg = ClaimMessage {
        claim: Claim {
            recipient: info.sender.to_string(),
            amount: reward,
        }
    };

    let claim_reward_msg = CosmosMsg::Wasm(Execute {
        contract_addr: state.reward_contract_address,
        msg: to_binary(&msg)?,
        funds: vec![],
    });
    if dragon.unstaking_process {
        dragon.reward_start_time = Uint64::zero();
        dragon.reward_end_time = Uint64::zero();
    } else {
        dragon.reward_start_time = Uint64::new(env.block.time.seconds());
    }
    DRAGON_INFO.save(deps.storage, token_id.u64(), &dragon)?;
    Ok(Response::new().add_submessages(vec![
        SubMsg::new(claim_reward_msg),
    ]))
}

fn execute_claim(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    token_id: Uint64,
) -> Result<Response, ContractError> {
    let mut dragon = DRAGON_INFO.load(deps.storage, token_id.u64())?;
    if !dragon.is_staked {
        return Err(ContractError::DragonNotStaked {});
    }
    if dragon.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    if dragon.unstaking_process {
        dragon.reward_start_time = Uint64::zero();
        dragon.reward_end_time = Uint64::zero();
    } else {
        dragon.reward_start_time = Uint64::new(env.block.time.seconds());
    }
    DRAGON_INFO.save(deps.storage, token_id.u64(), &dragon)?;
    Ok(Response::new().add_attribute("reward_start_time", dragon.reward_start_time)
        .add_attribute("reward_end_time", dragon.reward_end_time))
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CollectionInfo {} => to_binary(&query_config(deps)?),
        QueryMsg::DragonInfo { id } => to_binary(&query_dragon(deps, id)?),
        QueryMsg::RangeDragons { start_after, limit } => {
            to_binary(&range_dragons(deps, start_after, limit)?)
        }
        QueryMsg::RangeUserDragons {
            start_after,
            limit,
            owner,
        } => to_binary(&range_user_dragons(deps, start_after, limit, owner)?),
        QueryMsg::CalculateReward { token_id } => {
            to_binary(&query_calculate_reward(deps, env, token_id)?)
        }
        QueryMsg::State {} => to_binary(&query_state(deps)?),
        _ => Cw721Contract::default().query(deps, env, msg.into()),
    }
}

fn query_calculate_reward(deps: Deps, env: Env, token_id: Uint64) -> StdResult<Uint64> {
    let dragon = DRAGON_INFO.load(deps.storage, token_id.u64()).unwrap();
    let now = Uint64::new(env.block.time.seconds());
    let mut reward = Uint64::zero();
    if dragon.unstaking_process {
        if dragon.kind == "common".to_string() {
            let second_difference = dragon.reward_end_time.checked_sub(dragon.reward_start_time)?;
            let second_difference_multiplied = second_difference.checked_mul(Uint64::new(500000))?;
            reward = second_difference_multiplied.checked_div(Uint64::new(86400))?;
        } else {
            let second_difference = dragon.reward_end_time.checked_sub(dragon.reward_start_time)?;
            let second_difference_multiplied = second_difference.checked_mul(Uint64::new(1000000))?;
            let second_difference_multiplied_daily_income = second_difference_multiplied.checked_mul(Uint64::new(dragon.daily_income.parse::<u64>().unwrap()))?;
            reward = second_difference_multiplied_daily_income.checked_div(Uint64::new(86400))?;
        }
    } else {
        if dragon.kind == "common".to_string() {
            let second_difference = now.checked_sub(dragon.reward_start_time)?;
            let second_difference_multiplied = second_difference.checked_mul(Uint64::new(500000))?;
            reward = second_difference_multiplied.checked_div(Uint64::new(86400))?;
        } else {
            let second_difference = now.checked_sub(dragon.reward_start_time)?;
            let second_difference_multiplied = second_difference.checked_mul(Uint64::new(1000000))?;
            let second_difference_multiplied_daily_income = second_difference_multiplied.checked_mul(Uint64::new(dragon.daily_income.parse::<u64>().unwrap()))?;
            reward = second_difference_multiplied_daily_income.checked_div(Uint64::new(86400))?;
        }
    }
    Ok(reward)
}

fn query_dragon(deps: Deps, id: Uint64) -> StdResult<DragonResponse> {
    let dragon = DRAGON_INFO.load(deps.storage, id.u64())?;
    Ok(DragonResponse {
        token_id: dragon.token_id,
        owner: dragon.owner,
        kind: dragon.kind,
        ovulation_period: dragon.ovulation_period,
        hatch: dragon.hatch,
        daily_income: dragon.daily_income,
        is_staked: dragon.is_staked,
        stake_start_time: dragon.stake_start_time,
        reward_start_time: dragon.reward_start_time,
        unstaking_start_time: dragon.unstaking_start_time,
        unstaking_process: dragon.unstaking_process,
        reward_end_time: dragon.reward_end_time,
    })
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

fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(StateResponse {
        owner: state.owner,
        reward_contract_address: state.reward_contract_address
    })
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

fn range_dragons(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<DragonListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let dragons: StdResult<Vec<_>> = DRAGON_INFO
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();
    let res = DragonListResponse {
        dragons: dragons?.into_iter().map(|l| l.1.into()).collect(),
    };
    Ok(res)
}

fn range_user_dragons(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
    owner: String,
) -> StdResult<DragonListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);
    let dragons: StdResult<Vec<_>> = DRAGON_INFO
        .range(deps.storage, start, None, Order::Ascending)
        .filter(|r| r.as_ref().unwrap().1.owner == owner)
        .take(limit)
        .collect();
    let res = DragonListResponse {
        dragons: dragons?.into_iter().map(|l| l.1.into()).collect(),
    };
    Ok(res)
}
