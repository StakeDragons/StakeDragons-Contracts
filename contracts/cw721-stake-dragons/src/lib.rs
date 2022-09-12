use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Empty;
pub use cw721_base::{ContractError, InstantiateMsg, MintMsg, MinterResponse, QueryMsg};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Trait {
    pub name: String,
    pub base: String,
    pub accessory: Vec<String>,
    pub background: String,
}

// see: https://docs.opensea.io/docs/metadata-standards
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
pub struct Metadata {
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub name: Option<String>,
    pub attributes: Option<Trait>,
    pub background_color: Option<String>,
    pub animation_url: Option<String>,
    pub youtube_url: Option<String>,
}

pub type Extension = Option<Metadata>;

pub type Cw721StakeDragonsContract<'a> = cw721_base::Cw721Contract<'a, Extension, Empty>;
pub type ExecuteMsg = cw721_base::ExecuteMsg<Extension>;

#[cfg(not(feature = "library"))]
pub mod entry {

    use crate::{Cw721StakeDragonsContract, ExecuteMsg};
    use cosmwasm_std::{entry_point, StdError};
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
    use cw2::set_contract_version;
    use cw721_base::{ContractError, InstantiateMsg, QueryMsg};

    // This is a simple type to let us handle empty extensions

    const CONTRACT_NAME: &str = "crates.io:cw721-stake-dragons";
    const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

    // This makes a conscious choice on the various generics used by the contract
    #[entry_point]
    pub fn instantiate(
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> StdResult<Response> {
        let res = Cw721StakeDragonsContract::default().instantiate(deps.branch(), env, info, msg)?;
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        Ok(res)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        if let ExecuteMsg::Burn { .. } = msg {
            return Err(ContractError::Std(StdError::generic_err(
                "Operation not allowed",
            )));
        }
        Cw721StakeDragonsContract::default().execute(deps, env, info, msg)
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Cw721StakeDragonsContract::default().query(deps, env, msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ExecuteMsg;
    use cosmwasm_std::{to_binary, Addr, CosmosMsg, StdError, WasmMsg};
    use cw721::NftInfoResponse;
    use cw721_base::helpers::Cw721Contract;
    use cw_multi_test::{App, BasicApp, Contract, ContractWrapper, Executor};

    const CREATOR: &str = "creator";

    pub fn contract_cw721_stake_dragons() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(entry::execute, entry::instantiate, entry::query);
        Box::new(contract)
    }

    const TOKEN_ID: &str = "Enterprise";

    fn init() -> (BasicApp, Cw721Contract, MintMsg<Option<Metadata>>) {
        let mut app = App::default();
        let code_id = app.store_code(contract_cw721_stake_dragons());

        let init_msg = InstantiateMsg {
            name: "SpaceShips".to_string(),
            symbol: "SPACE".to_string(),
            minter: CREATOR.to_string(),
        };
        let contract_addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked(CREATOR),
                &init_msg,
                &[],
                "stake_dragons",
                None,
            )
            .unwrap();
        let contract = Cw721Contract(contract_addr);

        let mint_msg = MintMsg {
            token_id: TOKEN_ID.to_string(),
            owner: "john".to_string(),
            token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
            extension: Some(Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Enterprise".to_string()),
                ..Metadata::default()
            }),
        };
        let exec_msg = ExecuteMsg::Mint(mint_msg.clone());
        let cosmos_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract.addr().to_string(),
            msg: to_binary(&exec_msg).unwrap(),
            funds: vec![],
        });
        app.execute(Addr::unchecked(CREATOR), cosmos_msg).unwrap();
        (app, contract, mint_msg)
    }

    #[test]
    fn use_metadata_extension() {
        let (app, contract, mint_msg) = init();

        let res: NftInfoResponse<Extension> = contract
            .nft_info::<String, Extension>(&app.wrap(), TOKEN_ID.into())
            .unwrap();
        assert_eq!(res.token_uri, mint_msg.token_uri);
        assert_eq!(res.extension, mint_msg.extension);
    }

    #[test]
    #[should_panic]
    fn burn_disallowed() {
        let (mut app, contract, _) = init();

        let exec_msg = ExecuteMsg::Burn {
            token_id: TOKEN_ID.to_string(),
        };
        let cosmos_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract.addr().to_string(),
            msg: to_binary(&exec_msg).unwrap(),
            funds: vec![],
        });
        let res = app
            .execute(Addr::unchecked("john"), cosmos_msg)
            .unwrap_err();
        assert_eq!(
            res.to_string(),
            StdError::generic_err("Operation not allowed").to_string()
        );
    }
}
