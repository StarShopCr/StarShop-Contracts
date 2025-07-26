use soroban_sdk::{contracterror, contracttype, Address, Map, Symbol, Vec};

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[contracttype]
#[derive(Debug)]
pub enum VoteType {
    Upvote = 1,
    Downvote = 2,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    VotingPeriodEnded = 1,
    AlreadyVoted = 2,
    ReversalWindowExpired = 3,
    DailyLimitReached = 4,
    AccountTooNew = 5,
    ProductNotFound = 6,
    ProductExists = 7,
    InvalidInput = 8,
    // SECURITY FIX: Add new error types for authorization
    Unauthorized = 9,
    NotInitialized = 10,
    AdminOnly = 11,
    InvalidAdmin = 12,
    AlreadyInitialized = 13,
}

// SECURITY FIX: Enhanced Product struct with audit trail
#[derive(Clone)]
#[contracttype]
pub struct Product {
    pub id: Symbol,
    pub name: Symbol,
    pub created_at: u64,
    pub creator: Address,
    pub votes: Map<Address, Vote>,
    pub vote_history: Vec<VoteHistoryEntry>, // AUDIT TRAIL: Complete vote history
    pub is_active: bool,
}

// SECURITY FIX: Individual vote with current state
#[derive(Clone)]
#[contracttype]
pub struct Vote {
    pub vote_type: VoteType,
    pub timestamp: u64,
    pub voter: Address,
    pub last_modified: u64,
}

// AUDIT TRAIL: Immutable vote history entry
#[derive(Clone)]
#[contracttype]
pub struct VoteHistoryEntry {
    pub voter: Address,
    pub vote_type: VoteType,
    pub timestamp: u64,
    pub action: VoteAction,
    pub previous_vote: Option<VoteType>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[contracttype]
#[derive(Debug)]
pub enum VoteAction {
    NewVote = 1,
    ChangeVote = 2,
    RemoveVote = 3,
}

// AUTHORIZATION: Admin configuration
#[derive(Clone)]
#[contracttype]
pub struct AdminConfig {
    pub admin: Address,
    pub initialized: bool,
    pub max_products_per_user: u32,
    pub voting_period_days: u32,
    pub reversal_window_hours: u32,
}

// EVENTS: Contract events for transparency
#[derive(Clone)]
#[contracttype]
pub struct ProductCreatedEvent {
    pub product_id: Symbol,
    pub name: Symbol,
    pub creator: Address,
    pub timestamp: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct VoteCastEvent {
    pub product_id: Symbol,
    pub voter: Address,
    pub vote_type: VoteType,
    pub timestamp: u64,
    pub previous_vote: Option<VoteType>,
}

// DATA KEYS for storage organization
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Products,
    Rankings,
    UserVotes,
    ProductCount,
    UserProductCounts,
}
