use crate::types::*;
use soroban_sdk::{Address, Env, String, Symbol, Vec};

pub fn create_product(
    env: Env,
    creator: Address,
    name: String,
    description: String,
    funding_goal: u64,
    deadline: u64,
    reward_tiers: Vec<RewardTier>,
    milestones: Vec<Milestone>,
) -> u32 {
    creator.require_auth();

    // Validate inputs
    if funding_goal == 0 {
        panic!("Funding goal must be greater than zero");
    }
    if deadline <= env.ledger().timestamp() {
        panic!("Deadline must be in the future");
    }
    if name.len() < 3 || name.len() > 100 {
        panic!("Name must be between 3-100 characters");
    }
    if description.len() > 500 {
        panic!("Description too long");
    }
    if reward_tiers.is_empty() {
        panic!("At least one reward tier required");
    }
    if milestones.is_empty() {
        panic!("At least one milestone required");
    }

    // Validate reward tiers
    let mut prev_min = 0;
    for (_i, tier) in reward_tiers.iter().enumerate() {
        if tier.min_contribution <= prev_min {
            panic!("Reward tiers must be in ascending order");
        }
        if tier.discount > 100 {
            panic!("Discount cannot exceed 100%");
        }
        prev_min = tier.min_contribution;
    }

    // Validate milestones
    for (i, milestone) in milestones.iter().enumerate() {
        if milestone.id != i as u32 {
            panic!("Milestone IDs must be sequential starting from 0");
        }
        if milestone.completed {
            panic!("Milestones cannot be created as completed");
        }
    }

    // Get next product ID
    let product_id = next_product_id(&env);

    // Create product
    let product = Product {
        id: product_id,
        creator: creator.clone(),
        name: name.clone(),
        description: description.clone(),
        funding_goal,
        deadline,
        status: ProductStatus::Active,
        total_funded: 0,
    };

    // Store product
    env.storage()
        .instance()
        .set(&DataKey::Products(product_id), &product);

    // Store reward tiers and milestones
    env.storage()
        .instance()
        .set(&DataKey::Rewards(product_id), &reward_tiers);
    env.storage()
        .instance()
        .set(&DataKey::Milestones(product_id), &milestones);

    // Initialize contributions
    let contributions: Vec<Contribution> = Vec::new(&env);
    env.storage()
        .instance()
        .set(&DataKey::Contributions(product_id), &contributions);
    env.storage()
        .instance()
        .set(&DataKey::ContributionsTotal(product_id), &0u64);

    // Emit creation event
    env.events().publish(
        (Symbol::new(&env, "product_created"),),
        (product_id, creator, name, funding_goal, deadline),
    );

    product_id
}

pub fn get_product(env: Env, product_id: u32) -> Product {
    env.storage()
        .instance()
        .get(&DataKey::Products(product_id))
        .unwrap_or_else(|| panic!("Product not found"))
}

fn next_product_id(env: &Env) -> u32 {
    let product_id = env
        .storage()
        .instance()
        .get(&DataKey::NextProductId)
        .unwrap_or(1u32);
    env.storage()
        .instance()
        .set(&DataKey::NextProductId, &(product_id + 1));
    product_id
}
