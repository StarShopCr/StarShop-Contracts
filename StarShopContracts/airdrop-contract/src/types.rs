use soroban_sdk::{contracterror, contracttype, Address, Bytes, Map, Symbol};

/// Represents an airdrop event with dynamic eligibility conditions and constraints.
#[contracttype]
pub struct AirdropEvent {
    /// Human-readable name for the airdrop event (e.g., "Loyalty Rewards - July 2025").
    pub name: Symbol,
    /// Detailed description of the airdrop event.
    pub description: Bytes,
    /// Map of condition names to minimum required values (e.g., "purchases" -> 5).
    pub conditions: Map<Symbol, u64>,
    /// Amount of tokens to distribute to each eligible user.
    pub amount: i128,
    /// Address of the token contract (XLM or custom token).
    pub token_address: Address,
    /// Start timestamp (Unix seconds) for the event.
    pub start_time: u64,
    /// End timestamp (Unix seconds) for the event.
    pub end_time: u64,
    /// Optional max number of users who can claim.
    pub max_users: Option<u64>,
    /// Optional max total tokens to distribute.
    pub max_total_amount: Option<i128>,
    /// Whether the event is active (e.g., not paused or canceled).
    pub is_active: bool,
}

/// Statistics for an airdrop event.
#[contracttype]
pub struct EventStats {
    /// Number of users who have claimed.
    pub recipient_count: u64,
    /// Total tokens distributed.
    pub total_amount_distributed: i128,
}

/// Storage keys for persistent data in the contract.
#[contracttype]
pub enum DataKey {
    /// Key for the admin's address.
    Admin,
    /// Key for the current event ID counter.
    EventId,
    /// Key for an airdrop event, identified by its event ID.
    AirdropEvent(u64),
    /// Key to track if a user has claimed an airdrop, identified by event ID and user address.
    Claimed(u64, Address),
    /// Key for event statistics, identified by event ID.
    EventStats(u64),
    /// Key for the provider registry, mapping condition Symbol to provider Address.
    ProviderRegistry(Symbol),
    /// Key for the list of users who claimed an airdrop, identified by event ID.
    ClaimedUsers(u64),
}

/// Error codes for the airdrop contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AirdropError {
    AlreadyInitialized = 1,
    Unauthorized = 2,
    InvalidTokenConfig = 3,
    AirdropNotFound = 4,
    UserNotEligible = 5,
    AlreadyClaimed = 6,
    InsufficientContractBalance = 7,
    TokenTransferFailed = 8,
    ConditionNotFound = 9,
    InvalidAmount = 10,
    ProviderNotConfigured = 11,
    ProviderCallFailed = 12,
    EventInactive = 13,
    CapExceeded = 14,
    InvalidEventConfig = 15,
}
