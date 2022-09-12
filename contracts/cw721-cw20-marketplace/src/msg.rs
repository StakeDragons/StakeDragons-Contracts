use crate::state::{Config, Token};

use cosmwasm_std::{Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub admin: String,
    pub nft_addr: String,
    pub allowed_native: Option<String>,
    pub allowed_cw20: Option<String>,
    pub fee_percentage: Decimal,
    pub collector_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// ListTokens registers or relists tokens
    ListTokens {
        tokens: Vec<Token>,
    },
    /// Delist tokens removes tokens from marketplace
    DelistTokens {
        tokens: Vec<String>,
    },
    UpdatePrice {
        token: String,
        price: Uint128,
    },
    UpdateConfig {
        admin: Option<String>,
        nft_addr: Option<String>,
        allowed_native: Option<String>,
        allowed_cw20: Option<String>,
        fee_percentage: Option<Decimal>,
        collector_addr: Option<String>,
    },
    Receive(Cw20ReceiveMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Buy { recipient: String, token_id: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Token {
        id: String,
    },
    RangeTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    ListTokens {
        ids: Vec<String>,
    },
    ListTokensOnSale {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    ListByPriceAsc {
        start_after: Uint128,
        limit: Option<u32>,
    },
    ListByPriceDesc {
        start_after: Uint128,
        limit: Option<u32>,
    },
    ListByRarity {
        start_after: Option<String>,
        limit: Option<u32>,
        rarity: Vec<String>,
    },
    ListByRarityAsc {
        start_after: Option<String>,
        limit: Option<u32>,
        rarity: Vec<String>,
    },
    ListByRarityDesc {
        start_after: Option<String>,
        limit: Option<u32>,
        rarity: Vec<String>,
    },
    ListByOwner {
        start_after: Option<String>,
        limit: Option<u32>,
        owner: String,
    },
    GetListedSize {
        start_after: Option<String>,
    },
    GetFloorPrices {},
    GetListedTokensByOwner{owner:String},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub config: Config,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenResponse {
    pub token: Token,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokensResponse {
    pub tokens: Vec<Token>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct FloorPriceResponse {
    pub common: Uint128,
    pub uncommon: Uint128,
    pub rare: Uint128,
    pub epic: Uint128,
    pub legendary: Uint128,
}
