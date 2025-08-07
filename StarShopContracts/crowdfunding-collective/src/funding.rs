use crate::types::*;
use soroban_sdk::{Address, Env, Symbol, Vec};

pub fn contribute(env: Env, contributor: Address, product_id: u32, amount: u64) {
    contributor.require_auth();

    let mut product = get_product(&env, product_id);

    // Validate product state
    if product.status != ProductStatus::Active {
        panic!("Product is not active");
    }
    if env.ledger().timestamp() > product.deadline {
        panic!("Funding period has ended");
    }
    if amount == 0 {
        panic!("Contribution must be greater than zero");
    }

    // Check for overflow
    let total_funded = env
        .storage()
        .instance()
        .get(&DataKey::ContributionsTotal(product_id))
        .unwrap_or(0u64);
    let new_total = total_funded
        .checked_add(amount)
        .expect("Contribution overflow");

    if new_total > product.funding_goal {
        panic!("Contribution would exceed funding goal");
    }

    // Update contributions
    let mut contributions: Vec<Contribution> = env
        .storage()
        .instance()
        .get(&DataKey::Contributions(product_id))
        .unwrap_or_else(|| Vec::new(&env));

    contributions.push_back(Contribution {
        contributor: contributor.clone(),
        amount,
        timestamp: env.ledger().timestamp(),
    });

    env.storage()
        .instance()
        .set(&DataKey::Contributions(product_id), &contributions);

    // Update total funded
    env.storage()
        .instance()
        .set(&DataKey::ContributionsTotal(product_id), &new_total);

    // Update product status if goal reached
    product.total_funded = new_total;
    if product.total_funded >= product.funding_goal {
        product.status = ProductStatus::Funded;
        env.events()
            .publish((Symbol::new(&env, "ProductFunded"), product_id), product_id);
    }

    env.storage()
        .instance()
        .set(&DataKey::Products(product_id), &product);

    // Emit contribution event
    env.events().publish(
        (Symbol::new(&env, "contributed"),),
        (product_id, contributor, amount),
    );
}

pub fn distribute_funds(env: Env, caller: Address, product_id: u32) {
    caller.require_auth();

    let product = get_product(&env, product_id);
    if product.status != ProductStatus::Funded {
        panic!("Product is not funded");
    }
    if caller != product.creator {
        panic!("Only creator can distribute funds");
    }

    let milestones: Vec<Milestone> = env
        .storage()
        .instance()
        .get(&DataKey::Milestones(product_id))
        .unwrap_or_else(|| Vec::new(&env));

    for milestone in milestones.iter() {
        if !milestone.completed {
            panic!("Not all milestones are completed");
        }
    }

    let mut product = get_product(&env, product_id);
    product.status = ProductStatus::Completed;
    env.storage()
        .instance()
        .set(&DataKey::Products(product_id), &product);

    // Emit funds distributed event
    env.events().publish(
        (Symbol::new(&env, "FundsDistributed"), product_id),
        product.total_funded,
    );
}

pub fn refund_contributors(env: Env, caller: Address, product_id: u32) {
    caller.require_auth();

    let product = get_product(&env, product_id);
    if product.status != ProductStatus::Active {
        panic!("Product is not active");
    }
    if env.ledger().timestamp() <= product.deadline {
        panic!("Funding period has not ended");
    }
    if product.total_funded >= product.funding_goal {
        panic!("Product was funded, cannot refund");
    }

    // Only creator or admin can trigger refunds
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| panic!("Admin not set"));

    if caller != product.creator && caller != admin {
        panic!("Unauthorized refund attempt");
    }

    let mut product = get_product(&env, product_id);
    product.status = ProductStatus::Failed;
    env.storage()
        .instance()
        .set(&DataKey::Products(product_id), &product);

    let contributions: Vec<Contribution> = env
        .storage()
        .instance()
        .get(&DataKey::Contributions(product_id))
        .unwrap_or_else(|| Vec::new(&env));

    for contribution in contributions.iter() {
        env.events().publish(
            (Symbol::new(&env, "RefundIssued"), product_id),
            (contribution.contributor, contribution.amount),
        );
    }

    // Clear contributions
    env.storage().instance().set(
        &DataKey::Contributions(product_id),
        &Vec::<Contribution>::new(&env),
    );
    env.storage()
        .instance()
        .set(&DataKey::ContributionsTotal(product_id), &0u64);

    // Emit refund completed event
    env.events()
        .publish((Symbol::new(&env, "RefundCompleted"),), product_id);
}

fn get_product(env: &Env, product_id: u32) -> Product {
    env.storage()
        .instance()
        .get(&DataKey::Products(product_id))
        .unwrap_or_else(|| panic!("Product not found"))
}
