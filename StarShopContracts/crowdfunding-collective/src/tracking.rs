use crate::{get_product, types::*};
use soroban_sdk::{Address, Env, Symbol, Vec};

pub fn update_milestone(env: Env, creator: Address, product_id: u32, milestone_id: u32) {
    creator.require_auth();

    let product = get_product(env.clone(), product_id);

    // Validate permissions and state
    if product.creator != creator {
        panic!("Only the creator can update milestones");
    }
    if product.status != ProductStatus::Funded {
        panic!("Product is not funded");
    }

    let mut milestones: Vec<Milestone> = env
        .storage()
        .instance()
        .get(&DataKey::Milestones(product_id))
        .unwrap_or_else(|| Vec::new(&env));

    // Validate milestone ID
    if milestone_id >= milestones.len() as u32 {
        panic!("Invalid milestone ID");
    }

    // Check if previous milestones are completed
    for i in 0..milestone_id {
        if !milestones.get(i).unwrap().completed {
            panic!("Previous milestones not completed");
        }
    }

    let mut milestone = milestones.get(milestone_id).unwrap();
    if milestone.completed {
        panic!("Milestone already completed");
    }

    // Update milestone
    milestone.completed = true;
    milestones.set(milestone_id, milestone);
    env.storage()
        .instance()
        .set(&DataKey::Milestones(product_id), &milestones);

    // Emit milestone event
    env.events().publish(
        (Symbol::new(&env, "milestone_completed"),),
        (product_id, milestone_id),
    );
}

pub fn get_contributions(env: Env, product_id: u32) -> Vec<Contribution> {
    env.storage()
        .instance()
        .get(&DataKey::Contributions(product_id))
        .unwrap_or_else(|| Vec::new(&env))
}

pub fn get_milestones(env: Env, product_id: u32) -> Vec<Milestone> {
    env.storage()
        .instance()
        .get(&DataKey::Milestones(product_id))
        .unwrap_or_else(|| Vec::new(&env))
}
