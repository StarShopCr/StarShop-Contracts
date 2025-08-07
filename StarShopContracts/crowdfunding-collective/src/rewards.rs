use crate::types::*;
use soroban_sdk::{Address, Env, Symbol, Vec};

pub fn claim_reward(env: Env, contributor: Address, product_id: u32) {
    contributor.require_auth();

    let product: Product = env
        .storage()
        .instance()
        .get(&DataKey::Products(product_id))
        .unwrap_or_else(|| panic!("Product not found"));

    if product.status != ProductStatus::Completed {
        panic!("Product is not completed");
    }

    // Check if reward already claimed
    let claim_key = DataKey::RewardClaimed(product_id, contributor.clone());
    if env.storage().instance().has(&claim_key) {
        panic!("Reward already claimed");
    }

    // Get contributor's total contribution
    let contributions: Vec<Contribution> = env
        .storage()
        .instance()
        .get(&DataKey::Contributions(product_id))
        .unwrap_or_else(|| Vec::new(&env));

    let total_contributed: u64 = contributions
        .iter()
        .filter(|c| c.contributor == contributor)
        .map(|c| c.amount)
        .sum();

    if total_contributed == 0 {
        panic!("No contributions found for this contributor");
    }

    // Find highest eligible reward tier
    let reward_tiers: Vec<RewardTier> = env
        .storage()
        .instance()
        .get(&DataKey::Rewards(product_id))
        .unwrap_or_else(|| Vec::new(&env));

    let mut eligible_tier: Option<RewardTier> = None;
    for tier in reward_tiers.iter() {
        if total_contributed >= tier.min_contribution {
            if eligible_tier.is_none()
                || tier.min_contribution > eligible_tier.as_ref().unwrap().min_contribution
            {
                eligible_tier = Some(tier.clone());
            }
        }
    }

    if eligible_tier.is_none() {
        panic!("No eligible reward tier found for contribution amount");
    }

    // Mark reward as claimed
    env.storage().instance().set(&claim_key, &true);

    // Emit reward claim event
    env.events().publish(
        (Symbol::new(&env, "reward_claimed"),),
        (product_id, contributor, eligible_tier.unwrap().id),
    );
}

pub fn get_reward_tiers(env: Env, product_id: u32) -> Vec<RewardTier> {
    env.storage()
        .instance()
        .get(&DataKey::Rewards(product_id))
        .unwrap_or_else(|| Vec::new(&env))
}
