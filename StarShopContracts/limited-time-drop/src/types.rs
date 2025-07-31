use soroban_sdk::{contracterror, contracttype, Address, String};

/// Storage keys for contract data
#[contracttype]
pub enum DataKey {
    Admin,                  // Contract administrator
    Drop(u32),              // Drop ID -> Drop
    DropCount,              // Counter for drop IDs
    UserPurchases(Address), // User -> Map<DropID, PurchaseRecord>
    DropPurchases(u32),     // Drop ID -> Total purchases
    DropBuyers(u32),        // Drop ID -> Vec<Buyer>
    Whitelist,              // Whitelisted addresses
    UserLevels(Address),    // User -> Level
}

/// Represents a limited-time drop
#[contracttype]
#[derive(Clone)]
pub struct Drop {
    pub id: u32,
    pub creator: Address,
    pub title: String,
    pub product_id: u64,
    pub max_supply: u32,
    pub start_time: u64,
    pub end_time: u64,
    pub price: i128,
    pub per_user_limit: u32,
    pub image_uri: String,
    pub status: DropStatus,
    pub total_purchased: u32,
}

/// Status of a drop
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DropStatus {
    Pending,   // Drop created but not started
    Active,    // Drop is currently active
    Completed, // Drop ended successfully
    Cancelled, // Drop was cancelled
}

/// Record of a purchase
#[contracttype]
#[derive(Clone)]
pub struct PurchaseRecord {
    pub drop_id: u32,
    pub quantity: u32,
    pub timestamp: u64,
    pub price_paid: i128,
}

/// User level for access control
#[contracttype]
#[derive(Clone, PartialEq)]
pub enum UserLevel {
    Standard, // Basic access
    Premium,  // Premium features
    Verified, // KYC verified
}

/// Contract error types
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,           // Contract not initialized
    AlreadyInitialized = 2,       // Contract already setup
    Unauthorized = 3,             // Caller lacks permission
    DropNotFound = 4,             // Drop doesn't exist
    DropNotActive = 5,            // Drop is not active
    DropEnded = 6,                // Drop has ended
    DropNotStarted = 7,           // Drop hasn't started yet
    InsufficientSupply = 8,       // Not enough items left
    UserLimitExceeded = 9,        // User purchase limit reached
    InvalidQuantity = 10,         // Invalid purchase quantity
    InvalidTime = 11,             // Invalid time window
    InvalidPrice = 12,            // Invalid price
    NotWhitelisted = 13,          // User not whitelisted
    InsufficientLevel = 14,       // User level too low
    InvalidUserLevel = 15,        // Invalid user level
    PurchaseFailed = 16,          // Purchase transaction failed
    DuplicateWhitelistEntry = 17, // Duplicate whitelisted address
    InvalidStatusTransition = 18, // Invalid drop status transition
}
