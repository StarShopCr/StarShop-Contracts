use soroban_sdk::{contracterror, contracttype, symbol_short, Address, String, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalStatus {
    Draft = 0,
    Active = 1,
    Passed = 2,
    Rejected = 3,
    Executed = 4,
    Canceled = 5,
    Vetoed = 6,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalType {
    FeatureRequest = 0,
    PolicyChange = 1,
    ParameterChange = 2,
    ContractUpgrade = 3,
    EmergencyAction = 4,
    EconomicChange = 5,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Proposal {
    pub id: u32,
    pub proposer: Address,
    pub title: Symbol,
    pub description: Symbol,
    pub metadata_hash: String,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub created_at: u64,
    pub activated_at: u64,
    pub voting_config: VotingConfig,
    pub actions: Vec<Action>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ProposalRequirements {
    pub cooldown_period: u64,
    pub required_stake: i128,
    pub proposal_limit: u32,
    pub max_voting_power: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Vote {
    pub voter: Address,
    pub support: bool,
    pub weight: i128,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct VotingConfig {
    pub duration: u64,
    pub quorum: u128,
    pub threshold: u128,
    pub execution_delay: u64,
    pub one_address_one_vote: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct WeightSnapshot {
    pub proposal_id: u32,
    pub snapshot_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Moderator {
    pub address: Address,
    pub appointed_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    UpdateProposalRequirements(ProposalRequirements),
    AppointModerator(Address),
    RemoveModerator(Address),
    UpdateRewardRates(RewardRates),
    UpdateLevelRequirements(LevelRequirements),
    UpdateAuctionConditions(u32, AuctionConditions),
}

// Custom Errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    ProposalNotFound = 101,
    InvalidProposalStatus = 102,
    NotEligibleToPropose = 103,
    ProposalInCooldown = 104,
    InsufficientStake = 105,
    InvalidProposalType = 106,
    ProposalLimitReached = 107,
    ProposalNotActive = 201,
    AlreadyVoted = 202,
    NoVotingPower = 203,
    InvalidVotingPeriod = 204,
    InvalidDelegation = 301,
    SelfDelegationNotAllowed = 302,
    ProposalNotExecutable = 401,
    ExecutionFailed = 402,
    ExecutionDelayNotMet = 403,
    InvalidAction = 404,
    NotVerified = 501,
    UserLevelNotSet = 502,
    InsufficientReferralLevel = 503,
    ModeratorNotFound = 601,
    AlreadyModerator = 602,
    ContractCallFailed = 701,
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
