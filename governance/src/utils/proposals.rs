use crate::errors::SCErrors;
use crate::storage::core::{CoreState, CoreStorageKeys};
use crate::storage::proposals::{
    Proposal, ProposalStatus, ProposalType, ProposalVote, ProposalsStorageKeys, ProposerStat,
};

use crate::utils::core::{get_core_state, get_governance_token};
use soroban_sdk::{panic_with_error, vec, Address, BytesN, Env, IntoVal, Vec};

// PROPOSERS FUNCTIONS

pub fn authenticate_proposers(proposers_ids: Vec<Address>) {
    for proposers_id in proposers_ids.iter() {
        proposers_id.unwrap().require_auth();
    }
}

// This function checks if the proposers are paying the correct proposals fee
pub fn validate_proposers_payment(proposal_fee: &u128, proposers: &Vec<ProposerStat>) -> bool {
    let mut sum: u128 = 0;
    for proposer in proposers.iter() {
        sum = sum + proposer.unwrap().amount;
    }

    proposal_fee == &sum
}

// This function calculates the amount in governance token to pay for the proposal type
pub fn calculate_proposal_vote_price(
    amount: &u128,
    credit_price: &u128,
    proposal_type: &ProposalType,
) -> u128 {
    if proposal_type == &ProposalType::Structural {
        amount * credit_price
    } else {
        let voting_credits = amount.pow(2);
        voting_credits * credit_price
    }
}

pub fn validate_can_vote(
    voter: &Address,
    proposal: &Proposal,
    proposal_votes: &Vec<ProposalVote>,
) -> bool {
    // If the voter voted already, it can't vote again and instead it should update it
    for proposal_vote in proposal_votes.iter() {
        if &proposal_vote.unwrap().voter == voter {
            return false;
        }
    }

    proposal.status == ProposalStatus::Active
}

/// PROPOSALS FUNCTIONS

pub fn get_proposals_fee(env: &Env) -> u128 {
    let core_state: CoreState = env
        .storage()
        .get(&CoreStorageKeys::CoreState)
        .unwrap()
        .unwrap();

    if core_state.proposals_fee == 0 {
        panic_with_error!(&env, SCErrors::ProposalsFeeNotSet);
    }

    core_state.proposals_fee
}

/// This method charges the proposers with the amount they set,
/// this method should only be called after we have made sure the amounts set by the proposers are ok
pub fn charge_proposers(env: &Env, proposers: &Vec<ProposerStat>) {
    let (_, token) = get_governance_token(&env);

    for proposer_result in proposers.iter() {
        let proposer = proposer_result.unwrap();
        token.transfer(
            &proposer.id,
            &env.current_contract_address(),
            &(proposer.amount as i128),
        );
    }
}

pub fn get_proposal(env: &Env, proposal_id: &BytesN<32>) -> Proposal {
    if !env
        .storage()
        .has(&ProposalsStorageKeys::Proposal(proposal_id.clone()))
    {
        panic_with_error!(&env, SCErrors::ProposalDoesntExist);
    }

    env.storage()
        .get(&ProposalsStorageKeys::Proposal(proposal_id.clone()))
        .unwrap()
        .unwrap()
}

pub fn validate_new_proposal_id(env: &Env, proposal_id: &BytesN<32>) {
    if env
        .storage()
        .has(&ProposalsStorageKeys::Proposal(proposal_id.clone()))
    {
        panic_with_error!(&env, SCErrors::ProposalIdAlreadyInUse);
    }
}

pub fn is_voting_time_valid(
    env: &Env,
    voting_time: u64,
    proposal_type: &ProposalType,
    proposers: &Vec<ProposerStat>,
) -> bool {
    match proposal_type {
        ProposalType::Simple => voting_time > 3600 * 24 * 5,
        ProposalType::UpgradeContract => {
            let mut is_admin: bool = false;
            let core_state: CoreState = get_core_state(&env);

            for result in proposers.iter() {
                let proposer: ProposerStat = result.unwrap();
                if proposer.id == core_state.contract_admin {
                    is_admin = true
                }
            }

            if is_admin {
                voting_time > 3600 * 3
            } else {
                voting_time > 3600 * 24 * 14
            }
        }
        ProposalType::UpdateContract => voting_time > 3600 * 24 * 3,
        ProposalType::TreasuryPayment => voting_time > 3600 * 24 * 7,
        ProposalType::Structural => {
            // TODO: Returning false just because we don't allow this yet
            false
        }
    }
}

