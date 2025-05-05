use crate::tracking::TrackingManager;
use crate::types::{DataKey, Drop, DropStatus, Error};
use soroban_sdk::{Address, Env, Map, String, Symbol, Val, Vec};
use crate::access::AccessManager;

pub struct DropManager;

impl DropManager {
    /// Initialize the drop manager
    pub fn init(env: &Env) {
        // Initialize drop count if not exists
        if !env.storage().instance().has(&DataKey::DropCount) {
            env.storage().instance().set(&DataKey::DropCount, &0u32);
        }
    }

    /// Create a new drop
    pub fn create_drop(
        env: &Env,
        creator: Address,
        title: String,
        product_id: u64,
        max_supply: u32,
        start_time: u64,
        end_time: u64,
        price: i128,
        per_user_limit: u32,
        image_uri: String,
    ) -> Result<u32, Error> {
        // Validate time window
        let current_time = env.ledger().timestamp();
        if start_time <= current_time || end_time <= start_time {
            return Err(Error::InvalidTime);
        }

        // Validate price
        if price <= 0 {
            return Err(Error::InvalidPrice);
        }

        // Get next drop ID
        let drop_count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::DropCount)
            .unwrap_or(0);
        let drop_id = drop_count + 1;

        // Create new drop
        let drop = Drop {
            id: drop_id,
            creator: creator.clone(),
            title,
            product_id,
            max_supply,
            start_time,
            end_time,
            price,
            per_user_limit,
            image_uri,
            status: DropStatus::Pending,
            total_purchased: 0,
        };

        // Store drop
        env.storage().instance().set(&DataKey::Drop(drop_id), &drop);

        // Update drop count
        env.storage().instance().set(&DataKey::DropCount, &drop_id);

        // Initialize purchase tracking
        env.storage()
            .instance()
            .set(&DataKey::DropPurchases(drop_id), &0u32);
        env.storage()
            .instance()
            .set(&DataKey::DropBuyers(drop_id), &Vec::<Address>::new(env));

        // Emit event
        env.events().publish(
            (Symbol::new(env, "drop_created"), creator),
            (drop_id, product_id, start_time, end_time),
        );

        Ok(drop_id)
    }

    /// Get drop details
    pub fn get_drop(env: &Env, drop_id: u32) -> Result<Drop, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Drop(drop_id))
            .ok_or(Error::DropNotFound)
    }

    /// Get total purchases for a drop
    pub fn get_drop_purchases(env: &Env, drop_id: u32) -> Result<u32, Error> {
        env.storage()
            .instance()
            .get(&DataKey::DropPurchases(drop_id))
            .ok_or(Error::DropNotFound)
    }

    /// Update drop status
    pub fn update_status(
        env: &Env,
        admin: &Address,
        drop_id: u32,
        status: DropStatus,
    ) -> Result<(), Error> {
        // Verify admin
        let contract_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        if admin != &contract_admin {
            return Err(Error::Unauthorized);
        }

        let mut drop: Drop = Self::get_drop(env, drop_id)?;
        drop.status = status;
        env.storage().instance().set(&DataKey::Drop(drop_id), &drop);
        Ok(())
    }

    /// Check if drop is active
    pub fn is_active(env: &Env, drop_id: u32) -> Result<bool, Error> {
        let drop = Self::get_drop(env, drop_id)?;
        let current_time = env.ledger().timestamp();

        Ok(drop.status == DropStatus::Active
            && current_time >= drop.start_time
            && current_time <= drop.end_time)
    }

    /// Check if drop has ended
    pub fn has_ended(env: &Env, drop_id: u32) -> Result<bool, Error> {
        let drop = Self::get_drop(env, drop_id)?;
        let current_time = env.ledger().timestamp();

        Ok(drop.status == DropStatus::Completed
            || drop.status == DropStatus::Cancelled
            || current_time > drop.end_time)
    }

    /// Check if drop has started
    pub fn has_started(env: &Env, drop_id: u32) -> Result<bool, Error> {
        let drop = Self::get_drop(env, drop_id)?;
        let current_time = env.ledger().timestamp();

        Ok(current_time >= drop.start_time)
    }

    /// Process a purchase
    pub fn purchase(env: &Env, buyer: Address, drop_id: u32, quantity: u32) -> Result<(), Error> {
        // Verify purchase access (whitelist and user level)
        AccessManager::verify_purchase_access(env, &buyer)?;

        // Get drop
        let mut drop = Self::get_drop(env, drop_id)?;

        // Check if drop is active
        if !Self::is_active(env, drop_id)? {
            return Err(Error::DropNotActive);
        }

        // Check supply
        if drop.total_purchased + quantity > drop.max_supply {
            return Err(Error::InsufficientSupply);
        }

        // Check user limit
        let user_purchases = TrackingManager::get_user_purchases(env, &buyer, drop_id);
        if user_purchases + quantity > drop.per_user_limit {
            return Err(Error::UserLimitExceeded);
        }

        // Record purchase
        TrackingManager::record_purchase(env, &buyer, drop_id, quantity, drop.price)?;

        // Update drop total
        drop.total_purchased += quantity;
        env.storage().instance().set(&DataKey::Drop(drop_id), &drop);

        Ok(())
    }
}
