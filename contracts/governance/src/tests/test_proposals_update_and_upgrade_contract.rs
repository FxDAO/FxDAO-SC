#![cfg(test)]
extern crate std;

use crate::contract::{GovernanceContract, GovernanceContractClient};
use crate::errors::SCErrors;
use crate::storage::proposals::{
    Proposal, ProposalExecutionParams, ProposalType, ProposalVoteType, ProposerStat,
    TreasuryPaymentProposalOption, UpdateContractProposalOption, UpdateContractProposalParams,
    UpgradeContractProposalOption, UpgradeContractProposalParams,
};
use crate::tests::test_utils::create_token_contract;
use soroban_sdk::testutils::{Address as _, BytesN as __, Ledger, LedgerInfo};
use soroban_sdk::{
    map, symbol_short, token, vec, Address, BytesN, Env, IntoVal, Map, Symbol, Val, Vec,
};
use token::Client as TokenClient;
use token::StellarAssetClient as TokenAdminClient;

mod vaults {
    soroban_sdk::contractimport!(file = "../../target/wasm32-unknown-unknown/release/vaults.wasm");
}

mod safety_pool {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/safety_pool.wasm"
    );
}

struct TestData<'a> {
    pub governance_contract_admin: Address,

    pub collateral_token_admin: Address,
    pub collateral_token_client: TokenClient<'a>,
    pub collateral_token_admin_client: TokenAdminClient<'a>,

    pub governance_token_admin: Address,
    pub governance_token_client: TokenClient<'a>,
    pub governance_token_admin_client: TokenAdminClient<'a>,

    pub stablecoin_issuer_admin: Address,
    pub usd_stable_token_client: TokenClient<'a>,
    pub usd_stable_token_admin_client: TokenAdminClient<'a>,
    pub eur_stable_token_client: TokenClient<'a>,
    pub eur_stable_token_admin_client: TokenAdminClient<'a>,
    pub min_deposit_usd_safety_pool: u128,

    pub vaults_contract_address: Address,
    pub vaults_contract_client: vaults::Client<'a>,

    pub usd_safety_pool_contract_address: Address,
    pub usd_safety_pool_contract_client: safety_pool::Client<'a>,

    pub governance_contract_address: Address,
    pub governance_contract_client: GovernanceContractClient<'a>,
    pub governance_proposals_fee: u128,
    pub governance_voting_credit_price: u128,
    pub governance_cooldown_period: u64,
    pub governance_managing_contracts: Vec<Address>,
    pub governance_allowed_contracts_functions: Map<Address, Vec<Symbol>>,

    pub treasury_contract_address: Address,
}

