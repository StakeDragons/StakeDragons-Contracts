use cosmwasm_std::Uint64;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionInfo {
    pub name: String,
    pub symbol: String,
    pub minter: String,
    pub description: String,
    pub size: Uint64,
    pub base_price: Uint64,
}

pub const COLLECTION_INFO: Item<CollectionInfo> = Item::new("collection_info");
pub const OWNED_EGG_COUNT: Item<Uint64> = Item::new("owned_egg_count");
