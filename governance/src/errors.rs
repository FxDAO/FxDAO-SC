use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum SCErrors {
    // Core Errors
    ContractAlreadyInitiated = 10001,

    // Proposals Errors
    ProposalsFeeNotSet = 20000,
    InvalidProposalFee = 20001,
    ProposalDoesntExist = 20002,
    ProposalIdAlreadyInUse = 20003,
    ProposalIsNotActive = 20004,
    InvalidVotingTime = 20005,

    // Voting Errors
    CanNotVote = 30000,
}
