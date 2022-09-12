#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query, query_owned_egg_count};
    use crate::msg::{CustomMintMsg, ExecuteMsg, Extension, InstantiateMsg, QueryMsg};
    use std::fs::read_to_string;
    //use crate::state::{CollectionInfo, Egg, COLLECTION_INFO, OWNED_EGG_COUNT};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Empty, Uint64};
    use cw721::{ContractInfoResponse, Cw721Query};
    use cw721_base::msg::InstantiateMsg as Cw721InstantiateMsg;
    use cw721_base::{Cw721Contract, MintMsg};

    #[test]
    fn proper_initialization() {
        let contract = Cw721Contract::<Extension, Empty>::default();

        let mut deps = mock_dependencies();
        let collection = String::from("collection0");

        let msg = InstantiateMsg {
            base: Cw721InstantiateMsg {
                name: collection,
                symbol: String::from("EGG"),
                minter: String::from("creator"),
            },
            base_price: Uint64::new(1),
            size: Uint64::new(100),
        };

        let info = mock_info("creator", &coins(1000000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(1, res.attributes.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Minter {}).unwrap();
        println!("minter {}", res);

        let token_id = "petrify".to_string();
        let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();

        println!("MINT FOR MEDUSA 1 ");
        let mint_msg = CustomMintMsg {
            base: MintMsg {
                token_id: token_id.clone(),
                owner: String::from("medusa"),
                token_uri: Some(token_uri.clone()),
                extension: None,
            },
            hatch: None,
        };
        let msg = ExecuteMsg::Mint(mint_msg);

        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        println!("MINT FOR MEDUSA 2 ");
        let token_id_2 = "petrify2".to_string();
        let token_uri_2 = "https://www.merriam-webster.com/dictionary/petrify2".to_string();

        let mint_msg = CustomMintMsg {
            base: MintMsg {
                token_id: token_id_2.clone(),
                owner: String::from("medusa"),
                token_uri: Some(token_uri_2.clone()),
                extension: None,
            },
            hatch: None,
        };
        let msg2 = ExecuteMsg::Mint(mint_msg);

        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg2).unwrap();

        println!("MINT FOR ASD  ");
        let token_id_3 = "petrify3".to_string();
        let token_uri_3 = "https://www.merriam-webster.com/dictionary/petrify3".to_string();

        let mint_msg = CustomMintMsg {
            base: MintMsg {
                token_id: token_id_3.clone(),
                owner: String::from("asd"),
                token_uri: Some(token_uri_3.clone()),
                extension: None,
            },
            hatch: None,
        };
        let msg = ExecuteMsg::Mint(mint_msg);

        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        println!("ASD OWNED TOKENS ");
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Tokens {
                owner: "asd".to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        println!("{}", res);

        println!("ALL TOKENS ");
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AllTokens {
                start_after: None,
                limit: None,
            },
        )
        .unwrap()
        .to_string();
        println!("{}", res.len());

        println!("MEDUSA OWNED TOKENS ");

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Tokens {
                owner: "medusa".to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        println!("MEDUSA OWNED TOKENS {}", res);

        println!("NFT info token2 ");

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::NftInfo {
                token_id: token_id_2,
            },
        )
        .unwrap()
        .to_string();
        println!("{}", res);

        /*
        /// HATCH ASD OWNED TOKEN
        let res = execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::Hatch {
                token_id: token_id_3.to_string(),
            },
        )
        .unwrap();

         */

        println!("ASD OWNED TOKENS ");
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::Tokens {
                owner: "asd".to_string(),
                start_after: None,
                limit: None,
            },
        )
        .unwrap();
        println!("{}", res);

        let res = query_owned_egg_count(deps.as_ref()).unwrap();
        println!("owned egg count from state {}", res.owned);
    }
}
