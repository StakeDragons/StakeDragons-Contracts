use crate::msg::{ExecuteMsg, InstantiateMsg, ReceiveMsg};
use crate::state::{token_map, Config, Token, CONFIG};
use crate::ContractError;
use std::ops::Mul;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{attr, from_slice, to_binary, Addr, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128, WasmMsg};
use cw2::set_contract_version;
use cw20::{Cw20Contract, Cw20ExecuteMsg, Cw20ReceiveMsg};

const CONTRACT_NAME: &str = "crates.io:cw721-marketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let cfg = Config::new(
        deps.as_ref(),
        msg.admin,
        msg.nft_addr,
        msg.allowed_native,
        msg.allowed_cw20,
        msg.fee_percentage,
        msg.collector_addr,
    )?;

    CONFIG.save(deps.storage, &cfg)?;

    let res = Response::new().add_attributes(vec![
        attr("action", "instantiate"),
        attr("admin", cfg.admin),
        attr("nft_contract_addr", cfg.nft_contract_addr),
        attr(
            "allowed_native",
            cfg.allowed_native.unwrap_or("None".to_string()),
        ),
        attr(
            "allowed_cw20",
            cfg.allowed_cw20.unwrap_or(Addr::unchecked("None")),
        ),
        attr("fee_percentage", cfg.fee_percentage.to_string()),
        attr("collector_addr", cfg.collector_addr),
    ]);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ListTokens { tokens } => execute_list_token(deps, env, info, tokens),
        ExecuteMsg::DelistTokens { tokens } => execute_delist_token(deps, env, info, tokens),
        ExecuteMsg::UpdatePrice { token, price } => {
            execute_update_price(deps, env, info, token, price)
        }
        ExecuteMsg::UpdateConfig {
            admin,
            nft_addr,
            allowed_native,
            allowed_cw20,
            fee_percentage,
            collector_addr,
        } => execute_update_config(
            deps,
            env,
            info,
            admin,
            nft_addr,
            allowed_native,
            allowed_cw20,
            fee_percentage,
            collector_addr,
        ),
        ExecuteMsg::Receive(cw20_receive_msg) => execute_receive(deps, env, info, cw20_receive_msg),
    }
}

pub fn execute_list_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    tokens: Vec<Token>,
) -> Result<Response, ContractError> {
    if tokens.is_empty() {
        return Err(ContractError::WrongInput {});
    }

    let mut res = Response::new();
    for t in tokens {
        let opt_token = token_map().may_load(deps.storage, t.id.clone())?;
        // if exists update listing, if not register
        if let Some(mut token) = opt_token.clone() {
            // check if sender has approval
            if token.owner != info.sender {
                return Err(ContractError::Unauthorized {});
            }
            token.on_sale = true;
            token.price = t.price;
            token_map().save(deps.storage, token.id.clone(), &token)?;

        } else {
            // only admin can register new tokens
            //if cfg.admin != info.sender {
            // return Err(ContractError::Unauthorized {});
            //}
            token_map().save(deps.storage, t.id.clone(), &t)?;
        }
        res = res.add_attribute("token", format!("token{:?}", t.id));
    }

    Ok(res.add_attribute("action", "list_token"))
}

pub fn execute_delist_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    tokens: Vec<String>,
) -> Result<Response, ContractError> {
    let mut res = Response::new();
    let cfg = CONFIG.load(deps.storage)?;

    for t in tokens {
        let mut token = token_map().load(deps.storage, t.clone())?;
        // check if sender has approval
        // check if sender has approval

        if info.sender != token.clone().owner {
            return Err(ContractError::Unauthorized {});
        }

        token.on_sale = false;
        token_map().save(deps.storage, t.clone(), &token)?;

        //transfer ownership from market to owner
        let transfer_msg = cw721::Cw721ExecuteMsg::TransferNft {
            recipient: token.clone().owner,
            token_id: token.clone().id,
        };

        let execute_transfer_msg: CosmosMsg = WasmMsg::Execute {
            contract_addr: cfg.nft_contract_addr.clone().into_string(),
            msg: to_binary(&transfer_msg)?,
            funds: vec![],
        }
        .into();

        res = res
            .add_attribute("token", format!("token{:?}", t))
            .add_submessages(vec![SubMsg::new(execute_transfer_msg)]);
    }
    Ok(res.add_attribute("action", "delist_tokens"))
}