fn create_test_data(env: &Env) -> TestData {
    let governance_contract_admin: Address = Address::generate(&env);

    let collateral_token_admin: Address = Address::generate(&env);
    let (collateral_token_client, collateral_token_admin_client) =
        create_token_contract(&env, &collateral_token_admin);

    let governance_token_admin: Address = Address::generate(&env);
    let (governance_token_client, governance_token_admin_client) =
        create_token_contract(&env, &governance_token_admin);

    let stablecoin_issuer_admin: Address = Address::generate(&env);
    let (usd_stable_token_client, usd_stable_token_admin_client) =
        create_token_contract(&env, &stablecoin_issuer_admin);
    let (eur_stable_token_client, eur_stable_token_admin_client) =
        create_token_contract(&env, &stablecoin_issuer_admin);

    let vaults_contract_address: Address = env.register_contract_wasm(None, vaults::WASM);
    let vaults_contract_client = vaults::Client::new(&env, &vaults_contract_address);

    let usd_safety_pool_contract_address: Address =
        env.register_contract_wasm(None, safety_pool::WASM);
    let usd_safety_pool_contract_client =
        safety_pool::Client::new(&env, &usd_safety_pool_contract_address);
    let min_deposit_usd_safety_pool: u128 = 100_0000000;

    let governance_contract_address: Address = env.register_contract(None, GovernanceContract);
    let governance_contract_client =
        GovernanceContractClient::new(&env, &governance_contract_address);
    let governance_proposals_fee: u128 = 6_000_000_0000000;
    let governance_voting_credit_price: u128 = 1_0000000;
    let governance_cooldown_period: u64 = 3600 * 24;
    let governance_managing_contracts: Vec<Address> = vec![
        &env,
        usd_safety_pool_contract_client.address.clone(),
        vaults_contract_client.address.clone(),
    ] as Vec<Address>;
    let governance_allowed_contracts_functions: Map<Address, Vec<Symbol>> = map![
        &env,
        (
            vaults_contract_client.address.clone(),
            vec![
                &env,
                Symbol::new(&env, "upgrade"),
                Symbol::new(&env, "set_vault_conditions"),
                Symbol::new(&env, "create_currency"),
                Symbol::new(&env, "set_currency_rate"),
                Symbol::new(&env, "toggle_currency"),
            ]
        ),
        (
            usd_safety_pool_contract_client.address.clone(),
            vec![
                &env,
                Symbol::new(&env, "get_core_state"),
                Symbol::new(&env, "update_contract_admin"),
                Symbol::new(&env, "update_vaults_contract"),
                Symbol::new(&env, "update_treasury_contract"),
                Symbol::new(&env, "update_min_deposit"),
                Symbol::new(&env, "update_treasury_share"),
                Symbol::new(&env, "update_liquidator_share"),
            ]
        ),
    ]
        as Map<Address, Vec<Symbol>>;

    let treasury_contract_address: Address = Address::generate(&env);

    TestData {
        governance_contract_admin,

        collateral_token_admin,
        collateral_token_client,
        collateral_token_admin_client,

        governance_token_admin,
        governance_token_client,
        governance_token_admin_client,

        stablecoin_issuer_admin,
        usd_stable_token_client,
        usd_stable_token_admin_client,
        eur_stable_token_client,
        eur_stable_token_admin_client,

        vaults_contract_address,
        vaults_contract_client,

        usd_safety_pool_contract_address,
        usd_safety_pool_contract_client,
        min_deposit_usd_safety_pool,

        governance_contract_address,
        governance_contract_client,
        governance_proposals_fee,
        governance_voting_credit_price,
        governance_cooldown_period,
        governance_managing_contracts,
        governance_allowed_contracts_functions,

        treasury_contract_address,
    }
}

fn setup_contracts(env: &Env, test_data: &TestData) {
    test_data.vaults_contract_client.init(
        &test_data.governance_contract_address,
        &test_data.governance_contract_address,
        &test_data.governance_contract_address,
        &test_data.collateral_token_client.address,
        &test_data.stablecoin_issuer_admin,
    );

    test_data.usd_safety_pool_contract_client.init(
        &test_data.governance_contract_address,
        &test_data.vaults_contract_address,
        &test_data.treasury_contract_address,
        &test_data.collateral_token_client.address,
        &test_data.usd_stable_token_client.address,
        &symbol_short!("usd"),
        &test_data.min_deposit_usd_safety_pool,
        &(vec![&env, 1u32, 2u32] as Vec<u32>),
        &(vec![&env, 1u32, 2u32] as Vec<u32>),
        &test_data.governance_token_client.address,
    );

    test_data.governance_contract_client.init(
        &test_data.governance_token_client.address,
        &test_data.governance_proposals_fee,
        &test_data.governance_voting_credit_price,
        &test_data.governance_contract_admin,
        &test_data.governance_cooldown_period,
        &test_data.governance_managing_contracts,
        &test_data.governance_allowed_contracts_functions,
    );
}

