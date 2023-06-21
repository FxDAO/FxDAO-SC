#![cfg(test)]
extern crate std;

use crate::errors::SCErrors;
use crate::storage::proposals::{Proposal, ProposalStatus, ProposalType, ProposerStat};
use crate::tests::test_utils::{create_test_data, init_contract, TestData};

use soroban_sdk::testutils::{Address as __, BytesN as _};
use soroban_sdk::{vec, Address, BytesN, Env, IntoVal, RawVal, Status, Symbol, Vec};

#[test]
pub fn test_creating_new_proposal_single_proposer() {
    let env = Env::default();
    env.mock_all_auths();
    let test_data: TestData = create_test_data(&env);
    init_contract(&test_data);

    let voting_time: u64 = 3600 * 24 * 7;
    let proposer: Address = Address::random(&env);
    let id: BytesN<32> = BytesN::random(&env);
    let proposal_type: ProposalType = ProposalType::Simple;
    let proposers: Vec<ProposerStat> = vec![
        &env,
        ProposerStat {
            amount: 12_00_000_0000000,
            id: proposer.clone(),
        },
    ] as Vec<ProposerStat>;

    test_data
        .governance_token
        .mint(&proposer, &12_00_000_0000000);

    test_data.contract_client.create_proposal(
        &id,
        &proposal_type,
        &proposers,
        &voting_time,
        &false,
        &test_data.dumb_params,
    );

    // Check the function is requiring the sender approved this operation
    assert_eq!(
        env.auths(),
        std::vec![
            (
                proposer.clone(),
                test_data.contract_client.address.clone(),
                Symbol::new(&env, "create_proposal"),
                (
                    id.clone(),
                    proposal_type.clone(),
                    proposers.clone(),
                    voting_time.clone(),
                    false,
                    test_data.dumb_params.clone(),
                )
                    .into_val(&env),
            ),
            (
                proposer.clone(),
                test_data.governance_token.address.clone(),
                Symbol::new(&env, "transfer"),
                (
                    proposer.clone(),
                    test_data.contract_client.address.clone(),
                    12_00_000_0000000 as i128,
                )
                    .into_val(&env),
            )
        ]
    );

    // Account should be without governance tokens
    assert_eq!(test_data.governance_token.spendable_balance(&proposer), 0);

    // If we try to create a new proposal with the same id it should fail
    let id_already_in_use_error_result = test_data
        .contract_client
        .try_create_proposal(
            &id,
            &proposal_type,
            &proposers,
            &voting_time,
            &false,
            &test_data.dumb_params,
        )
        .unwrap_err();

    assert_eq!(
        id_already_in_use_error_result,
        Ok(Status::from_contract_error(
            SCErrors::ProposalIdAlreadyInUse as u32
        ))
    );

    // If we try again and the proposer doesn't have funds it should fail
    let not_funds_fail = test_data
        .contract_client
        .try_create_proposal(
            &BytesN::random(&env),
            &proposal_type,
            &proposers,
            &voting_time,
            &false,
            &test_data.dumb_params,
        )
        .unwrap_err();

    assert_eq!(not_funds_fail, Ok(Status::from_contract_error(10)));

    // Confirm the proposal was saved
    let proposal: Proposal = test_data.contract_client.get_proposal(&id);
    assert_eq!(proposal.proposers, proposers);
    assert_eq!(proposal.id, id);
    assert_eq!(proposal.status, ProposalStatus::Active);
    assert_eq!(proposal.proposal_type, proposal_type);
    assert_eq!(proposal.proposers, proposers);
    assert_eq!(proposal.credits_for, 0);
    assert_eq!(proposal.voters_for, 0);
    assert_eq!(proposal.votes_for, 0);
    assert_eq!(proposal.credits_against, 0);
    assert_eq!(proposal.voters_against, 0);
    assert_eq!(proposal.votes_against, 0);
    assert_eq!(proposal.created_at, env.ledger().timestamp());
    assert_eq!(
        proposal.ends_at,
        env.ledger().timestamp() + voting_time.clone()
    );
}

