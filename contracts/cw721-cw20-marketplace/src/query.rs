#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::msg::{ConfigResponse, FloorPriceResponse, QueryMsg, TokenResponse, TokensResponse};
use crate::state::{token_map, Token, CONFIG, ON_SALE};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult, Uint128};
use cw_storage_plus::Bound;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&ConfigResponse {
            config: CONFIG.load(deps.storage)?,
        }),
        QueryMsg::Token { id } => to_binary(&TokenResponse {
            token: token_map().load(deps.storage, id)?,
        }),
        QueryMsg::RangeTokens { start_after, limit } => {
            to_binary(&range_tokens(deps, start_after, limit)?)
        }
        QueryMsg::ListTokens { ids } => to_binary(&list_tokens(deps, ids)?),
        QueryMsg::ListTokensOnSale { start_after, limit } => {
            to_binary(&range_tokens_on_sale(deps, start_after, limit)?)
        }
        QueryMsg::ListByPriceAsc { start_after, limit } => {
            to_binary(&range_tokens_by_price_asc(deps, start_after, limit)?)
        }
        QueryMsg::ListByPriceDesc { start_after, limit } => {
            to_binary(&range_tokens_by_price_desc(deps, start_after, limit)?)
        }
        QueryMsg::ListByRarity {
            start_after,
            limit,
            rarity,
        } => to_binary(&range_tokens_by_rarity(deps, start_after, limit, rarity)?),
        QueryMsg::ListByOwner {
            start_after,
            limit,
            owner,
        } => to_binary(&range_tokens_by_owner(deps, start_after, limit, owner)?),
        QueryMsg::GetListedSize { start_after } => to_binary(&get_listed_size(deps, start_after)?),
        QueryMsg::ListByRarityAsc {
            start_after,
            limit,
            rarity,
        } => to_binary(&range_tokens_by_rarity_asc(
            deps,
            start_after,
            limit,
            rarity,
        )?),
        QueryMsg::ListByRarityDesc {
            start_after,
            limit,
            rarity,
        } => to_binary(&range_tokens_by_rarity_desc(
            deps,
            start_after,
            limit,
            rarity,
        )?),
        QueryMsg::GetFloorPrices {} => to_binary(&get_floor_prices(deps)?),

        QueryMsg::GetListedTokensByOwner { owner} => to_binary(&get_listed_by_owner(deps,owner)?)
    }
}

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

pub fn range_tokens(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let records: StdResult<Vec<_>> = token_map()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    let tokens = records?.into_iter().map(|r| r.1).collect();

    Ok(TokensResponse { tokens })
}

pub fn list_tokens(deps: Deps, ids: Vec<String>) -> StdResult<TokensResponse> {
    let tokens: StdResult<Vec<_>> = ids
        .into_iter()
        .map(|id| token_map().load(deps.storage, id))
        .collect();

    Ok(TokensResponse { tokens: tokens? })
}

pub fn range_tokens_on_sale(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let start = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let records: StdResult<Vec<_>> = token_map()
        .idx
        .on_sale
        .prefix(ON_SALE)
        .range(deps.storage, start, None, Order::Ascending)
        .collect();

    let mut tokens: Vec<Token> = records?.into_iter().map(|r| r.1).collect();
    tokens.sort_by(|a, b| a.id.cmp(&b.id));
    let token_res = tokens.into_iter().take(limit).collect();

    Ok(TokensResponse { tokens: token_res })
}

pub fn get_listed_size(deps: Deps, start_after: Option<String>) -> StdResult<usize> {
    let start = start_after.map(Bound::exclusive);

    let records: StdResult<Vec<_>> = token_map()
        .idx
        .on_sale
        .prefix(ON_SALE)
        .range(deps.storage, start, None, Order::Ascending)
        .collect();

    let tokens: Vec<Token> = records?.into_iter().map(|r| r.1).collect();

    Ok(tokens.len())
}

pub fn range_tokens_by_rarity(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    rarity: Vec<String>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let records: StdResult<Vec<_>> = token_map()
        .idx
        .on_sale
        .prefix(ON_SALE)
        .range(deps.storage, start, None, Order::Ascending)
        .collect();

    let tokens: Vec<Token> = records?
        .into_iter()
        .filter(|r| rarity.contains(&r.1.rarity))
        .map(|r| r.1)
        .collect();

    let token_res = tokens.into_iter().take(limit).collect();
    Ok(TokensResponse { tokens: token_res })
}

pub fn range_tokens_by_owner(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    owner: String,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let records: StdResult<Vec<_>> = token_map()
        .idx
        .on_sale
        .prefix(ON_SALE)
        .range(deps.storage, start, None, Order::Ascending)
        .collect();

    let tokens: Vec<Token> = records?
        .into_iter()
        .filter(|r| r.1.owner == owner)
        .map(|r| r.1)
        .collect();

    let token_res = tokens.into_iter().take(limit).collect();
    Ok(TokensResponse { tokens: token_res })
}

