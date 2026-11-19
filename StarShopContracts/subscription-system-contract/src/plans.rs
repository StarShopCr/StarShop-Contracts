use soroban_sdk::{contracttype, Address, Bytes, Env, Symbol};

#[derive(Clone)]
#[contracttype]
pub struct Plan {
    pub id: Symbol,      // e.g., "gold", "silver"
    pub name: Bytes,     // e.g., "Gold Membership"
    pub duration: u64,   // in seconds
    pub price: u64,      // in stroops or smallest unit
    pub benefits: Bytes, // human-readable description
    pub active: bool,    // is plan currently offered?
    pub version: u32,    // track updates
    pub tier: Symbol,    // tier level (1, 2, 3)
}

#[contracttype]
pub enum DataKey {
    Plan(Symbol),
    Admin, // Optional: Admin address
    Owner, // Optional: Owner address
}

/// Helper to fetch the stored admin, initializing it if absent.
fn get_or_init_admin(env: &Env, caller: &Address) -> Address {
    // Try to read the stored admin.
    if let Some(stored) = env.storage().instance().get(&DataKey::Admin) {
        stored
    } else {
        // No admin set yet – initialize with the caller.
        env.storage().instance().set(&DataKey::Admin, caller);
        caller.clone()
    }
}

/// Store a new plan (admin only)
pub fn create_plan(
    env: &Env,
    admin: Address,
    plan_id: Symbol,
    name: Bytes,
    duration: u64,
    price: u64,
    benefits: Bytes,
    version: u32,
    tier: Symbol,
) {
    // Ensure admin is initialized and matches the caller.
    let stored_admin = get_or_init_admin(env, &admin);
    assert_eq!(stored_admin, admin, "only admin can create plans");

    // Validate input
    assert!(duration > 0, "duration must be > 0");
    assert!(price > 0, "price must be > 0");

    // Check if plan already exists
    if env
        .storage()
        .instance()
        .has(&DataKey::Plan(plan_id.clone()))
    {
        panic!("Plan already exists");
    }

    let plan = Plan {
        id: plan_id.clone(),
        name,
        duration,
        price,
        benefits,
        active: true,
        version,
        tier,
    };

    env.storage().instance().set(&DataKey::Plan(plan_id), &plan);
}

/// Get a plan by ID
pub fn get_plan(env: &Env, plan_id: Symbol) -> Option<Plan> {
    env.storage().instance().get(&DataKey::Plan(plan_id))
}

/// Update a plan (e.g., price/duration), keeping same ID
pub fn update_plan(
    env: &Env,
    admin: Address,
    plan_id: Symbol,
    name: Bytes,
    duration: u64,
    price: u64,
    benefits: Bytes,
    version: u32,
    tier: Symbol,
) {
    let stored_admin = get_or_init_admin(env, &admin);
    assert_eq!(stored_admin, admin, "only admin can update plans");

    if !env
        .storage()
        .instance()
        .has(&DataKey::Plan(plan_id.clone()))
    {
        panic!("Plan does not exist");
    }

    let updated_plan = Plan {
        id: plan_id.clone(),
        name,
        duration,
        price,
        benefits,
        active: true,
        version,
        tier,
    };

    env.storage()
        .instance()
        .set(&DataKey::Plan(plan_id), &updated_plan);
}

/// Admin can deactivate a plan
pub fn disable_plan(env: &Env, admin: Address, plan_id: Symbol) {
    let stored_admin = get_or_init_admin(env, &admin);
    assert_eq!(stored_admin, admin, "only admin can disable plans");

    let mut plan: Plan = env
        .storage()
        .instance()
        .get(&DataKey::Plan(plan_id.clone()))
        .expect("plan not found");

    plan.active = false;
    env.storage().instance().set(&DataKey::Plan(plan_id), &plan);
}