use crate::errors::SCErrors;
use crate::storage::core::CoreState;
use crate::storage::proposals::{
    Proposal, ProposalStatus, ProposalType, ProposalVote, ProposalVoteType, ProposerStat,
};
use crate::utils::core::{can_init_contract, get_core_state, set_core_state};
use crate::utils::proposals::{
    authenticate_proposers, calculate_proposal_vote_price, charge_proposal_vote, charge_proposers,
    get_proposal, get_proposal_votes, get_proposals_fee, get_proposals_ids, is_proposal_active,
    is_voting_time_valid, new_proposal, save_new_proposal_id, save_proposal, save_proposal_votes,
    validate_can_vote, validate_new_proposal_id, validate_proposers_payment,
};
use soroban_sdk::{contractimpl, panic_with_error, vec, Address, BytesN, Env, Vec};

pub trait GovernanceContractTrait {
    fn init(
        env: Env,
        governance_token: Address,
        proposals_fee: u128,
        voting_credit_price: u128,
        contract_admin: Address,
    );

    fn create_proposal(
        env: Env,
        id: BytesN<32>,
        proposal_type: ProposalType,
        proposers: Vec<ProposerStat>,
        voting_time: u64,
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
}

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
    ) {
        can_init_contract(&env);
        set_core_state(
            &env,
            &CoreState {
                governance_token,
                proposals_fee,
                voting_credit_price,
                contract_admin,
            },
        );
    }

    fn create_proposal(
        env: Env,
        id: BytesN<32>,
        proposal_type: ProposalType,
        proposers: Vec<ProposerStat>,
        voting_time: u64,
    ) {
        validate_new_proposal_id(&env, &id);
        for item in proposers.iter() {
            let proposer: ProposerStat = item.unwrap();
            proposer.id.require_auth();
        }

        // TODO: Test this
        if !is_voting_time_valid(&env, voting_time.clone(), &proposal_type, &proposers) {
            panic_with_error!(&env, SCErrors::InvalidVotingTime);
        }

        let proposal_fee: u128 = get_proposals_fee(&env);
        if !validate_proposers_payment(&proposal_fee, &proposers) {
            panic_with_error!(&env, SCErrors::InvalidProposalFee);
        }

        let new_proposal: Proposal = new_proposal(
            &id,
            &proposers,
            &proposal_type,
            env.ledger().timestamp(),
            env.ledger().timestamp() + (3600 * 24 * 14),
        );

        charge_proposers(&env, &proposers);
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
        let mut proposal_votes: Vec<ProposalVote> = get_proposal_votes(&env, &proposal_id);

        // TODO: test this
        if !is_proposal_active(&env, &proposal) {
            panic_with_error!(&env, SCErrors::ProposalIsNotActive);
        }

        if !validate_can_vote(&voter, &proposal, &proposal_votes) {
            panic_with_error!(&env, SCErrors::CanNotVote);
        }

        let core_state: CoreState = get_core_state(&env);

        let vote_price = calculate_proposal_vote_price(
            &amount,
            &core_state.voting_credit_price,
            &proposal.proposal_type,
        );

        charge_proposal_vote(&env, &voter, &vote_price);

        proposal_votes.push_front(ProposalVote {
            voter,
            vote_type: vote_type.clone(),
            proposal_id: proposal_id.clone(),
            amount,
        });

        save_proposal_votes(&env, &proposal_id, &proposal_votes);

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
}
