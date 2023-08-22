#![cfg(test)]
use crate::contract::{GovernanceContract, GovernanceContractClient};
use crate::storage::proposals::{
    ProposalExecutionParams, TreasuryPaymentProposalOption, UpdateContractProposalOption,
    UpgradeContractProposalOption,
};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{map, token, vec, Address, Env, Map, Symbol, Vec};
use token::AdminClient as TokenAdminClient;
use token::Client as TokenClient;

pub const TEST_PROPOSAL_FEE: u128 = 12_00_000_0000000;
pub const TEST_VOTING_CREDIT_PRICE: u128 = 1_0000000;

pub fn create_token_contract<'a>(
    e: &Env,
    admin: &Address,
) -> (TokenClient<'a>, TokenAdminClient<'a>) {
    let contract_address = e.register_stellar_asset_contract(admin.clone());
    (
        TokenClient::new(e, &contract_address),
        TokenAdminClient::new(e, &contract_address),
    )
}

pub struct TestData<'a> {
    pub governance_token_admin: Address,
    pub governance_token_client: TokenClient<'a>,
    pub governance_token_admin_client: TokenAdminClient<'a>,
    pub contract_admin: Address,
    pub contract_client: GovernanceContractClient<'a>,
    pub cooldown_period: u64,
    pub dumb_params: ProposalExecutionParams,
    pub managing_contracts: Vec<Address>,
    pub allowed_contracts_functions: Map<Address, Vec<Symbol>>,
}

pub fn create_test_data(env: &Env) -> TestData {
    let governance_token_admin = Address::random(&env);
    let (governance_token_client, governance_token_admin_client) =
        create_token_contract(&env, &governance_token_admin);

    let contract_admin: Address = Address::random(&env);
    let contract_client =
        GovernanceContractClient::new(&env, &env.register_contract(None, GovernanceContract));

    TestData {
        governance_token_admin,
        governance_token_client,
        governance_token_admin_client,
        contract_admin,
        contract_client,
        cooldown_period: 3600 * 24, // TODO: Implement this at the create proposal level IE proposers cooldown checks
        dumb_params: ProposalExecutionParams {
            upgrade_contract: UpgradeContractProposalOption::None,
            treasury_payment: TreasuryPaymentProposalOption::None,
            update_contract: UpdateContractProposalOption::None,
        },
        managing_contracts: vec![&env] as Vec<Address>,
        allowed_contracts_functions: map![&env],
    }
}

pub fn init_contract(test_data: &TestData) {
    test_data.contract_client.init(
        &test_data.governance_token_client.address,
        &TEST_PROPOSAL_FEE,
        &TEST_VOTING_CREDIT_PRICE,
        &test_data.contract_admin,
        &test_data.cooldown_period,
        &test_data.managing_contracts,
        &test_data.allowed_contracts_functions,
    );
}
