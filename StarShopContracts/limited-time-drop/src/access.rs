use crate::types::{DataKey, Error, UserLevel};
use soroban_sdk::{Address, Env, Map, Symbol, Vec};

pub struct AccessManager;

impl AccessManager {
    /// Initialize the access manager
    pub fn init(env: &Env) {
        // Initialize whitelist if not exists
        if !env.storage().instance().has(&DataKey::Whitelist) {
            env.storage()
                .instance()
                .set(&DataKey::Whitelist, &Vec::<Address>::new(env));
        }
    }

    /// Add user to whitelist
    pub fn add_to_whitelist(env: &Env, admin: &Address, user: &Address) -> Result<(), Error> {
        // Verify admin authentication and authorization

        Self::verify_admin(env, admin)?;

        let mut whitelist: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Whitelist)
            .unwrap_or_else(|| Vec::new(env));

        // Check if already whitelisted
        if whitelist.contains(user) {
            return Err(Error::DuplicateWhitelistEntry);
        }

        whitelist.push_back(user.clone());
        env.storage()
            .instance()
            .set(&DataKey::Whitelist, &whitelist);

        Ok(())
    }

    /// Remove user from whitelist
    pub fn remove_from_whitelist(env: &Env, admin: &Address, user: &Address) -> Result<(), Error> {
        // Verify admin authentication and authorization
        Self::verify_admin(env, admin)?;

        let mut whitelist: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Whitelist)
            .unwrap_or_else(|| Vec::new(env));

        // Remove user if exists
        if let Some(index) = whitelist.first_index_of(user) {
            whitelist.remove(index);
            env.storage()
                .instance()
                .set(&DataKey::Whitelist, &whitelist);
        }

        Ok(())
    }

    /// Check if user is whitelisted
    pub fn is_whitelisted(env: &Env, user: &Address) -> bool {
        let whitelist: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Whitelist)
            .unwrap_or_else(|| Vec::new(env));

        whitelist.contains(user)
    }

    /// Set user level
    pub fn set_user_level(
        env: &Env,
        admin: &Address,
        user: &Address,
        level: UserLevel,
    ) -> Result<(), Error> {
        Self::verify_admin(env, admin)?; // Checks this address is the admin
                                         // Set user level (all enum variants are valid)
        env.storage()
            .instance()
            .set(&DataKey::UserLevels(user.clone()), &level);
        Ok(())
    }

    /// Get user level
    pub fn get_user_level(env: &Env, user: &Address) -> UserLevel {
        env.storage()
            .instance()
            .get(&DataKey::UserLevels(user.clone()))
            .unwrap_or(UserLevel::Standard)
    }

    /// Verify admin
    pub fn verify_admin(env: &Env, admin: &Address) -> Result<(), Error> {
        let contract_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        if admin != &contract_admin {
            return Err(Error::Unauthorized);
        }

        Ok(())
    }

    /// Verify user can purchase
    pub fn verify_purchase_access(env: &Env, user: &Address) -> Result<(), Error> {
        // Check whitelist
        if !Self::is_whitelisted(env, user) {
            return Err(Error::NotWhitelisted);
        }

        // Check user level
        let user_level = Self::get_user_level(env, user);
        if user_level == UserLevel::Standard {
            return Err(Error::InsufficientLevel);
        }

        Ok(())
    }
}
