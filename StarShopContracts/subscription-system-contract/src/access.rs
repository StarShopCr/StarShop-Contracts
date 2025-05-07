use soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol, Vec};

use crate::minting::{is_active, is_in_grace_period};
use crate::plans::{DataKey as PlanKey, Plan};

/// Storage key for tracking admin/manager roles
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum RoleKey {
    Admin,
    Manager,
    Usage(Address, Symbol), // (user, feature)
}

/// Internal: Check access by role
fn require_role(env: &Env, role_key: RoleKey, user: &Address) {
    let allowed: Vec<Address> = env.storage().instance().get(&role_key).unwrap_or_else(|| {
        // Create an empty vector
        Vec::new(&env)
    });
    if !allowed.iter().any(|addr| addr == *user) {
        panic!("access denied: missing role {:?}", role_key);
    }
}

/// Add user to a role (Admin or Manager)
pub fn add_role(env: &Env, role: Symbol, user: Address) {
    let admin_sym = Symbol::new(env, "admin");
    let manager_sym = Symbol::new(env, "manager");

    let key = match role {
        sym if sym == admin_sym => RoleKey::Admin,
        sym if sym == manager_sym => RoleKey::Manager,
        _ => panic!("Invalid role: unauthorized access"),
    };

    let mut current: Vec<Address> = env.storage().instance().get(&key).unwrap_or_else(|| {
        // Create an empty vector
        Vec::new(&env)
    });
    if !current.contains(&user) {
        current.push_back(user);
        env.storage().instance().set(&key, &current);
    }
}

/// Verify user has access to a feature by subscription
fn require_subscription(env: &Env, user: &Address, plan_id: &Symbol) {
    if !is_active(env, user.clone(), plan_id.clone())
        && !is_in_grace_period(env, user.clone(), plan_id.clone())
    {
        panic!("access denied: subscription expired or not found");
    }
}

/// Tier-based access (only certain plans unlock some features)
fn require_plan(env: &Env, user: &Address, plan_id: &Symbol, required_tier: Symbol) {
    require_subscription(env, user, plan_id);

    let plan: Plan = env
        .storage()
        .instance()
        .get(&PlanKey::Plan(plan_id.clone()))
        .expect("plan not found");

    if plan.tier != required_tier {
        panic!("access denied: tier '{:?}' required", required_tier);
    }
}

/// Example 1: Gated content (any valid subscription)
pub fn view_premium_content(env: &Env, user: Address, plan_id: Symbol) -> String {
    require_subscription(env, &user, &plan_id);
    track_usage(env, &user, symbol_short!("premium"));
    String::from_str(&env, "🎉 Welcome to premium content!")
}

/// Example 2: Tier-gated access (e.g., only 'gold' tier users)
pub fn access_gold_feature(env: &Env, user: Address, plan_id: Symbol) -> Symbol {
    require_plan(env, &user, &plan_id, symbol_short!("gold"));
    track_usage(env, &user, symbol_short!("gold_feat"));
    symbol_short!("GOLD")
}

/// Example 3: Admin-only reset logic
pub fn admin_reset_subscription(env: &Env, caller: Address, target_user: Address, plan_id: Symbol) {
    require_role(env, RoleKey::Admin, &caller);

    use crate::minting::MintKey;

    let sub_key = MintKey::Subscription(target_user.clone(), plan_id.clone());

    env.storage().instance().remove(&sub_key);
}

/// Optional: track how many times a user accessed a feature
pub fn track_usage(env: &Env, user: &Address, feature: Symbol) {
    let key = RoleKey::Usage(user.clone(), feature.clone());
    let count: u32 = env.storage().instance().get(&key).unwrap_or(0);
    env.storage().instance().set(&key, &(count + 1));
}

/// Optional: get usage count for a feature
pub fn get_usage_count(env: &Env, user: Address, feature: Symbol) -> u32 {
    let key = RoleKey::Usage(user, feature);
    env.storage().instance().get(&key).unwrap_or(0)
}