#[test]
pub fn test_setup_contracts() {
    let env: Env = Env::default();
    env.budget().reset_unlimited(); // We reset the budget
    let test_data: TestData = create_test_data(&env);
    setup_contracts(&env, &test_data);

    let vaults_core_state = test_data.vaults_contract_client.get_core_state();
    let vaults_contract_admin = vaults_core_state.admin.clone();
    assert_eq!(
        &vaults_core_state.col_token,
        &test_data.collateral_token_client.address,
    );
    assert_eq!(
        &vaults_core_state.stable_issuer,
        &test_data.stablecoin_issuer_admin,
    );
    assert_eq!(
        &vaults_contract_admin,
        &test_data.governance_contract_client.address,
    );

    let usd_safety_pool_core_state = test_data.usd_safety_pool_contract_client.get_core_state();
    assert_eq!(
        &usd_safety_pool_core_state.admin,
        &test_data.governance_contract_client.address,
    );
    assert_eq!(
        &usd_safety_pool_core_state.vaults_contract,
        &test_data.vaults_contract_client.address,
    );
    assert_eq!(
        &usd_safety_pool_core_state.treasury_contract,
        &test_data.treasury_contract_address,
    );
    assert_eq!(
        &usd_safety_pool_core_state.collateral_asset,
        &test_data.collateral_token_client.address,
    );
    assert_eq!(
        &usd_safety_pool_core_state.deposit_asset,
        &test_data.usd_stable_token_client.address,
    );
    assert_eq!(
        &usd_safety_pool_core_state.denomination_asset,
        &symbol_short!("usd"),
    );
    assert_eq!(
        &usd_safety_pool_core_state.min_deposit,
        &test_data.min_deposit_usd_safety_pool,
    );
    assert_eq!(
        &usd_safety_pool_core_state.treasury_share,
        &vec![&env, 1u32, 2u32],
    );
    assert_eq!(
        &usd_safety_pool_core_state.liquidator_share,
        &vec![&env, 1u32, 2u32],
    );
}

#[test]
pub fn test_create_update_proposal_wrong_params() {
    let env: Env = Env::default();
    env.mock_all_auths();
    let test_data: TestData = create_test_data(&env);
    setup_contracts(&env, &test_data);

    let proposer = Address::generate(&env);
    let voting_time: u64 = 3600 * 24 * 7;

    let no_options_error = test_data
        .governance_contract_client
        .try_create_proposal(
            &BytesN::generate(&env),
            &ProposalType::UpdateContract,
            &(vec![
                &env,
                ProposerStat {
                    amount: test_data.governance_proposals_fee,
                    id: proposer.clone(),
                },
            ] as Vec<ProposerStat>),
            &voting_time,
            &ProposalExecutionParams {
                upgrade_contract: UpgradeContractProposalOption::None,
                treasury_payment: TreasuryPaymentProposalOption::None,
                update_contract: UpdateContractProposalOption::None,
            },
        )
        .unwrap_err();

    // TODO: UPDATE THIS AFTER SOROBAN FIX IT
    // assert_eq!(&no_options_error.unwrap(), SCErrors::InvalidExecutionParams.into());

    let wrong_contract_error = test_data
        .governance_contract_client
        .try_create_proposal(
            &BytesN::generate(&env),
            &ProposalType::UpdateContract,
            &(vec![
                &env,
                ProposerStat {
                    amount: test_data.governance_proposals_fee,
                    id: proposer.clone(),
                },
            ] as Vec<ProposerStat>),
            &voting_time,
            &ProposalExecutionParams {
                upgrade_contract: UpgradeContractProposalOption::None,
                treasury_payment: TreasuryPaymentProposalOption::None,
                update_contract: UpdateContractProposalOption::Some(UpdateContractProposalParams {
                    params: vec![&env] as Vec<Val>,
                    function_name: Symbol::new(&env, "set_vault_conditions"),
                    contract_id: Address::generate(&env),
                }),
            },
        )
        .unwrap_err();

    // TODO: UPDATE THIS AFTER SOROBAN FIX IT
    // assert_eq!(
    //     &wrong_contract_error.unwrap(),
    //     &SCErrors::InvalidExecutionParams.into()
    // );

    let wrong_function_name_error = test_data
        .governance_contract_client
        .try_create_proposal(
            &BytesN::generate(&env),
            &ProposalType::UpdateContract,
            &(vec![
                &env,
                ProposerStat {
                    amount: test_data.governance_proposals_fee,
                    id: proposer.clone(),
                },
            ] as Vec<ProposerStat>),
            &voting_time,
            &ProposalExecutionParams {
                upgrade_contract: UpgradeContractProposalOption::None,
                treasury_payment: TreasuryPaymentProposalOption::None,
                update_contract: UpdateContractProposalOption::Some(UpdateContractProposalParams {
                    params: vec![&env] as Vec<Val>,
                    function_name: Symbol::new(&env, "set_vault_conditions_wrong"),
                    contract_id: test_data.vaults_contract_client.address.clone(),
                }),
            },
        )
        .unwrap_err();

    // TODO: UPDATE THIS AFTER SOROBAN FIX IT
    // assert_eq!(
    //     &wrong_function_name_error.unwrap(),
    //     &SCErrors::InvalidExecutionParams.into()
    // );
}

