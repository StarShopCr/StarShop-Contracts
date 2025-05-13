use soroban_sdk::{contracterror, contracttype, symbol_short, Address, String, Symbol, Vec};

// Enum representing the status of a proposal
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalStatus {
    Draft = 0,    // Proposal is in draft state
    Active = 1,   // Proposal is active and open for voting
    Passed = 2,   // Proposal has passed voting
    Rejected = 3, // Proposal has been rejected
    Executed = 4, // Proposal has been executed
    Canceled = 5, // Proposal has been canceled
    Vetoed = 6,   // Proposal has been vetoed
}

// Enum representing the type of a proposal
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalType {
    FeatureRequest = 0,  // Request for a new feature
    PolicyChange = 1,    // Change in policy
    ParameterChange = 2, // Change in parameters
    ContractUpgrade = 3, // Upgrade to the contract
    EmergencyAction = 4, // Emergency action
    EconomicChange = 5,  // Economic change
}

// Struct representing a proposal
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Proposal {
    pub id: u32,                     // Unique ID of the proposal
    pub proposer: Address,           // Address of the proposer
    pub title: Symbol,               // Title of the proposal
    pub description: Symbol,         // Description of the proposal
    pub metadata_hash: String,       // Metadata hash for the proposal
    pub proposal_type: ProposalType, // Type of the proposal
    pub status: ProposalStatus,      // Current status of the proposal
    pub created_at: u64,             // Timestamp when the proposal was created
    pub activated_at: u64,           // Timestamp when the proposal was activated
    pub voting_config: VotingConfig, // Voting configuration for the proposal
    pub actions: Vec<Action>,        // Actions associated with the proposal
}

// Struct representing the requirements for a proposal
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ProposalRequirements {
    pub cooldown_period: u64, // Cooldown period before creating another proposal
    pub required_stake: i128, // Stake required to create a proposal
    pub proposal_limit: u32,  // Maximum number of proposals allowed
    pub max_voting_power: i128, // Maximum voting power allowed
}

// Struct representing a vote
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Vote {
    pub voter: Address, // Address of the voter
    pub support: bool,  // Whether the vote is in support of the proposal
    pub weight: i128,   // Weight of the vote
    pub timestamp: u64, // Timestamp when the vote was cast
}

// Struct representing the voting configuration
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct VotingConfig {
    pub duration: u64,              // Duration of the voting period
    pub quorum: u128,               // Minimum quorum required for the proposal to pass
    pub threshold: u128,            // Minimum threshold required for the proposal to pass
    pub execution_delay: u64,       // Delay before the proposal can be executed
    pub one_address_one_vote: bool, // Whether each address gets one vote
}

// Struct representing a snapshot of weights
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct WeightSnapshot {
    pub proposal_id: u32, // ID of the proposal
    pub snapshot_at: u64, // Timestamp when the snapshot was taken
}

// Struct representing a moderator
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Moderator {
    pub address: Address,  // Address of the moderator
    pub appointed_at: u64, // Timestamp when the moderator was appointed
}

// Enum representing actions that can be performed
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    UpdateProposalRequirements(ProposalRequirements), // Update proposal requirements
    AppointModerator(Address),                        // Appoint a new moderator
    RemoveModerator(Address),                         // Remove an existing moderator
    UpdateRewardRates(RewardRates),                   // Update reward rates
    UpdateLevelRequirements(LevelRequirements),       // Update level requirements
    UpdateAuctionConditions(u32, AuctionConditions),  // Update auction conditions
}

// Custom Errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,          // Contract is already initialized
    NotInitialized = 2,              // Contract is not initialized
    Unauthorized = 3,                // Unauthorized action
    ProposalNotFound = 101,          // Proposal not found
    InvalidProposalStatus = 102,     // Invalid proposal status
    NotEligibleToPropose = 103,      // Not eligible to propose
    ProposalInCooldown = 104,        // Proposal is in cooldown period
    InsufficientStake = 105,         // Insufficient stake to create a proposal
    InvalidProposalType = 106,       // Invalid proposal type
    ProposalLimitReached = 107,      // Proposal limit reached
    InvalidProposalInput = 108,      // Invalid proposal input
    ProposalNotActive = 201,         // Proposal is not active
    AlreadyVoted = 202,              // Voter has already voted
    NoVotingPower = 203,             // No voting power available
    InvalidVotingPeriod = 204,       // Invalid voting period
    InvalidDelegation = 301,         // Invalid delegation
    SelfDelegationNotAllowed = 302,  // Self-delegation is not allowed
    ProposalNotExecutable = 401,     // Proposal is not executable
    ExecutionFailed = 402,           // Execution of the proposal failed
    ExecutionDelayNotMet = 403,      // Execution delay not met
    InvalidAction = 404,             // Invalid action
    NotVerified = 501,               // User is not verified
    UserLevelNotSet = 502,           // User level is not set
    InsufficientReferralLevel = 503, // Insufficient referral level
    ModeratorNotFound = 601,         // Moderator not found
    AlreadyModerator = 602,          // Address is already a moderator
    ContractCallFailed = 701,        // Contract call failed
}

// Constants
pub const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
pub const PROPOSAL_COUNTER_KEY: Symbol = symbol_short!("PCNT");
pub const REQUIREMENTS_KEY: Symbol = symbol_short!("REQS");
pub const TOKEN_KEY: Symbol = symbol_short!("TOKN");
pub const REFERRAL_KEY: Symbol = symbol_short!("REFR");
pub const AUCTION_KEY: Symbol = symbol_short!("AUCT");
pub const DELEGATE_PREFIX: Symbol = symbol_short!("DELG");
pub const PROPOSAL_PREFIX: Symbol = symbol_short!("PROP");
pub const PROPOSAL_STATUS_PREFIX: Symbol = symbol_short!("STAT");
pub const SNAPSHOT_PREFIX: Symbol = symbol_short!("SNAP");
pub const VOTE_PREFIX: Symbol = symbol_short!("VOTE");
pub const WEIGHT_PREFIX: Symbol = symbol_short!("WGHT");
pub const MODERATOR_KEY: Symbol = symbol_short!("MODS");
pub const DEFAULT_CONFIG_KEY: Symbol = symbol_short!("DEFCFG");

// Referral contract types
#[contracttype]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct RewardRates {
    pub silver_rate: i128,
    pub gold_rate: i128,
    pub platinum_rate: i128,
}

#[contracttype]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct LevelCriteria {
    pub required_direct_referrals: u32,
    pub required_team_size: u32,
    pub required_total_rewards: i128,
}

#[contracttype]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct LevelRequirements {
    pub silver: LevelCriteria,
    pub gold: LevelCriteria,
    pub platinum: LevelCriteria,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum UserLevel {
    Basic,
    Silver,
    Gold,
    Platinum,
}

// Auction contract types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuctionConditions {
    pub auction_type: AuctionType,
    pub end_time: u64,
    pub starting_price: i128,
    pub on_bid_count: Option<u32>,
    pub on_target_price: Option<i128>,
    pub on_inactivity_seconds: Option<u64>,
    pub on_fixed_sequence_number: Option<u32>,
    pub on_minimum_participants: Option<u32>,
    pub on_maximum_participants: Option<u32>,
}

#[contracttype]
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum AuctionType {
    Regular,
    Reverse,
    Dutch(DutchAuctionData),
}

#[contracttype]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct DutchAuctionData {
    pub floor_price: i128,
}
