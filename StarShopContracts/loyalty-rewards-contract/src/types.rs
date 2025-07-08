use soroban_sdk::{contracterror, contracttype, Address, Symbol, Vec};

/// Loyalty levels with increasing benefits
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum LoyaltyLevel {
    Bronze = 0, // Basic level
    Silver = 1, // Intermediate level
    Gold = 2,   // Premium level
}

/// Points transaction types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransactionType {
    Earned,  // Points earned from purchases or activities
    Spent,   // Points spent on rewards
    Expired, // Points that have expired (marker, not used in calculation)
    Bonus,   // Bonus points from milestones or promotions
}

/// Transaction record for points
#[contracttype]
#[derive(Clone, Debug)]
pub struct PointsTransaction {
    pub amount: i128,
    pub transaction_type: TransactionType,
    pub timestamp: u64,
    pub description: Symbol,
    pub expiration: u64, // When these points expire
    pub product_id: Option<Symbol>, // To track product-specific milestones
    pub category: Option<Symbol>,   // To track category-specific milestones
}

/// User data containing all loyalty-related information
#[contracttype]
#[derive(Clone, Debug)]
pub struct UserData {
    pub address: Address,
    pub lifetime_points: i128, // Total points earned over time
    pub level: LoyaltyLevel,   // Current loyalty level
    pub level_updated_at: u64, // When user reached current level
    pub transactions: Vec<PointsTransaction>, // History of point transactions
    pub completed_milestones: Vec<u32>, // IDs of completed milestones
    pub join_date: u64,        // When user joined the program
    pub last_anniversary_awarded: u64, // Timestamp of the last anniversary bonus
}

/// Milestone achievement criteria and rewards
#[contracttype]
#[derive(Clone)]
pub struct Milestone {
    pub id: u32,
    pub name: Symbol,
    pub description: Symbol,
    pub points_reward: i128, // Points awarded for completion
    pub requirement: MilestoneRequirement,
}

/// Different types of milestone requirements
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MilestoneRequirement {
    TotalPurchases(u32),      // Number of purchases made
    SpendAmount(i128),        // Total amount spent (approximated)
    PointsEarned(i128),       // Total points earned
    SpecificProduct(Symbol),  // Purchase of specific product
    SpecificCategory(Symbol), // Purchase in specific category
    DaysActive(u64),          // Days as a member
}

/// Requirements for each loyalty level
#[contracttype]
#[derive(Clone)]
pub struct LevelRequirements {
    pub silver: LevelCriteria, // Requirements for Silver
    pub gold: LevelCriteria,   // Requirements for Gold
}

/// Criteria for level upgrades
#[contracttype]
#[derive(Clone)]
pub struct LevelCriteria {
    pub min_points: i128,      // Minimum lifetime points required
    pub min_purchases: u32,    // Minimum purchases required
    pub min_days_active: u64,  // Minimum days as member
}

/// Reward item that can be redeemed with points
#[contracttype]
#[derive(Clone)]
pub struct Reward {
    pub id: u32,
    pub name: Symbol,
    pub description: Symbol,
    pub points_cost: i128, // Points required to redeem
    pub reward_type: RewardType,
    pub min_level: LoyaltyLevel, // Minimum level required to redeem
    pub max_per_user: u32, // Max redemptions per user (0 for unlimited)
}

/// Types of rewards available
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RewardType {
    Discount(u32),        // Percentage discount (basis points)
    Product(Symbol),      // Free product identifier
    XLM(i128),            // XLM amount (in stroops)
    Token(Address, i128), // Other token address and amount
}

/// Storage keys for contract data
#[contracttype]
pub enum DataKey {
    Admin,                      // Contract administrator
    User(Address),              // User data
    Milestone(u32),             // Milestone data
    Reward(u32),                // Reward data
    LevelRequirements,          // Level upgrade criteria
    PointsExpiryDays,           // Days until points expire
    MaxRedemptionPercentage,    // Max % of purchase that can be paid with points
    PointsPerPurchaseRatio,     // Points earned per purchase amount unit
    ProductCategoryBonus(Symbol),// Bonus points for specific categories
    ProductBonus(Symbol),       // Bonus points for specific products
    TotalMilestones,            // Counter for total milestones
    TotalRewards,               // Counter for total rewards
    UserRedemption(UserRedemption), // Tracks (user, reward) redemption count
}

/// Key for tracking user redemptions of a specific reward
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserRedemption(pub Address, pub u32);


/// Contract error types
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,            // Contract not initialized
    AlreadyInitialized = 2,        // Contract already setup
    Unauthorized = 3,              // Caller lacks permission
    UserNotFound = 4,              // User doesn't exist
    InsufficientPoints = 5,        // Not enough points
    InvalidAmount = 6,             // Invalid amount
    MilestoneNotFound = 7,         // Milestone not found
    MilestoneAlreadyCompleted = 8, // Milestone already completed
    RewardNotFound = 9,            // Reward not found
    InsufficientLoyaltyLevel = 10, // User level too low for reward
    MaxRedemptionExceeded = 11,    // Exceeds max redemption percentage
    InvalidPointsExpiry = 12,      // Invalid points expiry period
    InvalidLevelRequirements = 13, // Invalid level requirements
    PointsExpired = 14,            // Points have expired
    ProductNotFound = 15,          // Product not found
    CategoryNotFound = 16,         // Category not found
    MilestoneNotEligible = 17,     // Milestone not eligible
    RewardLimitReached = 18,       // Reward limit reached
}