#[test]
pub fn test_update_proposal_flow() {
    let env: Env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let test_data: TestData = create_test_data(&env);
    setup_contracts(&env, &test_data);

    let proposer = Address::generate(&env);

    let proposers: Vec<ProposerStat> = vec![
        &env,
        ProposerStat {
            id: proposer.clone(),
            amount: test_data.governance_proposals_fee,
        },
    ] as Vec<ProposerStat>;
    let voting_time: u64 = 3600 * 24 * 7;

    let voter: Address = Address::generate(&env);

    test_data
        .governance_token_admin_client
        .mint(&proposer, &(test_data.governance_proposals_fee as i128));

    test_data
        .governance_token_admin_client
        .mint(&voter, &(test_data.governance_proposals_fee as i128));

    let proposal_id: BytesN<32> = BytesN::generate(&env);
    let usd_pool_core_state = test_data.usd_safety_pool_contract_client.get_core_state();

    assert_eq!(
        &usd_pool_core_state.min_deposit,
        &test_data.min_deposit_usd_safety_pool,
    );

    test_data.governance_contract_client.create_proposal(
        &proposal_id,
        &ProposalType::UpdateContract,
        &proposers,
        &voting_time,
        &ProposalExecutionParams {
            upgrade_contract: UpgradeContractProposalOption::None,
            treasury_payment: TreasuryPaymentProposalOption::None,
            update_contract: UpdateContractProposalOption::Some(UpdateContractProposalParams {
                contract_id: test_data.usd_safety_pool_contract_address.clone(),
                function_name: Symbol::new(&env, "update_min_deposit"),
                params: vec![
                    &env,
                    (test_data.min_deposit_usd_safety_pool - 25_0000000u128).into_val(&env),
                ] as Vec<Val>,
            }),
        },
    );

    // Confirm the proposal was saved, we don't test everything as the rest is tested on another place
    let proposal: Proposal = test_data
        .governance_contract_client
        .get_proposal(&proposal_id);
    assert_eq!(&proposal.id, &proposal_id);
    assert_eq!(&proposal.proposal_type, &ProposalType::UpdateContract);

    env.budget().reset_unlimited(); // We reset the budget

    test_data
        .governance_contract_client
        .vote(&voter, &proposal_id, &ProposalVoteType::For, &1);

    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + voting_time + 1,
        protocol_version: 1,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

    test_data
        .governance_contract_client
        .end_proposal(&proposal_id);

    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + test_data.governance_cooldown_period,
        protocol_version: 1,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

    test_data
        .governance_contract_client
        .execute_proposal_result(&proposal_id);

    let updated_usd_pool_core_state = test_data.usd_safety_pool_contract_client.get_core_state();

    assert_eq!(
        &updated_usd_pool_core_state.min_deposit,
        &(test_data.min_deposit_usd_safety_pool - 25_0000000u128),
    );
}

