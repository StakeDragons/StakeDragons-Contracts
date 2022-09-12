#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query_members, query_state};
    use crate::msg::{CustomMintMsg, ExecuteMsg, InstantiateMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::DepsMut;
    use cw721_base::msg::InstantiateMsg as Cw721InstantiateMsg;
    use cw721_base::MintMsg;

    const ADMIN: &str = "minter";

    fn setup_contract(deps: DepsMut) {
        let msg = InstantiateMsg {
            base: Cw721InstantiateMsg {
                name: "STARTER DRAGON".to_string(),
                symbol: "STRTR".to_string(),
                minter: "minter".to_string(),
            },
            members: vec!["adsfsa".to_string()],
        };
        let info = mock_info(ADMIN, &[]);
        let res = instantiate(deps, mock_env(), info, msg).unwrap();
        assert_eq!(1, res.attributes.len());
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());
    }

    #[test]
    fn improper_initialization_dedup() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            base: Cw721InstantiateMsg {
                name: "STARTER DRAGON".to_string(),
                symbol: "STRTR".to_string(),
                minter: "minter".to_string(),
            },
            members: vec![
                "adsfsa".to_string(),
                "adsfsa".to_string(),
                "eddddd".to_string(),
                "adssss".to_string(),
                "adssss".to_string(),
                "eddddd".to_string(),
            ],
        };
        let info = mock_info(ADMIN, &[]);
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        let res = query_state(deps.as_ref()).unwrap();
        println!("PRINT STATE -> size: {} -  name: {}", res.size, res.name);
        assert_eq!(3, res.size);
    }

    #[test]
    fn update_members() {
        let mut deps = mock_dependencies();
        setup_contract(deps.as_mut());

        // dedupe addrs
        let msg = ExecuteMsg::AddMembers {
            members: vec![
                "eeeeee".to_string(),
                "eeeeee".to_string(),
                "ffffff".to_string(),
                "wwwwww".to_string(),
                "rrrrrr".to_string(),
                "wwwwww".to_string(),
            ],
        };
        let info = mock_info(ADMIN, &[]);
        execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
        let res = query_members(deps.as_ref(), None, None).unwrap();
        println!(
            "PRINT AFTER ADD MEMBER -> member vector length: {}",
            res.members.len()
        );

        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();

        let msg = ExecuteMsg::RemoveMembers {
            members: vec!["eeeeee".to_string()],
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let res = query_members(deps.as_ref(), None, None).unwrap();
        println!(
            "PRINT AFTER REMOVE MEMBER -> member vector length: {}",
            res.members.len()
        );

        let token_id = "petrify".to_string();
        let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();

        println!("MINT FOR eeeeee should return error ");
        let mint_msg = CustomMintMsg {
            base: MintMsg {
                token_id: token_id.clone(),
                owner: String::from("eeeeee"),
                token_uri: Some(token_uri.clone()),
                extension: None,
            },
        };
        let msg = ExecuteMsg::Mint(mint_msg);
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();

        let res = query_state(deps.as_ref()).unwrap();
        println!("after eeee mint {}", res.claimed_dragons);

        println!("MINT FOR rrrrrr should work ");
        let mint_msg = CustomMintMsg {
            base: MintMsg {
                token_id: token_id.clone(),
                owner: String::from("rrrrrr"),
                token_uri: Some(token_uri.clone()),
                extension: None,
            },
        };
        let msg = ExecuteMsg::Mint(mint_msg);
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let res = query_state(deps.as_ref()).unwrap();
        println!("after rrrrrr mint {}", res.claimed_dragons);

        println!("MINT FOR rrrrrr second time should not work ");
        let mint_msg = CustomMintMsg {
            base: MintMsg {
                token_id: token_id.clone(),
                owner: String::from("rrrrrr"),
                token_uri: Some(token_uri.clone()),
                extension: None,
            },
        };
        let msg = ExecuteMsg::Mint(mint_msg);
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();

        let res = query_state(deps.as_ref()).unwrap();
        println!("after rrrrrr mint again {}", res.claimed_dragons);
    }
}
