use crate::errors::SCErrors;
use crate::storage::core::CoreState;
use crate::storage::proposals::{
    Proposal, ProposalExecutionParams, ProposalStatus, ProposalType, ProposalVoteIndex,
    ProposalVoteType, ProposerStat, TreasuryPaymentProposalOption, UpdateContractProposalOption,
    UpgradeContractProposalOption,
};
use crate::utils::core::{
    can_init_contract, get_allowed_contracts_functions, get_core_state, get_managing_contracts,
    save_allowed_contracts_functions, save_managing_contracts, set_core_state,
};
use crate::utils::proposals::{
    are_update_contract_params_valid, calculate_proposal_vote_price, charge_proposal_vote,
    charge_proposers, get_proposal, get_proposal_votes, get_proposals_fee, get_proposals_ids,
    is_contract_to_upgrade_valid, is_proposal_active, is_voting_time_valid, make_treasury_payment,
    new_proposal, proposal_can_be_ended, proposal_cooldown_completed, save_new_proposal_id,
    save_proposal, save_proposal_votes, validate_can_vote, validate_new_proposal_id,
    validate_proposers_payment,
};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, vec, Address, BytesN, Env, Map, Symbol,
    Val, Vec,
};

pub const CONTRACT_DESCRIPTION: Symbol = symbol_short!("Gov");
pub const CONTRACT_VERSION: Symbol = symbol_short!("0_3_0");

pub trait GovernanceContractTrait {
    fn init(
        env: Env,
        governance_token: Address,
        proposals_fee: u128,
        voting_credit_price: u128,
        contract_admin: Address,
        cooldown_period: u64,
        managing_contracts: Vec<Address>,
        allowed_contracts_functions: Map<Address, Vec<Symbol>>,
    );

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>);

    fn version(env: Env) -> (Symbol, Symbol);

    fn create_proposal(
        env: Env,
        id: BytesN<32>,
        proposal_type: ProposalType,
        proposers: Vec<ProposerStat>,
        voting_time: u64,
        execution_params: ProposalExecutionParams,
    );

    fn get_proposal(env: Env, proposal_id: BytesN<32>) -> Proposal;

    fn get_proposals_ids(env: Env) -> Vec<BytesN<32>>;

    fn vote(
        env: Env,
        voter: Address,
        proposal_id: BytesN<32>,
        vote_type: ProposalVoteType,
        amount: u128,
    );

    fn end_proposal(env: Env, proposal_id: BytesN<32>);

    fn execute_proposal_result(env: Env, proposal_id: BytesN<32>);
}

#[contract]
pub struct GovernanceContract;

// TODO: Add events for each function
#[contractimpl]
impl GovernanceContractTrait for GovernanceContract {
    fn init(
        env: Env,
        governance_token: Address,
        proposals_fee: u128,
        voting_credit_price: u128,
        contract_admin: Address,
        cooldown_period: u64,
        managing_contracts: Vec<Address>,
        allowed_contracts_functions: Map<Address, Vec<Symbol>>,
    ) {
        can_init_contract(&env);
        set_core_state(
            &env,
            &CoreState {
                governance_token,
                proposals_fee,
                voting_credit_price,
                contract_admin,
                cooldown_period,
            },
        );
        save_managing_contracts(&env, &managing_contracts);
        save_allowed_contracts_functions(&env, &allowed_contracts_functions);
    }

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let core_state: CoreState = get_core_state(&env);
        core_state.contract_admin.require_auth();
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn version(env: Env) -> (Symbol, Symbol) {
        (CONTRACT_DESCRIPTION, CONTRACT_VERSION)
    }

    fn create_proposal(
        env: Env,
        id: BytesN<32>,
        proposal_type: ProposalType,
        proposers: Vec<ProposerStat>,
        voting_time: u64,
        execution_params: ProposalExecutionParams,
    ) {
        validate_new_proposal_id(&env, &id);

        // This value is set as true if one of the proposers is the admin and the proposal type is to upgrade a contract
        let mut admin_upgrade = false;
        let core_state = get_core_state(&env);

        for proposer_stat in proposers.iter() {
            proposer_stat.id.require_auth();
            if proposal_type == ProposalType::UpgradeContract
                && core_state.contract_admin == proposer_stat.id
            {
                admin_upgrade = true;
            }
        }

        // TODO: Test this
        if !is_voting_time_valid(voting_time.clone(), &proposal_type, admin_upgrade) {
            panic_with_error!(&env, SCErrors::InvalidVotingTime);
        }

        let proposal_fee: u128 = get_proposals_fee(&env);
        if !validate_proposers_payment(&proposal_fee, &proposers) {
            panic_with_error!(&env, SCErrors::InvalidProposalFee);
        }

        // If the proposal type is UpdateContract, confirm that the target contract and function name is valid
        // This is with the goal of avoiding this contract calling a malicious contract
        if proposal_type == ProposalType::UpdateContract {
            let managing_contracts = get_managing_contracts(&env);
            let allowed_contracts_functions = get_allowed_contracts_functions(&env);
            if !are_update_contract_params_valid(
                &managing_contracts,
                &allowed_contracts_functions,
                &execution_params,
            ) {
                panic_with_error!(&env, SCErrors::InvalidExecutionParams);
            }
        }

        // If the proposal is type UpgradeContract, confirm that the target contract is a valid id
        if proposal_type == ProposalType::UpgradeContract {
            let managing_contracts = get_managing_contracts(&env);
            if !is_contract_to_upgrade_valid(&managing_contracts, &execution_params) {
                panic_with_error!(&env, SCErrors::InvalidExecutionParams);
            }
        }

        // TODO: Prevent wrong params
        // TODO: Create a test for such cases
        // TODO: The check needs to make sure the type of the proposal goes ok with the params
        // TODO: Even check the amount of parameters are valid and if possible its type

        let new_proposal: Proposal = new_proposal(
            &id,
            &proposers,
            &proposal_type,
            env.ledger().timestamp(),
            voting_time,
            admin_upgrade,
            execution_params,
        );

        if !admin_upgrade {
            charge_proposers(&env, &proposers);
        }
        save_proposal(&env, &new_proposal);
        save_new_proposal_id(&env, &id);
    }

