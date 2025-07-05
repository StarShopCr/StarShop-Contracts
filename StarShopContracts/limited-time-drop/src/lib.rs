#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, panic_with_error, Address, Env, Map, String, Symbol, Val,
    Vec,
};

mod access;
mod drop;
mod interface;
mod tracking;
mod types;

use crate::access::AccessManager;
use crate::drop::DropManager;
use crate::tracking::TrackingManager;
use crate::types::{DataKey, Drop, DropStatus, Error, PurchaseRecord, UserLevel};

#[contract]
pub struct LimitedTimeDropContract;

#[contractimpl]
impl LimitedTimeDropContract {
    /// Initialize the contract with an admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);

        // Initialize managers
        DropManager::init(&env);
        AccessManager::init(&env);
        TrackingManager::init(&env);

        env.events().publish((Symbol::new(&env, "init"),), (admin,));
        Ok(())
    }

    /// Create a new drop
    pub fn create_drop(
        env: Env,
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
        creator.require_auth();
        DropManager::create_drop(
            &env,
            creator,
            title,
            product_id,
            max_supply,
            start_time,
            end_time,
            price,
            per_user_limit,
            image_uri,
        )
    }

    /// Purchase from a drop
    pub fn purchase(env: Env, buyer: Address, drop_id: u32, quantity: u32) -> Result<(), Error> {
        buyer.require_auth();
        DropManager::purchase(&env, buyer, drop_id, quantity)
    }

    /// Get drop details
    pub fn get_drop(env: Env, drop_id: u32) -> Result<Drop, Error> {
        DropManager::get_drop(&env, drop_id)
    }

    /// Get purchase history for a user
    pub fn get_purchase_history(
        env: Env,
        user: Address,
        drop_id: u32,
    ) -> Result<Vec<PurchaseRecord>, Error> {
        TrackingManager::get_purchase_history(&env, user, drop_id)
    }

    /// Get total purchases for a drop
    pub fn get_drop_purchases(env: Env, drop_id: u32) -> Result<u32, Error> {
        DropManager::get_drop_purchases(&env, drop_id)
    }

    /// Get buyer list for a drop
    pub fn get_buyer_list(env: Env, drop_id: u32) -> Result<Vec<Address>, Error> {
        TrackingManager::get_buyer_list(&env, drop_id)
    }

    /// Add a user to the whitelist (Admin only)
    pub fn add_to_whitelist(env: Env, admin: Address, user: Address) -> Result<(), Error> {
        AccessManager::add_to_whitelist(&env, &admin, &user)
    }

    /// Set a user's access level (Admin only)
    pub fn set_user_level(env: Env, admin: Address, user: Address, level: UserLevel) -> Result<(), Error> {
        AccessManager::set_user_level(&env, &admin, &user, level)
    }
}

#[cfg(test)]
mod test;