#[test]
pub fn test_create_upgrade_proposal_flow_wrong_params() {
    let env: Env = Env::default();
    env.mock_all_auths();
    let test_data: TestData = create_test_data(&env);
    setup_contracts(&env, &test_data);

    let proposer = Address::generate(&env);

    test_data
        .governance_token_admin_client
        .mint(&proposer, &(test_data.governance_proposals_fee as i128));

    // Should fail because it's using a voting time only for admins
    let invalid_time_error = test_data
        .governance_contract_client
        .try_create_proposal(
            &BytesN::generate(&env),
            &ProposalType::UpgradeContract,
            &(vec![
                &env,
                ProposerStat {
                    amount: test_data.governance_proposals_fee,
                    id: proposer.clone(),
                },
            ] as Vec<ProposerStat>),
            &(3600 * 5),
            &ProposalExecutionParams {
                upgrade_contract: UpgradeContractProposalOption::None,
                treasury_payment: TreasuryPaymentProposalOption::None,
                update_contract: UpdateContractProposalOption::None,
            },
        )
        .unwrap_err();

    // TODO: UPDATE THIS ONCE SOROBAN FIX IT
    // assert_eq!(&invalid_time_error.unwrap(), SCErrors::InvalidVotingTime.into());

    // Should fail because even admins have a min voting time
    let invalid_admin_time_error = test_data
        .governance_contract_client
        .try_create_proposal(
            &BytesN::generate(&env),
            &ProposalType::UpgradeContract,
            &(vec![
                &env,
                ProposerStat {
                    amount: test_data.governance_proposals_fee,
                    id: test_data.governance_contract_admin.clone(),
                },
            ] as Vec<ProposerStat>),
            &3500,
            &ProposalExecutionParams {
                upgrade_contract: UpgradeContractProposalOption::None,
                treasury_payment: TreasuryPaymentProposalOption::None,
                update_contract: UpdateContractProposalOption::None,
            },
        )
        .unwrap_err();

    // TODO: UPDATE THIS ONCE SOROBAN FIX IT
    // assert_eq!(
    //     &invalid_admin_time_error.unwrap(),
    //     &SCErrors::InvalidVotingTime.into(),
    // );

    // Should fail because the target contract to upgrade is not one we are admins
    let invalid_target_contract = test_data
        .governance_contract_client
        .try_create_proposal(
            &BytesN::generate(&env),
            &ProposalType::UpgradeContract,
            &(vec![
                &env,
                ProposerStat {
                    amount: test_data.governance_proposals_fee,
                    id: test_data.governance_contract_admin.clone(),
                },
            ] as Vec<ProposerStat>),
            &(3600 * 24 * 15),
            &ProposalExecutionParams {
                upgrade_contract: UpgradeContractProposalOption::Some(
                    UpgradeContractProposalParams {
                        contract_id: Address::generate(&env),
                        new_contract_hash: BytesN::generate(&env),
                    },
                ),
                treasury_payment: TreasuryPaymentProposalOption::None,
                update_contract: UpdateContractProposalOption::None,
            },
        )
        .unwrap_err();

    // TODO: UPDATE THIS ONCE SOROBAN FIX IT
    // assert_eq!(
    //     &invalid_target_contract.unwrap(),
    //     &SCErrors::InvalidExecutionParams.into(),
    // );
}

#[test]
pub fn test_upgrade_proposal_flow() {
    let env: Env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited(); // We reset the budget
    let test_data = create_test_data(&env);
    setup_contracts(&env, &test_data);

    let proposer: Address = Address::generate(&env);
    let voter: Address = Address::generate(&env);

    test_data
        .governance_token_admin_client
        .mint(&proposer, &(test_data.governance_proposals_fee as i128));

    test_data
        .governance_token_admin_client
        .mint(&voter, &(test_data.governance_proposals_fee as i128));

    // We create a new governance contract instance, we will update the safety pool with this one
    let new_wasm = env.deployer().upload_contract_wasm(safety_pool::WASM);

    let proposal_id: BytesN<32> = BytesN::generate(&env);
    test_data.governance_contract_client.create_proposal(
        &proposal_id,
        &ProposalType::UpgradeContract,
        &(vec![
            &env,
            ProposerStat {
                amount: test_data.governance_proposals_fee.clone(),
                id: proposer.clone(),
            },
        ] as Vec<ProposerStat>),
        &(3600 * 24 * 15),
        &ProposalExecutionParams {
            treasury_payment: TreasuryPaymentProposalOption::None,
            update_contract: UpdateContractProposalOption::None,
            upgrade_contract: UpgradeContractProposalOption::Some(UpgradeContractProposalParams {
                contract_id: test_data.vaults_contract_client.address.clone(),
                new_contract_hash: new_wasm.clone(),
            }),
        },
    );

    let (description, _) = test_data.vaults_contract_client.version();
    assert_eq!(description, symbol_short!("Vaults"));

    test_data
        .governance_contract_client
        .vote(&voter, &proposal_id, &ProposalVoteType::For, &1);

    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + (3600 * 24 * 16),
        protocol_version: 1,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

    test_data
        .governance_contract_client
        .end_proposal(&proposal_id);

    env.ledger().set(LedgerInfo {
        timestamp: env.ledger().timestamp() + test_data.governance_cooldown_period + 1,
        protocol_version: 1,
        sequence_number: env.ledger().sequence(),
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 1,
        min_persistent_entry_ttl: 1,
        max_entry_ttl: u32::MAX,
    });

    test_data
        .governance_contract_client
        .execute_proposal_result(&proposal_id);

    let (description, _) = test_data.vaults_contract_client.version();
    assert_eq!(description, symbol_short!("SafetyP"));
}