pub fn range_tokens_by_price_asc(
    deps: Deps,
    start_after: Uint128,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let records: StdResult<Vec<_>> = token_map()
        .idx
        .on_sale
        .prefix(ON_SALE)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let mut tokens:Vec<Token>;
    if start_after != Uint128::new(0){
        tokens = records?.into_iter().filter(|r| r.1.price < start_after).map(|r| r.1).collect();
    }else {
        // all tokens
        tokens = records?.into_iter().map(|r| r.1).collect();
    }
    tokens.sort_by(|a, b| b.price.cmp(&a.price));

    let token_res = tokens.into_iter().take(limit).collect();
    Ok(TokensResponse { tokens: token_res })
}

pub fn range_tokens_by_rarity_asc(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    rarity: Vec<String>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let records: StdResult<Vec<_>> = token_map()
        .idx
        .on_sale
        .prefix(ON_SALE)
        .range(deps.storage, start, None, Order::Ascending)
        .collect();

    let mut tokens: Vec<Token> = records?
        .into_iter()
        .filter(|r| rarity.contains(&r.1.rarity))
        .map(|r| r.1)
        .collect();
    tokens.sort_by(|a, b| b.price.cmp(&a.price));

    let token_res = tokens.into_iter().take(limit).collect();
    Ok(TokensResponse { tokens: token_res })
}

pub fn range_tokens_by_price_desc(
    deps: Deps,
    start_after: Uint128,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let records: StdResult<Vec<_>> = token_map()
        .idx
        .on_sale
        .prefix(ON_SALE)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let mut tokens:Vec<Token>;
    if start_after != Uint128::new(0){
        tokens = records?.into_iter().filter(|r| r.1.price > start_after).map(|r| r.1).collect();
    }else {
        // all tokens
        tokens = records?.into_iter().map(|r| r.1).collect();
    }
    tokens.sort_by(|a, b| a.price.cmp(&b.price));

    let token_res = tokens.into_iter().take(limit).collect();
    Ok(TokensResponse { tokens: token_res })

}

pub fn range_tokens_by_rarity_desc(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    rarity: Vec<String>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let records: StdResult<Vec<_>> = token_map()
        .idx
        .on_sale
        .prefix(ON_SALE)
        .range(deps.storage, start, None, Order::Ascending)
        .collect();

    let mut tokens: Vec<Token> = records?
        .into_iter()
        .filter(|r| rarity.contains(&r.1.rarity))
        .map(|r| r.1)
        .collect();
    tokens.sort_by(|a, b| a.price.cmp(&b.price));

    let token_res = tokens.into_iter().take(limit).collect();
    Ok(TokensResponse { tokens: token_res })
}

pub fn get_listed_by_owner(
    deps: Deps,
    owner: String,
) -> StdResult<TokensResponse> {

    let records: StdResult<Vec<_>> = token_map()
        .idx
        .on_sale
        .prefix(ON_SALE)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let tokens: Vec<Token> = records?
        .into_iter()
        .filter(|r| r.1.owner == owner)
        .map(|r| r.1)
        .collect();

    Ok(TokensResponse { tokens })
}

pub fn get_floor_prices(deps: Deps) -> StdResult<FloorPriceResponse> {
    let mut common = Uint128::new(0);
    let mut uncommon = Uint128::new(0);
    let mut rare = Uint128::new(0);
    let mut epic = Uint128::new(0);
    let mut legendary = Uint128::new(0);

    let records: StdResult<Vec<_>> = token_map()
        .idx
        .on_sale
        .prefix(ON_SALE)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let tokens: Vec<Token> = records?.into_iter().map(|r| r.1).collect();

    for token in tokens {
        if token.rarity == "common" {
            if common == Uint128::new(0) || common > token.price {
                common = token.price;
            }
        }
        if token.rarity == "uncommon" {
            if uncommon == Uint128::new(0) || uncommon > token.price {
                uncommon = token.price;
            }
        }
        if token.rarity == "rare" {
            if rare == Uint128::new(0) || rare > token.price {
                rare = token.price;
            }
        }
        if token.rarity == "epic" {
            if epic == Uint128::new(0) || epic > token.price {
                epic = token.price;
            }
        }
        if token.rarity == "legendary" {
            if legendary == Uint128::new(0) || legendary > token.price {
                legendary = token.price;
            }
        }
    }

    Ok(FloorPriceResponse {
        common,
        uncommon,
        rare,
        epic,
        legendary,
    })
}
