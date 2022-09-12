use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub name: String,
    pub symbol: String,
    pub minter: String,
    pub size: u32,
    pub claimed_dragons: u32,
}

pub const STATE: Item<State> = Item::new("state");
pub const WHITELIST: Map<Addr, bool> = Map::new("whitelist");