#[test]
pub fn test_create_new_proposal_multiple_proposers() {
    let env = Env::default();
    env.mock_all_auths();
    let test_data: TestData = create_test_data(&env);
    let voting_time: u64 = 36000 * 24 * 7;
    init_contract(&test_data);

    let proposer_stat_1: ProposerStat = ProposerStat {
        id: Address::random(&env),
        amount: 2_00_000_0000000,
    };
    let proposer_stat_2: ProposerStat = ProposerStat {
        id: Address::random(&env),
        amount: 4_00_000_0000000,
    };
    let proposer_stat_3: ProposerStat = ProposerStat {
        id: Address::random(&env),
        amount: 5_00_000_0000000,
    };
    let proposer_stat_4: ProposerStat = ProposerStat {
        id: Address::random(&env),
        amount: 1_00_000_0000000,
    };

    let id: BytesN<32> = BytesN::random(&env);
    let proposal_type: ProposalType = ProposalType::Simple;
    let proposers: Vec<ProposerStat> = vec![
        &env,
        proposer_stat_1.clone(),
        proposer_stat_2.clone(),
        proposer_stat_3.clone(),
        proposer_stat_4.clone(),
    ] as Vec<ProposerStat>;

    for proposer in proposers.clone().iter() {
        test_data
            .governance_token
            .mint(&proposer.unwrap().id, &12_00_000_0000000);
    }

    test_data.contract_client.create_proposal(
        &id,
        &proposal_type,
        &proposers,
        &voting_time,
        &false,
        &test_data.dumb_params,
    );

    // Check that all of the proposers signed it and were authorized
    let mut value = std::vec![] as std::vec::Vec<(Address, Address, Symbol, Vec<RawVal>)>;
    for item in proposers.iter() {
        let proposer: ProposerStat = item.unwrap();
        value.push((
            proposer.id.clone(),
            test_data.contract_client.address.clone(),
            Symbol::new(&env, "create_proposal"),
            (
                id.clone(),
                proposal_type.clone(),
                proposers.clone(),
                voting_time.clone(),
                false,
                test_data.dumb_params.clone(),
            )
                .into_val(&env),
        ));
        value.push((
            proposer.id.clone(),
            test_data.governance_token.address.clone(),
            Symbol::new(&env, "transfer"),
            (
                proposer.id.clone(),
                test_data.contract_client.address.clone(),
                proposer.amount.clone() as i128,
            )
                .into_val(&env),
        ));
    }

    assert_eq!(env.auths(), value);

    // Check that all of the funds were charged correctly
    assert_eq!(
        test_data
            .governance_token
            .spendable_balance(&test_data.contract_client.address),
        12_00_000_0000000
    );

    for proposer in proposers.iter() {
        let proposer_stat = proposer.unwrap();
        assert_eq!(
            test_data
                .governance_token
                .spendable_balance(&proposer_stat.id),
            12_00_000_0000000 - (proposer_stat.amount as i128)
        );
    }

    // Confirm the proposal was saved
    let proposal: Proposal = test_data.contract_client.get_proposal(&id);
    assert_eq!(proposal.proposers, proposers);
}

#[test]
pub fn test_proposals_ids() {
    let env: Env = Env::default();
    env.mock_all_auths();
    let test_data: TestData = create_test_data(&env);
    init_contract(&test_data);

    let voting_time: u64 = 3600 * 24 * 7;
    let proposer: Address = Address::random(&env);
    let proposal_type: ProposalType = ProposalType::Simple;
    let proposers: Vec<ProposerStat> = vec![
        &env,
        ProposerStat {
            amount: 12_00_000_0000000,
            id: proposer.clone(),
        },
    ] as Vec<ProposerStat>;
    let id_1: BytesN<32> = BytesN::random(&env);
    let id_2: BytesN<32> = BytesN::random(&env);
    let id_3: BytesN<32> = BytesN::random(&env);
    let id_4: BytesN<32> = BytesN::random(&env);

    let ids: [BytesN<32>; 4] = [id_1, id_2, id_3, id_4];

    test_data
        .governance_token
        .mint(&proposer, &(12_00_000_0000000 * 4));

    for id in ids.iter() {
        test_data.contract_client.create_proposal(
            &id,
            &proposal_type,
            &proposers,
            &voting_time,
            &false,
            &test_data.dumb_params,
        );
    }

    let proposal_ids: Vec<BytesN<32>> = test_data.contract_client.get_proposals_ids();

    let mut reversed_ids = ids.clone();
    reversed_ids.reverse();

    assert_eq!(proposal_ids, Vec::from_array(&env, reversed_ids));
}

// #[test]
// pub fn test_simple_proposal_voting() {
//     todo!()
// }
//
// #[test]
// pub fn test_multiple_proposal_voting() {
//     todo!()
// }

// #[test]
// pub fn test_end_proposal() {
//     todo!()
// }

// #[test]
// pub fn text_execute_proposal_result() {
//     todo!()
// }
