use soroban_sdk::{contracttype, vec, Address, BytesN, RawVal, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalType {
    /// A simple proposal is a proposal in which community is asked to decide on something off-chain.
    /// This could be something like "Start the development of a dark version of the UI".
    /// Simple proposals can be used as an "entry plebiscite" where something is requested and later when it's time to deploy the change, the community needs to approved it which it means it's an "exit plebiscite"
    /// Proposers need to understand that they will need to spend twice so in some situations using another type of proposal could be a better option
    Simple,
    /// This type of proposal must be used when there is the need to update any of the contracts where this governance contract is the admin
    /// This type of proposal is the only proposal that can be done in an "urgent" way which means the protocol maintainer can request an express proposal (as low as 1hr before voting ends)
    UpgradeContract,
    /// Different contracts in the protocol have certain parameters that can be updated with a proposal
    /// Things like fees, percentages, behavior, etc.
    UpdateContract,
    /// Move funds from the treasury to any address
    /// This could be used for multiple reason like paying a provider, a donation, grants, allocation of funds, etc  
    TreasuryPayment,
    // TODO: Structural is not supported yet so it needs to be added
    Structural,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ProposerStat {
    pub id: Address,
    pub amount: u128,
}

#[contracttype]
#[derive(Debug, PartialEq)]
pub enum ProposalStatus {
    Active,
    Accepted,
    Denied,
    Cancelled,
}

#[contracttype]
#[derive(Clone)]
pub struct TreasuryPaymentProposalParams {
    pub recipient: Address,
    pub amount: u128,
}

#[contracttype]
#[derive(Clone)]
pub enum TreasuryPaymentProposalOption {
    None,
    Some(TreasuryPaymentProposalParams),
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub struct UpdateContractProposalParams {
    pub contract_id: Address,
    pub function_name: Symbol,
    pub params: Vec<RawVal>,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum UpdateContractProposalOption {
    None,
    Some(UpdateContractProposalParams),
}

#[contracttype]
#[derive(Clone)]
pub struct ProposalExecutionParams {
    pub treasury_payment: TreasuryPaymentProposalOption,
    pub update_contract: UpdateContractProposalOption,
}

#[contracttype]
pub struct Proposal {
    /// The proposal id is the SHA256 hash of the text used in the proposal
    pub id: BytesN<32>,
    pub status: ProposalStatus,
    pub proposal_type: ProposalType,
    pub proposers: Vec<ProposerStat>,
    pub credits_for: u128,
    pub voters_for: u128,
    pub votes_for: u128,
    pub credits_against: u128,
    pub voters_against: u128,
    pub votes_against: u128,
    pub created_at: u64,
    pub ends_at: u64,
    pub emergency_proposal: bool,
    pub execution_params: ProposalExecutionParams,
    pub executed: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalVoteType {
    For,
    Against,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ProposalVote {
    pub proposal_id: BytesN<32>,
    pub voter: Address,
    pub amount: u128,
    pub vote_type: ProposalVoteType,
}

#[contracttype]
pub struct ProposalVoteIndex {
    pub proposal_id: BytesN<32>,
    pub voter_id: Address,
}

#[contracttype]
pub enum ProposalsStorageKeys {
    /// A Vec with the Ids of the proposals sorted from newest to oldest
    ProposalsIds,

    Proposal(BytesN<32>),

    /// A Vec<ProposalVoteIndex> value with the votes in order from newest to oldest
    ProposalVotes(BytesN<32>),

    /// The struct returned is ProposalVote
    ProposalVote(ProposalVoteIndex),
}
