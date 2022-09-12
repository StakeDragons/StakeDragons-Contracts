use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::ContractError;
use cosmwasm_std::{Addr, Decimal, Deps, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub nft_contract_addr: Addr,
    pub allowed_native: Option<String>,
    pub allowed_cw20: Option<Addr>,
    pub fee_percentage: Decimal,
    pub collector_addr: Addr,
}

pub const MAX_FEE_LIMIT: u64 = 15;

impl Config {
    pub fn new(
        deps: Deps,
        admin: String,
        nft_contract_addr: String,
        allowed_native: Option<String>,
        allowed_cw20: Option<String>,
        fee_percentage: Decimal,
        collector_addr: String,
    ) -> Result<Self, ContractError> {
        let admin = deps.api.addr_validate(&admin)?;
        let nft_contract_addr = deps.api.addr_validate(&nft_contract_addr)?;
        let collector_addr = deps.api.addr_validate(&collector_addr)?;

        if fee_percentage > Decimal::percent(MAX_FEE_LIMIT) {
            return Err(ContractError::WrongInput {});
        }
        let config = match (allowed_native, allowed_cw20) {
            (Some(native_token), None) => Ok(Config {
                admin,
                nft_contract_addr,
                allowed_native: Some(native_token),
                allowed_cw20: None,
                fee_percentage,
                collector_addr,
            }),
            (None, Some(cw20_addr)) => Ok(Config {
                admin,
                nft_contract_addr,
                allowed_native: None,
                allowed_cw20: Some(deps.api.addr_validate(&cw20_addr)?),
                fee_percentage,
                collector_addr,
            }),
            _ => Err(ContractError::InvalidTokenType {}),
        }?;
        Ok(config)
    }
}

pub const CONFIG: Item<Config> = Item::new("config");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token {
    pub id: String,
    pub price: Uint128,
    pub on_sale: bool,
    pub rarity: String,
    pub owner: String,
    pub ovulation_period: String,
    pub daily_reward: String,
}

pub struct TokenIndexes<'a> {
    pub on_sale: MultiIndex<'a, &'a [u8], Token>,
}

impl<'a> IndexList<Token> for TokenIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Token>> + '_> {
        let v: Vec<&dyn Index<Token>> = vec![&self.on_sale];
        Box::new(v.into_iter())
    }
}

pub const ON_SALE: &[u8] = &[0u8];
pub const NON_SALE: &[u8] = &[1u8];

pub fn token_map<'a>() -> IndexedMap<'a, String, Token, TokenIndexes<'a>> {
    let indexes = TokenIndexes {
        on_sale: MultiIndex::new(
            |d: &Token| if d.on_sale { ON_SALE } else { NON_SALE },
            "tokens",
            "tokens__on_sale",
        ),
    };
    IndexedMap::new("tokens", indexes)
}