    fn get_proposal(env: Env, proposal_id: BytesN<32>) -> Proposal {
        get_proposal(&env, &proposal_id)
    }

    fn get_proposals_ids(env: Env) -> Vec<BytesN<32>> {
        get_proposals_ids(&env)
    }

    fn vote(
        env: Env,
        voter: Address,
        proposal_id: BytesN<32>,
        vote_type: ProposalVoteType,
        amount: u128,
    ) {
        voter.require_auth();

        let mut proposal: Proposal = get_proposal(&env, &proposal_id);
        let mut proposal_votes_indexes: Vec<ProposalVoteIndex> =
            get_proposal_votes(&env, &proposal_id);

        // TODO: test this
        if !is_proposal_active(&env, &proposal) {
            panic_with_error!(&env, SCErrors::ProposalIsNotActive);
        }

        if !validate_can_vote(&env, &voter, &proposal) {
            panic_with_error!(&env, SCErrors::CanNotVote);
        }

        let core_state: CoreState = get_core_state(&env);

        let vote_price = calculate_proposal_vote_price(
            &amount,
            &core_state.voting_credit_price,
            &proposal.proposal_type,
        );

        charge_proposal_vote(&env, &voter, &vote_price);

        proposal_votes_indexes.push_front(ProposalVoteIndex {
            voter_id: voter.clone(),
            proposal_id: proposal_id.clone(),
        });

        save_proposal_votes(&env, &proposal_id, &proposal_votes_indexes);

        if vote_type == ProposalVoteType::For {
            proposal.credits_for += vote_price;
            proposal.votes_for += amount;
            proposal.voters_for += 1;
        } else {
            proposal.credits_against += vote_price;
            proposal.votes_against += amount;
            proposal.voters_against += 1;
        }

        save_proposal(&env, &proposal);
    }

    fn end_proposal(env: Env, proposal_id: BytesN<32>) {
        let mut proposal: Proposal = get_proposal(&env, &proposal_id);

        if proposal.status != ProposalStatus::Active {
            panic_with_error!(&env, &SCErrors::ProposalIsNotActive);
        }

        if !proposal_can_be_ended(&env, &proposal) {
            panic_with_error!(&env, SCErrors::ProposalPeriodHasNotEnded);
        }

        if proposal.votes_for > proposal.votes_against {
            proposal.status = ProposalStatus::Accepted;
        } else {
            proposal.status = ProposalStatus::Cancelled;
        }

        save_proposal(&env, &proposal);
    }

    fn execute_proposal_result(env: Env, proposal_id: BytesN<32>) {
        let core_state: CoreState = get_core_state(&env);
        let mut proposal: Proposal = get_proposal(&env, &proposal_id);

        // TODO: test this
        if proposal.executed {
            panic_with_error!(&env, SCErrors::ProposalAlreadyExecuted);
        }

        if !proposal_cooldown_completed(&env, &proposal, &core_state) {
            panic_with_error!(&env, SCErrors::ExecutionCooldownUncompleted);
        }

        match proposal.proposal_type {
            ProposalType::Simple => {
                // We don't do anything in this type because the Simple proposal type doesn't include any execution logic
            }
            ProposalType::UpgradeContract => match &proposal.execution_params.upgrade_contract {
                UpgradeContractProposalOption::None => {
                    panic_with_error!(&env, SCErrors::ExecutionParamsAreInvalid);
                }
                UpgradeContractProposalOption::Some(data) => {
                    env.invoke_contract::<Val>(
                        &data.contract_id.clone(),
                        &symbol_short!("upgrade"),
                        vec![&env, data.new_contract_hash.to_val()] as Vec<Val>,
                    );
                }
            },
            ProposalType::UpdateContract => match &proposal.execution_params.update_contract {
                UpdateContractProposalOption::None => {
                    panic_with_error!(&env, SCErrors::ExecutionParamsAreInvalid);
                }
                UpdateContractProposalOption::Some(data) => {
                    env.invoke_contract::<Val>(
                        &data.contract_id,
                        &data.function_name,
                        data.params.clone(),
                    );
                }
            },
            ProposalType::TreasuryPayment => match &proposal.execution_params.treasury_payment {
                TreasuryPaymentProposalOption::None => {
                    panic_with_error!(&env, SCErrors::ExecutionParamsAreInvalid);
                }
                TreasuryPaymentProposalOption::Some(params) => {
                    make_treasury_payment(&env, &core_state, &params.recipient, &params.amount);
                }
            },
            ProposalType::Structural => {
                // TODO:
                panic_with_error!(&env, SCErrors::UnsupportedProposalType);
            }
        }

        proposal.executed = true;
        save_proposal(&env, &proposal);
    }
}