pub fn new_proposal(
    id: &BytesN<32>,
    proposers: &Vec<ProposerStat>,
    proposal_type: &ProposalType,
    created_at: u64,
    ends_at: u64,
) -> Proposal {
    Proposal {
        id: id.clone(),
        status: ProposalStatus::Active,
        proposal_type: proposal_type.clone(),
        proposers: proposers.clone(),
        credits_for: 0,
        voters_for: 0,
        votes_for: 0,
        credits_against: 0,
        voters_against: 0,
        votes_against: 0,
        created_at,
        ends_at,
    }
}

pub fn get_proposals_ids(env: &Env) -> Vec<BytesN<32>> {
    env.storage()
        .get(&ProposalsStorageKeys::ProposalsIds)
        .unwrap_or(Ok(vec![&env] as Vec<BytesN<32>>))
        .unwrap()
}

pub fn get_proposal_votes(env: &Env, proposal_id: &BytesN<32>) -> Vec<ProposalVote> {
    env.storage()
        .get(&ProposalsStorageKeys::ProposalVotes(proposal_id.clone()))
        .unwrap_or(Ok(vec![&env] as Vec<ProposalVote>))
        .unwrap()
}

pub fn save_new_proposal_id(env: &Env, proposal_id: &BytesN<32>) {
    let mut current_values: Vec<BytesN<32>> = get_proposals_ids(&env);
    current_values.push_front(proposal_id.clone());
    env.storage()
        .set(&ProposalsStorageKeys::ProposalsIds, &current_values);
}

pub fn save_proposal(env: &Env, proposal: &Proposal) {
    env.storage().set(
        &ProposalsStorageKeys::Proposal(proposal.id.clone()),
        proposal,
    );
}

pub fn charge_proposal_vote(env: &Env, voter: &Address, vote_price: &u128) {
    let (_, token_client) = get_governance_token(&env);

    token_client.transfer(
        voter,
        &env.current_contract_address(),
        &(vote_price.clone() as i128),
    );
}

pub fn save_proposal_votes(env: &Env, proposal_id: &BytesN<32>, votes: &Vec<ProposalVote>) {
    env.storage().set(
        &ProposalsStorageKeys::ProposalVotes(proposal_id.clone()),
        votes,
    );
}

pub fn is_proposal_active(env: &Env, proposal: &Proposal) -> bool {
    let timestamp: u64 = env.ledger().timestamp();
    timestamp <= proposal.ends_at
}

#[cfg(test)]
mod test {
    use crate::storage::proposals::ProposalType;
    use crate::tests::test_utils::TEST_VOTING_CREDIT_PRICE;
    use crate::utils::proposals::calculate_proposal_vote_price;

    #[test]
    fn test_calculate_proposal_vote_price() {
        assert_eq!(
            calculate_proposal_vote_price(&1, &TEST_VOTING_CREDIT_PRICE, &ProposalType::Simple),
            TEST_VOTING_CREDIT_PRICE
        );
        assert_eq!(
            calculate_proposal_vote_price(&10, &TEST_VOTING_CREDIT_PRICE, &ProposalType::Simple),
            100 * TEST_VOTING_CREDIT_PRICE
        );
        assert_eq!(
            calculate_proposal_vote_price(&50, &TEST_VOTING_CREDIT_PRICE, &ProposalType::Simple),
            2500 * TEST_VOTING_CREDIT_PRICE
        );

        assert_eq!(
            calculate_proposal_vote_price(&1, &TEST_VOTING_CREDIT_PRICE, &ProposalType::Structural),
            TEST_VOTING_CREDIT_PRICE
        );
        assert_eq!(
            calculate_proposal_vote_price(
                &10,
                &TEST_VOTING_CREDIT_PRICE,
                &ProposalType::Structural
            ),
            10 * TEST_VOTING_CREDIT_PRICE
        );
        assert_eq!(
            calculate_proposal_vote_price(
                &50,
                &TEST_VOTING_CREDIT_PRICE,
                &ProposalType::Structural
            ),
            50 * TEST_VOTING_CREDIT_PRICE
        );
    }
}
