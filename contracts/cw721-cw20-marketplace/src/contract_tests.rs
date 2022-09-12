#[cfg(test)]
mod tests {
    use crate::helpers::Cw721MarketplaceContract;
    use crate::msg::{ExecuteMsg, QueryMsg, TokensResponse};
    use crate::ContractError;
    use anyhow::{anyhow, Result};
    use derivative::Derivative;

    use cosmwasm_std::{
        to_binary, Addr, Binary, Coin, Decimal, Empty, QueryRequest, StdError, Uint128, WasmQuery,
    };
    use cw20::Cw20Contract;
    use cw20::{BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20ReceiveMsg};
    use cw721_base::Extension;

    use cw_multi_test::{App, AppResponse, Contract, ContractWrapper, Executor};
    use serde::de::DeserializeOwned;

    use crate::state::Token;
    use cw721_base::helpers::Cw721Contract;
    use cw721_stake_dragons;

    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

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

    pub fn contract_cw20() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_base::contract::execute,
            cw20_base::contract::instantiate,
            cw20_base::contract::query,
        );
        Box::new(contract)
    }

    pub fn contract_marketplace() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            crate::execute::execute,
            crate::execute::instantiate,
            crate::query::query,
        );
        Box::new(contract)
    }
    pub fn contract_cw721_stake_dragons() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw721_stake_dragons::entry::execute,
            cw721_stake_dragons::entry::instantiate,
            cw721_stake_dragons::entry::query,
        );
        Box::new(contract)
    }

    const OWNER: &str = "owner";
    const OWNER_INIT_BALANCE: u128 = 300u128;
    const MINTER: &str = "minter";
    const ADMIN: &str = "admin";
    const RANDOM: &str = "random";
    const ALLOWED_NATIVE: &str = "ujuno";
    const ALLOWED_CW20: &str = "contract0";
    const ALLOWED_CW20_OWNER: &str = "cw20_owner";
    const NOT_ALLOWED_CW20: &str = "another_contract";
    const COLLECTOR: &str = "collector";

    const TOKEN_ID1: &str = "token1";
    const TOKEN_ID2: &str = "token2";
    const TOKEN_ID3: &str = "token3";

    fn mock_app() -> App {
        App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(RANDOM),
                    vec![
                        Coin {
                            denom: "ujuno".into(),
                            amount: Uint128::new(50000000),
                        },
                        Coin {
                            denom: "zuhaha".into(),
                            amount: Uint128::new(400),
                        },
                    ],
                )
                .unwrap();

            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(OWNER),
                    vec![Coin {
                        denom: "ujuno".into(),
                        amount: Uint128::new(OWNER_INIT_BALANCE),
                    }],
                )
                .unwrap();
        })
    }

    #[derive(Derivative)]
    #[derivative(Debug)]
    pub struct Suite {
        /// Application mock
        #[derivative(Debug = "ignore")]
        pub app: App,
        /// Special account
        pub owner: String,

        nft_code_id: u64,
        marketplace_code_id: u64,
    }

    #[allow(dead_code)]
    impl Suite {
        pub fn init() -> Result<Suite> {
            let mut app = mock_app();
            let owner = OWNER.to_owned();
            let nft_code_id = app.store_code(contract_cw721_stake_dragons());
            let marketplace_code_id = app.store_code(contract_marketplace());
            let _cw20_code_id = app.store_code(contract_cw20());

            Ok(Suite {
                app,
                owner,
                nft_code_id,
                marketplace_code_id,
            })
        }

        fn instantiate_cw20(&mut self, cw20_owner: Addr) -> Addr {
            let cw20_code_id = self.app.store_code(contract_cw20());
            let msg = cw20_base::msg::InstantiateMsg {
                name: "Cash Money".to_string(),
                symbol: "CASH".to_string(),
                decimals: 2,
                initial_balances: vec![
                    Cw20Coin {
                        address: cw20_owner.to_string(),
                        amount: Uint128::new(5000),
                    },
                    Cw20Coin {
                        address: OWNER.to_string(),
                        amount: Uint128::new(OWNER_INIT_BALANCE),
                    },
                ],
                mint: None,
                marketing: None,
            };

            self.app
                .instantiate_contract(cw20_code_id, cw20_owner, &msg, &[], "CASH", None)
                .unwrap()
        }

        fn instantiate_nft(&mut self, minter: String) -> Cw721Contract {
            let nft_id = self.app.store_code(contract_cw721_stake_dragons());
            let msg = cw721_base::InstantiateMsg {
                name: "Stake Dragons".to_string(),
                symbol: "SDR".to_string(),
                minter: minter.clone(),
            };
            Cw721Contract(
                self.app
                    .instantiate_contract(nft_id, Addr::unchecked(minter), &msg, &[], "flex", None)
                    .unwrap(),
            )
        }

        fn instantiate_marketplace(
            &mut self,
            nft_addr: String,
            allowed_native: Option<String>,
            allowed_cw20: Option<String>,
            fee_percentage: Decimal,
        ) -> Result<Cw721MarketplaceContract> {
            let marketplace_id = self.app.store_code(contract_marketplace());
            let msg = crate::msg::InstantiateMsg {
                admin: String::from(ADMIN),
                nft_addr,
                allowed_native,
                allowed_cw20,
                fee_percentage,
                collector_addr: String::from(COLLECTOR),
            };
            let contract = self.app.instantiate_contract(
                marketplace_id,
                Addr::unchecked(ADMIN),
                &msg,
                &[],
                "flex",
                None,
            );
            contract.map(Cw721MarketplaceContract)
        }

        fn proper_instantiate_native(&mut self) -> (Cw721Contract, Cw721MarketplaceContract) {
            // setup nft contract
            let nft = self.instantiate_nft(String::from(MINTER));
            let mint_msg: cw721_base::msg::MintMsg<Metadata> = cw721_base::MintMsg {
                token_id: TOKEN_ID1.to_string(),
                owner: OWNER.to_string(),
                token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
                extension: Metadata {
                    description: Some("Spaceship with Warp Drive".into()),
                    name: Some("Starship USS Enterprise".to_string()),
                    ..Metadata::default()
                },
            };
            let exec_msg = cw721_base::ExecuteMsg::Mint(mint_msg);
            let cosmos_msg = nft.call(exec_msg).unwrap();
            self.app
                .execute(Addr::unchecked(MINTER), cosmos_msg)
                .unwrap();
            let marketplace = self.instantiate_marketplace(
                nft.addr().into(),
                Some(String::from(ALLOWED_NATIVE)),
                None,
                Decimal::from_ratio(3u64, 100u64),
            );
            (nft, marketplace.unwrap())
        }
        fn proper_instantiate_cw20(&mut self) -> (Cw721Contract, Cw721MarketplaceContract, Addr) {
            //set up cw20 contract
            let cw20_addr = self.instantiate_cw20(Addr::unchecked(ALLOWED_CW20_OWNER));
            // setup nft contract
            let nft = self.instantiate_nft(String::from(MINTER));
            let mint_msg: cw721_base::msg::MintMsg<Metadata> = cw721_base::MintMsg {
                token_id: TOKEN_ID1.to_string(),
                owner: OWNER.to_string(),
                token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
                extension: Metadata {
                    description: Some("Spaceship with Warp Drive".into()),
                    name: Some("Starship USS Enterprise".to_string()),
                    ..Metadata::default()
                },
            };
            let exec_msg = cw721_base::ExecuteMsg::Mint(mint_msg);
            let cosmos_msg = nft.call(exec_msg).unwrap();
            self.app
                .execute(Addr::unchecked(MINTER), cosmos_msg)
                .unwrap();
            let marketplace = self.instantiate_marketplace(
                nft.addr().into(),
                None,
                Some(String::from(ALLOWED_CW20)),
                Decimal::from_ratio(3u64, 100u64),
            );
            (nft, marketplace.unwrap(), cw20_addr)
        }

        fn failed_instantiate_native_cw20(&mut self) -> (Cw721Contract, Cw721MarketplaceContract) {
            // setup nft contract
            let nft = self.instantiate_nft(String::from(MINTER));
            let mint_msg: cw721_base::msg::MintMsg<Metadata> = cw721_base::MintMsg {
                token_id: TOKEN_ID1.to_string(),
                owner: OWNER.to_string(),
                token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
                extension: Metadata {
                    description: Some("Spaceship with Warp Drive".into()),
                    name: Some("Starship USS Enterprise".to_string()),
                    ..Metadata::default()
                },
            };
            let exec_msg = cw721_base::ExecuteMsg::Mint(mint_msg);
            let cosmos_msg = nft.call(exec_msg).unwrap();
            self.app
                .execute(Addr::unchecked(MINTER), cosmos_msg)
                .unwrap();
            let marketplace = self.instantiate_marketplace(
                nft.addr().into(),
                Some(String::from(ALLOWED_NATIVE)),
                Some(String::from(ALLOWED_CW20)),
                Decimal::from_ratio(3u64, 100u64),
            );
            (nft, marketplace.unwrap())
        }

        pub fn execute<M>(
            &mut self,
            sender: Addr,
            contract_addr: Addr,
            msg: ExecuteMsg,
            _funds: Vec<Coin>,
        ) -> Result<AppResponse>
        where
            M: Serialize + DeserializeOwned,
        {
            self.app
                .execute_contract(sender, contract_addr, &msg, &[])
                .map_err(|err| anyhow!(err))
        }

        pub fn query<M>(&self, target_contract: Addr, msg: M) -> Result<M, StdError>
        where
            M: Serialize + DeserializeOwned,
        {
            self.app.wrap().query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: target_contract.to_string(),
                msg: to_binary(&msg).unwrap(),
            }))
        }
    }
    #[test]
    #[should_panic]
    fn failed_instantiation() {
        let mut suite = Suite::init().unwrap();
        let (_nft_contract, _marketplace_contract) = suite.failed_instantiate_native_cw20();
    }

    #[test]
    fn test_register_tokens() {
        let mut suite = Suite::init().unwrap();
        let (_nft_contract, marketplace_contract) = suite.proper_instantiate_native();

        // empty tokens throw error
        let msg = marketplace_contract
            .call(ExecuteMsg::ListTokens { tokens: vec![] }, vec![])
            .unwrap();
        let res = suite.app.execute(Addr::unchecked(ADMIN), msg).unwrap_err();
        assert_eq!(ContractError::WrongInput {}, res.downcast().unwrap());

        // only admin can register tokens
        let token = Token {
            id: TOKEN_ID1.into(),
            price: Default::default(),
            on_sale: true,
            rarity: "".to_string(),
        };
        let msg = marketplace_contract
            .call(
                ExecuteMsg::ListTokens {
                    tokens: vec![token],
                },
                vec![],
            )
            .unwrap();
        let res = suite
            .app
            .execute(Addr::unchecked(RANDOM), msg.clone())
            .unwrap_err();
        assert_eq!(ContractError::Unauthorized {}, res.downcast().unwrap());

        // admin can register token
        suite.app.execute(Addr::unchecked(ADMIN), msg).unwrap();
    }

    #[test]
    fn test_list_tokens() {
        let mut suite = Suite::init().unwrap();
        let (nft_contract, marketplace_contract) = suite.proper_instantiate_native();

        // empty tokens throw error
        let msg = marketplace_contract
            .call(ExecuteMsg::ListTokens { tokens: vec![] }, vec![])
            .unwrap();
        let res = suite.app.execute(Addr::unchecked(ADMIN), msg).unwrap_err();
        assert_eq!(ContractError::WrongInput {}, res.downcast().unwrap());

        let token = Token {
            id: String::from(TOKEN_ID1),
            price: Uint128::new(100),
            on_sale: true,
            rarity: "".to_string(),
        };
        let msg = marketplace_contract
            .call(
                ExecuteMsg::ListTokens {
                    tokens: vec![token.clone()],
                },
                vec![],
            )
            .unwrap();

        // register token
        suite
            .app
            .execute(Addr::unchecked(ADMIN), msg.clone())
            .unwrap();

        // only token owner can list
        let res = suite
            .app
            .execute(Addr::unchecked(RANDOM), msg.clone())
            .unwrap_err();
        assert_eq!(ContractError::Unauthorized {}, res.downcast().unwrap());

        // non approved tokens are not accepted
        let res = suite.app.execute(Addr::unchecked(OWNER), msg).unwrap_err();
        assert_eq!(ContractError::NotApproved {}, res.downcast().unwrap());

        // marketplace contract is not spender
        let exec_msg: cw721_base::ExecuteMsg<Extension> = cw721_base::ExecuteMsg::Approve {
            spender: RANDOM.into(),
            token_id: TOKEN_ID1.into(),
            expires: None,
        };
        let msg = nft_contract.call(exec_msg).unwrap();
        suite.app.execute(Addr::unchecked(OWNER), msg).unwrap();

        let msg = marketplace_contract
            .call(
                ExecuteMsg::ListTokens {
                    tokens: vec![token.clone()],
                },
                vec![],
            )
            .unwrap();
        let res = suite.app.execute(Addr::unchecked(OWNER), msg).unwrap_err();
        assert_eq!(ContractError::NotApproved {}, res.downcast().unwrap());

        // marketplace contract is spender, happy path
        let exec_msg = cw721_base::ExecuteMsg::<Extension>::Approve {
            spender: marketplace_contract.addr().into(),
            token_id: token.id.clone(),
            expires: None,
        };
        let msg = nft_contract.call(exec_msg).unwrap();
        suite.app.execute(Addr::unchecked(OWNER), msg).unwrap();

        let msg = marketplace_contract
            .call(
                ExecuteMsg::ListTokens {
                    tokens: vec![token.clone()],
                },
                vec![],
            )
            .unwrap();
        suite.app.execute(Addr::unchecked(OWNER), msg).unwrap();

        let t = marketplace_contract.token(&suite.app, TOKEN_ID1).unwrap();
        assert_eq!(t.token, token)
    }

    #[test]
    fn test_delist_token() {
        let mut suite = Suite::init().unwrap();
        let (_nft_contract, marketplace_contract) = suite.proper_instantiate_native();

        // list token
        let token = Token {
            id: TOKEN_ID1.into(),
            price: Default::default(),
            on_sale: true,
            rarity: "".to_string(),
        };
        let msg = marketplace_contract
            .call(
                ExecuteMsg::ListTokens {
                    tokens: vec![token.clone()],
                },
                vec![],
            )
            .unwrap();
        suite.app.execute(Addr::unchecked(ADMIN), msg).unwrap();

        let msg = marketplace_contract
            .call(
                ExecuteMsg::DelistTokens {
                    tokens: vec![token.id.clone()],
                },
                vec![],
            )
            .unwrap();

        // only owner can delist
        let res = suite
            .app
            .execute(Addr::unchecked(RANDOM), msg.clone())
            .unwrap_err();
        assert_eq!(ContractError::Unauthorized {}, res.downcast().unwrap());

        // happy path
        suite.app.execute(Addr::unchecked(OWNER), msg).unwrap();

        let t = marketplace_contract.token(&suite.app, TOKEN_ID1).unwrap();
        assert_eq!(
            t.token,
            Token {
                id: token.id,
                price: token.price,
                on_sale: false,
                rarity: "".to_string()
            }
        )
    }

    #[test]
    fn test_change_price() {
        let mut suite = Suite::init().unwrap();
        let (nft_contract, marketplace_contract) = suite.proper_instantiate_native();

        let token = Token {
            id: TOKEN_ID1.into(),
            price: Uint128::new(1),
            on_sale: true,
            rarity: "".to_string(),
        };
        let msg = marketplace_contract
            .call(
                ExecuteMsg::ListTokens {
                    tokens: vec![token.clone()],
                },
                vec![],
            )
            .unwrap();
        suite.app.execute(Addr::unchecked(ADMIN), msg).unwrap();

        let msg = marketplace_contract
            .call(
                ExecuteMsg::UpdatePrice {
                    token: TOKEN_ID1.into(),
                    price: Uint128::new(100),
                },
                vec![],
            )
            .unwrap();

        // only approved can update price
        let res = suite
            .app
            .execute(Addr::unchecked(RANDOM), msg.clone())
            .unwrap_err();
        assert_eq!(ContractError::Unauthorized {}, res.downcast().unwrap());

        // RANDOM now approved
        let exec_msg: cw721_base::ExecuteMsg<Extension> = cw721_base::ExecuteMsg::Approve {
            spender: RANDOM.into(),
            token_id: TOKEN_ID1.into(),
            expires: None,
        };
        let cosmos_msg = nft_contract.call(exec_msg).unwrap();
        suite
            .app
            .execute(Addr::unchecked(OWNER), cosmos_msg)
            .unwrap();

        // happy path
        suite.app.execute(Addr::unchecked(RANDOM), msg).unwrap();

        let t = marketplace_contract.token(&suite.app, TOKEN_ID1).unwrap();
        assert_eq!(
            t.token,
            Token {
                id: token.id,
                price: Uint128::new(100),
                on_sale: true,
                rarity: "".to_string()
            }
        )
    }

    #[test]
    fn test_delist_and_register() {
        let mut suite = Suite::init().unwrap();
        let (nft_contract, marketplace_contract) = suite.proper_instantiate_native();

        // list token
        let mut token = Token {
            id: TOKEN_ID1.into(),
            price: Uint128::new(100),
            on_sale: true,
            rarity: "".to_string(),
        };

        // owner approves
        let exec_msg = cw721_base::ExecuteMsg::<Extension>::Approve {
            spender: marketplace_contract.addr().into(),
            token_id: token.id.clone(),
            expires: None,
        };
        let msg = nft_contract.call(exec_msg).unwrap();
        suite.app.execute(Addr::unchecked(OWNER), msg).unwrap();

        // admin lists
        let msg = marketplace_contract
            .call(
                ExecuteMsg::ListTokens {
                    tokens: vec![token.clone()],
                },
                vec![],
            )
            .unwrap();
        suite.app.execute(Addr::unchecked(ADMIN), msg).unwrap();
        // owner delists
        let msg = marketplace_contract
            .call(
                ExecuteMsg::DelistTokens {
                    tokens: vec![token.id.clone()],
                },
                vec![],
            )
            .unwrap();
        suite.app.execute(Addr::unchecked(OWNER), msg).unwrap();

        let new_price = Uint128::new(14);
        token.price = new_price;
        // owner lists
        let msg = marketplace_contract
            .call(
                ExecuteMsg::ListTokens {
                    tokens: vec![token],
                },
                vec![],
            )
            .unwrap();
        suite.app.execute(Addr::unchecked(OWNER), msg).unwrap();

        let t = marketplace_contract.token(&suite.app, TOKEN_ID1).unwrap();
        assert_eq!(t.token.price, new_price)
    }

    #[test]
    fn test_buy_native() {
        let mut suite = Suite::init().unwrap();
        let (nft_contract, marketplace_contract) = suite.proper_instantiate_native();

        let price = Uint128::new(100);
        let token = Token {
            id: TOKEN_ID1.into(),
            price,
            on_sale: true,
            rarity: "".to_string(),
        };
        let msg = marketplace_contract
            .call(
                ExecuteMsg::ListTokens {
                    tokens: vec![token.clone()],
                },
                vec![],
            )
            .unwrap();
        suite.app.execute(Addr::unchecked(ADMIN), msg).unwrap();

        // approve marketplace
        let exec_msg = cw721_base::ExecuteMsg::<Extension>::Approve {
            spender: marketplace_contract.addr().into(),
            token_id: token.id,
            expires: None,
        };
        let msg = nft_contract.call(exec_msg).unwrap();
        suite.app.execute(Addr::unchecked(OWNER), msg).unwrap();

        // no tokens
        let msg = marketplace_contract
            .call(
                ExecuteMsg::Buy {
                    recipient: None,
                    token_id: TOKEN_ID1.into(),
                },
                vec![],
            )
            .unwrap();
        let res = suite.app.execute(Addr::unchecked(RANDOM), msg).unwrap_err();
        assert_eq!(
            ContractError::SendSingleNativeToken {},
            res.downcast().unwrap()
        );

        // multiple tokens
        let msg = marketplace_contract
            .call(
                ExecuteMsg::Buy {
                    recipient: None,
                    token_id: TOKEN_ID1.into(),
                },
                vec![
                    Coin {
                        denom: "ujuno".into(),
                        amount: Uint128::new(2),
                    },
                    Coin {
                        denom: "zuhaha".into(),
                        amount: Uint128::new(2),
                    },
                ],
            )
            .unwrap();
        let res = suite.app.execute(Addr::unchecked(RANDOM), msg).unwrap_err();
        assert_eq!(
            ContractError::SendSingleNativeToken {},
            res.downcast().unwrap()
        );

        // disallowed native token
        let msg = marketplace_contract
            .call(
                ExecuteMsg::Buy {
                    recipient: None,
                    token_id: TOKEN_ID1.into(),
                },
                vec![Coin {
                    denom: "zuhaha".into(),
                    amount: Uint128::new(1),
                }],
            )
            .unwrap();
        let res = suite.app.execute(Addr::unchecked(RANDOM), msg).unwrap_err();
        assert_eq!(
            ContractError::NativeDenomNotAllowed {
                denom: "zuhaha".into()
            },
            res.downcast().unwrap()
        );

        // wrong fund amount
        let msg = marketplace_contract
            .call(
                ExecuteMsg::Buy {
                    recipient: None,
                    token_id: TOKEN_ID1.into(),
                },
                vec![Coin {
                    denom: ALLOWED_NATIVE.into(),
                    amount: Uint128::new(200),
                }],
            )
            .unwrap();
        let res = suite.app.execute(Addr::unchecked(RANDOM), msg).unwrap_err();
        assert_eq!(
            ContractError::SentWrongFundsAmount {
                need: Uint128::new(100),
                sent: Uint128::new(200),
            },
            res.downcast().unwrap()
        );

        // wrong coin amount
        let msg = marketplace_contract
            .call(
                ExecuteMsg::Buy {
                    recipient: None,
                    token_id: TOKEN_ID1.into(),
                },
                vec![Coin {
                    denom: ALLOWED_NATIVE.into(),
                    amount: Uint128::new(10000),
                }],
            )
            .unwrap();
        let res = suite.app.execute(Addr::unchecked(RANDOM), msg).unwrap_err();
        assert_eq!(
            ContractError::SentWrongFundsAmount {
                need: Uint128::new(100),
                sent: Uint128::new(10000)
            },
            res.downcast().unwrap()
        );

        let raw_price = Uint128::new(100u128);
        let fee = Uint128::new(3u128);
        let owner_payout = Uint128::new(97u128);
        // happy path
        let msg = marketplace_contract
            .call(
                ExecuteMsg::Buy {
                    recipient: None,
                    token_id: TOKEN_ID1.into(),
                },
                vec![Coin {
                    denom: ALLOWED_NATIVE.into(),
                    amount: raw_price,
                }],
            )
            .unwrap();
        suite.app.execute(Addr::unchecked(RANDOM), msg).unwrap();

        // collector balance updated
        let collector_balance = suite
            .app
            .wrap()
            .query_balance(COLLECTOR, ALLOWED_NATIVE)
            .unwrap();
        assert_eq!(collector_balance.amount, fee);

        // nft owner updated
        let res = nft_contract
            .owner_of(&suite.app.wrap(), TOKEN_ID1, false)
            .unwrap();
        assert_eq!(res.owner, String::from(RANDOM));

        // owner balance updated
        let owner_balance = suite
            .app
            .wrap()
            .query_balance(OWNER, ALLOWED_NATIVE)
            .unwrap();
        assert_eq!(
            owner_balance.amount.u128(),
            OWNER_INIT_BALANCE + owner_payout.u128()
        );
    }

    #[test]
    fn test_buy_cw20() {
        let mut suite = Suite::init().unwrap();
        let (nft_contract, marketplace_contract, cw20_addr) = suite.proper_instantiate_cw20();

        let price = Uint128::new(100);
        let token = Token {
            id: TOKEN_ID1.into(),
            price,
            on_sale: true,
            rarity: "".to_string(),
        };
        let msg = marketplace_contract
            .call(
                ExecuteMsg::ListTokens {
                    tokens: vec![token.clone()],
                },
                vec![],
            )
            .unwrap();
        suite.app.execute(Addr::unchecked(ADMIN), msg).unwrap();

        // approve marketplace
        let exec_msg = cw721_base::ExecuteMsg::<Extension>::Approve {
            spender: marketplace_contract.addr().into(),
            token_id: token.id,
            expires: None,
        };
        let msg = nft_contract.call(exec_msg).unwrap();
        suite.app.execute(Addr::unchecked(OWNER), msg).unwrap();

        let send_msg =
            Binary::from(r#"{"buy":{"recipient":"my_addr", "token_id":"token1"}}"#.as_bytes());
        // no tokens
        let msg = marketplace_contract
            .call(
                ExecuteMsg::Receive {
                    0: Cw20ReceiveMsg {
                        sender: "my_addr".to_string(),
                        amount: Uint128::zero(),
                        msg: send_msg.clone(),
                    },
                },
                vec![],
            )
            .unwrap();
        let res = suite
            .app
            .execute(Addr::unchecked(ALLOWED_CW20), msg.clone())
            .unwrap_err();
        assert_eq!(
            ContractError::SentWrongFundsAmount {
                need: Uint128::new(100),
                sent: Uint128::zero()
            },
            res.downcast().unwrap()
        );

        // Wrong amount of tokens
        let msg = marketplace_contract
            .call(
                ExecuteMsg::Receive {
                    0: Cw20ReceiveMsg {
                        sender: "my_addr".to_string(),
                        amount: Uint128::new(150),
                        msg: send_msg.clone(),
                    },
                },
                vec![],
            )
            .unwrap();
        let res = suite
            .app
            .execute(Addr::unchecked(ALLOWED_CW20), msg.clone())
            .unwrap_err();
        assert_eq!(
            ContractError::SentWrongFundsAmount {
                need: Uint128::new(100),
                sent: Uint128::new(150)
            },
            res.downcast().unwrap()
        );

        // Disallowed CW20 address
        let res = suite
            .app
            .execute(Addr::unchecked(NOT_ALLOWED_CW20), msg.clone())
            .unwrap_err();
        assert_eq!(
            ContractError::CW20TokenNotAllowed {
                sent: NOT_ALLOWED_CW20.to_string(),
                need: ALLOWED_CW20.to_string()
            },
            res.downcast().unwrap()
        );

        //Buying a non-existent token
        let send_msg =
            Binary::from(r#"{"buy":{"recipient":"my_addr", "token_id":"token2"}}"#.as_bytes());
        let msg = marketplace_contract
            .call(
                ExecuteMsg::Receive {
                    0: Cw20ReceiveMsg {
                        sender: "my_addr".to_string(),
                        amount: Uint128::new(150),
                        msg: send_msg.clone(),
                    },
                },
                vec![],
            )
            .unwrap();
        let res = suite
            .app
            .execute(Addr::unchecked(ALLOWED_CW20), msg.clone())
            .unwrap_err();
        assert_eq!(ContractError::NotFound {}, res.downcast().unwrap());

        // happy path
        let raw_price = Uint128::new(100u128);
        let fee = Uint128::new(3u128);
        let owner_payout = Uint128::new(97u128);

        let send_msg = Binary::from(
            r#"{"buy":{"recipient":"new_owner_addr", "token_id":"token1"}}"#.as_bytes(),
        );
        let cw20_execute_msg_op = Cw20ExecuteMsg::Send {
            contract: marketplace_contract.addr().to_string(),
            amount: raw_price,
            msg: send_msg,
        };
        let msg = Cw20Contract(cw20_addr.clone())
            .call(cw20_execute_msg_op)
            .unwrap();
        let _response = suite
            .app
            .execute(Addr::unchecked(ALLOWED_CW20_OWNER), msg)
            .unwrap();

        // nft owner updated
        let res = nft_contract
            .owner_of(&suite.app.wrap(), TOKEN_ID1, false)
            .unwrap();
        assert_eq!(res.owner, String::from("new_owner_addr"));

        // collector balance updated
        let collector_balance: BalanceResponse = suite
            .app
            .wrap()
            .query_wasm_smart(
                cw20_addr.clone(),
                &cw20_base::msg::QueryMsg::Balance {
                    address: COLLECTOR.to_string(),
                },
            )
            .unwrap();
        assert_eq!(collector_balance.balance, fee);

        //owner balance updated
        let owner_balance: BalanceResponse = suite
            .app
            .wrap()
            .query_wasm_smart(
                cw20_addr.clone(),
                &cw20_base::msg::QueryMsg::Balance {
                    address: OWNER.to_string(),
                },
            )
            .unwrap();
        assert_eq!(
            owner_balance.balance.u128(),
            OWNER_INIT_BALANCE + owner_payout.u128()
        );
    }

    #[test]
    fn test_query_tokens_on_sale() {
        let mut suite = Suite::init().unwrap();
        let (_nft_contract, marketplace_contract, _cw20_addr) = suite.proper_instantiate_cw20();

        let token1 = Token {
            id: String::from(TOKEN_ID1),
            price: Default::default(),
            on_sale: true,
            rarity: "".to_string(),
        };

        let token2 = Token {
            id: String::from(TOKEN_ID2),
            price: Default::default(),
            on_sale: true,
            rarity: "".to_string(),
        };

        let token_false = Token {
            id: String::from(TOKEN_ID3),
            price: Default::default(),
            on_sale: false,
            rarity: "".to_string(),
        };
        let msg = marketplace_contract
            .call(
                ExecuteMsg::ListTokens {
                    tokens: vec![token1.clone(), token2.clone(), token_false],
                },
                vec![],
            )
            .unwrap();

        // register token
        suite.app.execute(Addr::unchecked(ADMIN), msg).unwrap();

        // query tokens on sale
        let query_msg = QueryMsg::ListTokensOnSale {
            start_after: None,
            limit: None,
        };
        let res: TokensResponse = suite
            .app
            .wrap()
            .query_wasm_smart(marketplace_contract.addr(), &query_msg)
            .unwrap();
        assert_eq!(res.tokens, vec![token1, token2])
    }

    #[test]
    fn test_config_instantiate() {
        let mut suite = Suite::init().unwrap();
        let nft = suite.instantiate_nft(String::from(MINTER));
        let mint_msg: cw721_base::msg::MintMsg<Metadata> = cw721_base::MintMsg {
            token_id: TOKEN_ID1.to_string(),
            owner: OWNER.to_string(),
            token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
            extension: Metadata {
                description: Some("Spaceship with Warp Drive".into()),
                name: Some("Starship USS Enterprise".to_string()),
                ..Metadata::default()
            },
        };
        let exec_msg = cw721_base::ExecuteMsg::Mint(mint_msg);
        let cosmos_msg = nft.call(exec_msg).unwrap();
        suite
            .app
            .execute(Addr::unchecked(MINTER), cosmos_msg)
            .unwrap();

        // test fee validation
        let wrong_fee = Decimal::percent(34);
        let err = suite
            .instantiate_marketplace(
                nft.addr().into(),
                Some(String::from(ALLOWED_NATIVE)),
                None,
                wrong_fee,
            )
            .unwrap_err();
        assert_eq!(ContractError::WrongInput {}, err.downcast().unwrap());

        // test fee validation
        let wrong_fee = Decimal::percent(34);
        let err = suite
            .instantiate_marketplace(
                nft.addr().into(),
                None,
                Some(String::from(ALLOWED_CW20)),
                wrong_fee,
            )
            .unwrap_err();
        assert_eq!(ContractError::WrongInput {}, err.downcast().unwrap());

        // happy path
        let correct_fee = Decimal::percent(14);
        suite
            .instantiate_marketplace(
                nft.addr().into(),
                Some(String::from(ALLOWED_NATIVE)),
                None,
                correct_fee,
            )
            .unwrap();

        suite
            .instantiate_marketplace(
                nft.addr().into(),
                None,
                Some(String::from(ALLOWED_CW20)),
                correct_fee,
            )
            .unwrap();
    }
}
