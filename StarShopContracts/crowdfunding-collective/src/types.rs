use soroban_sdk::{contracttype, Address, String};

#[contracttype]
pub enum DataKey {
    Admin,                   // Admin address
    Products(u32),           // Product ID -> Product
    Contributions(u32),      // Product ID -> Vec<Contribution>
    Rewards(u32),            // Product ID -> Vec<RewardTier>
    Milestones(u32),         // Product ID -> Vec<Milestone>
    NextProductId,           // Counter for product IDs
    ContributionsTotal(u32), // Product ID -> Total contributed amount
}

#[contracttype]
#[derive(Clone)]
pub struct Product {
    pub id: u32,
    pub creator: Address,
    pub name: String,
    pub description: String,
    pub funding_goal: u64, // In XLM (stroops)
    pub deadline: u64,     // Ledger timestamp
    pub status: ProductStatus,
    pub total_funded: u64, // Total funds collected
}

#[contracttype]
#[derive(Clone, PartialEq, Debug)] // Added Debug
pub enum ProductStatus {
    Active,
    Funded,
    Failed,
    Completed,
}

#[contracttype]
#[derive(Clone)]
pub struct Contribution {
    pub contributor: Address,
    pub amount: u64, // In XLM (stroops)
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct RewardTier {
    pub id: u32,
    pub min_contribution: u64, // Minimum contribution for this tier
    pub description: String,   // E.g., "Discounted product" or "Exclusive perk"
    pub discount: u32,         // Percentage discount (0-100)
}

#[contracttype]
#[derive(Clone)]
pub struct Milestone {
    pub id: u32,
    pub description: String,
    pub target_date: u64, // Expected completion timestamp
    pub completed: bool,
}
