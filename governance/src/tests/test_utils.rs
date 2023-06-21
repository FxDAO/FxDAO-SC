#![cfg(test)]
use crate::contract::{GovernanceContract, GovernanceContractClient};
use crate::storage::proposals::{
    ProposalExecutionParams, TreasuryPaymentProposalOption, TreasuryPaymentProposalParams,
    UpdateContractProposalOption,
};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{token, vec, Address, Env, Vec};
use token::Client as TokenClient;

pub const TEST_PROPOSAL_FEE: u128 = 12_00_000_0000000;
pub const TEST_VOTING_CREDIT_PRICE: u128 = 1_0000000;

pub fn create_token_contract<'a>(e: &Env, admin: &Address) -> TokenClient<'a> {
    token::Client::new(&e, &e.register_stellar_asset_contract(admin.clone()))
}

pub struct TestData<'a> {
    pub governance_token_admin: Address,
    pub governance_token: TokenClient<'a>,
    pub contract_admin: Address,
    pub contract_client: GovernanceContractClient<'a>,
    pub cooldown_period: u64,
    pub dumb_params: ProposalExecutionParams,
}

pub fn create_test_data(env: &Env) -> TestData {
    let governance_token_admin = Address::random(&env);
    let governance_token = create_token_contract(&env, &governance_token_admin);

    let contract_admin: Address = Address::random(&env);
    let contract_client =
        GovernanceContractClient::new(&env, &env.register_contract(None, GovernanceContract));

    TestData {
        governance_token_admin,
        governance_token,
        contract_admin,
        contract_client,
        cooldown_period: 3600 * 24, // TODO: Implement this at the create proposal level IE proposers cooldown checks
        dumb_params: ProposalExecutionParams {
            treasury_payment: TreasuryPaymentProposalOption::None,
            update_contract: UpdateContractProposalOption::None,
        },
    }
}

pub fn init_contract(test_data: &TestData) {
    test_data.contract_client.init(
        &test_data.governance_token.address,
        &TEST_PROPOSAL_FEE,
        &TEST_VOTING_CREDIT_PRICE,
        &test_data.contract_admin,
        &test_data.cooldown_period,
    );
}