pub fn execute_update_price(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    price: Uint128,
) -> Result<Response, ContractError> {
    let mut token = token_map()
        .may_load(deps.storage, token_id.clone())?
        .ok_or(ContractError::NotFound {})?;

    // check if sender has approval
    if info.sender != token.clone().owner {
        return Err(ContractError::Unauthorized {});
    }

    token.price = price;
    token_map().save(deps.storage, token_id.clone(), &token)?;

    Ok(Response::new()
        .add_attribute("action", "update_price")
        .add_attribute("token_id", token_id)
        .add_attribute("price", price))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    nft_addr: Option<String>,
    allowed_native: Option<String>,
    allowed_cw20: Option<String>,
    fee_percentage: Option<Decimal>,
    collector_addr: Option<String>,
) -> Result<Response, ContractError> {
    let mut cfg = CONFIG.load(deps.storage)?;
    if cfg.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(admin) = admin {
        cfg.admin = deps.api.addr_validate(&admin)?
    }
    if let Some(nft_addr) = nft_addr {
        cfg.nft_contract_addr = deps.api.addr_validate(&nft_addr)?
    }

    match (allowed_native, allowed_cw20) {
        (Some(native), None) => cfg.allowed_native = Some(native),
        (None, Some(cw20_addr)) => cfg.allowed_cw20 = Some(deps.api.addr_validate(&cw20_addr)?),
        _ => return Err(ContractError::InvalidTokenType {}),
    }

    if let Some(fee_percentage) = fee_percentage {
        cfg.fee_percentage = fee_percentage
    }

    if let Some(collector_addr) = collector_addr {
        cfg.collector_addr = deps.api.addr_validate(&collector_addr)?
    }

    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn execute_receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: ReceiveMsg = from_slice(&wrapper.msg)?;
    let amount = wrapper.amount;
    let sender = wrapper.sender;
    match msg {
        ReceiveMsg::Buy {
            recipient,
            token_id,
        } => execute_buy_cw20(deps, _env, info, sender, recipient, token_id, amount),
    }
}

pub fn execute_buy_cw20(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _sender: String,
    recipient: String,
    token_id: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;

    if info.sender
        != cfg
            .allowed_cw20
            .clone()
            .ok_or(ContractError::CW20TokenNotSupported {})?
    {
        return Err(ContractError::CW20TokenNotAllowed {
            sent: info.sender.to_string(),
            need: cfg.allowed_cw20.unwrap().to_string(),
        });
    }

    let mut nft_token = token_map()
        .load(deps.storage, token_id.clone())
        .map_err(|_e| ContractError::NotFound {})?;

    // check if nft is on sale
    if !nft_token.on_sale {
        return Err(ContractError::NftNotOnSale {});
    }

    // check price matches
    if nft_token.price != amount {
        return Err(ContractError::SentWrongFundsAmount {
            need: nft_token.price,
            sent: amount,
        });
    }

    // now we can buy
    let transfer_msg = cw721::Cw721ExecuteMsg::TransferNft {
        recipient: recipient.clone(),
        token_id: token_id.clone(),
    };

    let execute_transfer_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: cfg.nft_contract_addr.clone().into_string(),
        msg: to_binary(&transfer_msg)?,
        funds: vec![],
    }
    .into();

    // deduce fee
    let fee;
    if nft_token.price < Uint128::new(5) {
        fee = Uint128::new(1);
    } else {
        fee = nft_token.price.mul(cfg.fee_percentage);
    }
    let price_fee_deduct = nft_token.price.checked_sub(fee)?;

    let cw20_execute_msg_op = Cw20ExecuteMsg::Transfer {
        recipient: nft_token.clone().owner,
        amount: price_fee_deduct,
    };
    let owner_payout_msg = Cw20Contract(cfg.allowed_cw20.clone().unwrap())
        .call(cw20_execute_msg_op)
        .map_err(ContractError::Std)?;

    //0.05 fee to collector of the market contract
    let cw20_execute_msg_fp = Cw20ExecuteMsg::Transfer {
        recipient: cfg.collector_addr.into_string(),
        amount: fee,
    };
    let fee_payout_msg = Cw20Contract(cfg.allowed_cw20.unwrap())
        .call(cw20_execute_msg_fp)
        .map_err(ContractError::Std)?;

    // update token owner and sale status
    nft_token.on_sale = false;
    nft_token.owner = recipient;

    token_map().save(deps.storage, token_id.clone(), &nft_token)?;

    let res = Response::new()
        .add_submessages(vec![
            SubMsg::new(execute_transfer_msg),
            SubMsg::new(owner_payout_msg),
            SubMsg::new(fee_payout_msg),
        ])
        .add_attribute("action", "buy_cw20")
        .add_attribute("token_id", token_id)
        .add_attribute("price", nft_token.price)
        .add_attribute("fee", fee);

    Ok(res)
}
