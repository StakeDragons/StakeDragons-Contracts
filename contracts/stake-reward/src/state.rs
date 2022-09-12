use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner: String,
    pub dragon_contract: String,
    pub cw20_contract: Addr,
    pub admin: String,
}

pub const STATE: Item<State> = Item::new("state");
