use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

use crate::plans::{DataKey as PlanKey, Plan};

/// Represents a single NFT-based subscription owned by a user
#[derive(Clone)]
#[contracttype]
pub struct SubscriptionNFT {
    pub plan_id: Symbol,
    pub user: Address,
    pub start_time: u64,
    pub expiry_time: u64,
    pub version: u32,
}

/// Storage keys for minted subscriptions and user plan tracking
#[contracttype]
pub enum MintKey {
    Subscription(Address, Symbol), // (user, plan_id)
    UserPlans(Address),            // Track all plans for a user
}

/// Duration of the grace period in seconds (e.g., 3 days)
const GRACE_PERIOD: u64 = 3 * 86400;

/// Helper: Track which plans a user has subscribed to
fn _track_user_plan(env: &Env, user: Address, plan_id: Symbol) {
    let mut plans = env
        .storage()
        .instance()
        .get::<_, Vec<Symbol>>(&MintKey::UserPlans(user.clone()))
        .unwrap_or_else(|| {
            // Create an empty vector
            Vec::new(env)
        });

    if !plans.contains(&plan_id) {
        plans.push_back(plan_id.clone());
        env.storage()
            .instance()
            .set(&MintKey::UserPlans(user), &plans);
    }
}

/// Mint a new subscription NFT for a given user and plan.
/// This stores the subscription info and enforces:
/// - Unique subscription per user per plan
/// - Plan must be active
/// - NFT holds metadata like start time and expiry
pub fn mint_subscription(env: &Env, user: Address, plan_id: Symbol) {
    // Fetch and validate plan
    let plan: Plan = env
        .storage()
        .instance()
        .get(&PlanKey::Plan(plan_id.clone()))
        .expect("plan not found");

    assert!(plan.active, "plan is not active");

    // Check if user is already subscribed to this plan
    let sub_key = MintKey::Subscription(user.clone(), plan_id.clone());
    if env.storage().instance().has(&sub_key) {
        panic!("user already subscribed to this plan");
    }

    let current_time = env.ledger().timestamp();
    let expiry_time = current_time + plan.duration;

    let subscription = SubscriptionNFT {
        plan_id: plan_id.clone(),
        user: user.clone(),
        start_time: current_time,
        expiry_time,
        version: plan.version,
    };

    env.storage().instance().set(&sub_key, &subscription);
}

/// Check if a user's subscription is currently active (not expired)
pub fn is_active(env: &Env, user: Address, plan_id: Symbol) -> bool {
    let sub_key = MintKey::Subscription(user.clone(), plan_id.clone());

    if let Some(nft) = env.storage().instance().get::<_, SubscriptionNFT>(&sub_key) {
        let now = env.ledger().timestamp();
        now < nft.expiry_time
    } else {
        false
    }
}

/// Check if a user's subscription is within the grace period
pub fn is_in_grace_period(env: &Env, user: Address, plan_id: Symbol) -> bool {
    let sub_key = MintKey::Subscription(user.clone(), plan_id.clone());

    if let Some(nft) = env.storage().instance().get::<_, SubscriptionNFT>(&sub_key) {
        let now = env.ledger().timestamp();
        now >= nft.expiry_time && now < nft.expiry_time + GRACE_PERIOD
    } else {
        false
    }
}

/// Internal helper for checking if user is expired (optional use in renewal/expiration logic)
pub fn is_expired(env: &Env, user: Address, plan_id: Symbol) -> bool {
    let sub_key = MintKey::Subscription(user.clone(), plan_id.clone());

    if let Some(nft) = env.storage().instance().get::<_, SubscriptionNFT>(&sub_key) {
        let now = env.ledger().timestamp();
        now >= nft.expiry_time
    } else {
        true // If no subscription exists, consider it expired
    }
}

/// Renew an existing subscription (only after expiry)
/// Reuses the same plan and version, but updates time window
pub fn renew_subscription(env: &Env, user: Address, plan_id: Symbol) {
    let plan: Plan = env
        .storage()
        .instance()
        .get(&PlanKey::Plan(plan_id.clone()))
        .expect("plan not found");

    assert!(plan.active, "plan is not active");

    let sub_key = MintKey::Subscription(user.clone(), plan_id.clone());

    let current_time = env.ledger().timestamp();

    // Check subscription exists and has expired
    if let Some(mut nft) = env.storage().instance().get::<_, SubscriptionNFT>(&sub_key) {
        if current_time < nft.expiry_time {
            panic!("subscription is still active; cannot renew early");
        }

        nft.start_time = current_time;
        nft.expiry_time = current_time + plan.duration;
        nft.version = plan.version;

        env.storage().instance().set(&sub_key, &nft);
    } else {
        panic!("no existing subscription to renew");
    }
}

/// Admin-only function to reset or renew any user's subscription forcibly
pub fn _admin_reset_subscription(env: &Env, admin: Address, user: Address, plan_id: Symbol) {
    // You can replace this with your actual role validation
    // e.g., assert_is_admin(env, &admin);
    if admin != env.current_contract_address() {
        panic!("only contract admin can reset subscriptions");
    }

    let plan: Plan = env
        .storage()
        .instance()
        .get(&PlanKey::Plan(plan_id.clone()))
        .expect("plan not found");

    let current_time = env.ledger().timestamp();
    let expiry_time = current_time + plan.duration;

    let subscription = SubscriptionNFT {
        plan_id: plan_id.clone(),
        user: user.clone(),
        start_time: current_time,
        expiry_time,
        version: plan.version,
    };

    env.storage().instance().set(
        &MintKey::Subscription(user.clone(), plan_id.clone()),
        &subscription,
    );
    _track_user_plan(env, user, plan_id); // ensure plan tracking
}

/// List all plans a user is subscribed to (historically)
pub fn _get_user_plans(env: &Env, user: Address) -> Vec<Symbol> {
    env.storage()
        .instance()
        .get::<_, Vec<Symbol>>(&MintKey::UserPlans(user))
        .unwrap_or_else(|| {
            // Create an empty vector
            Vec::new(&env)
        })
}

/// Burn/remove a user's subscription NFT
pub fn _burn_subscription(env: &Env, user: Address, plan_id: Symbol) {
    let key = MintKey::Subscription(user, plan_id);
    env.storage().instance().remove(&key);
}
