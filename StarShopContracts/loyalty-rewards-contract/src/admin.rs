use crate::types::{DataKey, Error};
use soroban_sdk::{Address, Env, Symbol};

pub struct AdminModule;

impl AdminModule {
    /// Initialize the contract with an admin
    pub fn init(env: &Env, admin: &Address) -> Result<(), Error> {
        // Check if already initialized
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        // Require authorization from the admin
        admin.require_auth();

        // Store admin address
        env.storage().instance().set(&DataKey::Admin, admin);

        // Initialize default settings
        let default_expiry_days = 365u64; // 1 year
        let default_max_redemption = 5000u32; // 50%
        let default_points_ratio = 100u32; // 1 point per 100 units

        env.storage()
            .instance()
            .set(&DataKey::PointsExpiryDays, &default_expiry_days);
        env.storage()
            .instance()
            .set(&DataKey::MaxRedemptionPercentage, &default_max_redemption);
        env.storage()
            .instance()
            .set(&DataKey::PointsPerPurchaseRatio, &default_points_ratio);

        // Initialize counters
        env.storage().instance().set(&DataKey::TotalMilestones, &0u32);
        env.storage().instance().set(&DataKey::TotalRewards, &0u32);


        // Publish contract initialization event
        env.events().publish(
            (Symbol::new(env, "contract_initialized"),),
            ((
                admin,
                default_expiry_days,
                default_max_redemption,
                default_points_ratio,
                env.ledger().timestamp(),
            ),),
        );

        Ok(())
    }

    /// Verify if caller is admin
    pub fn verify_admin(env: &Env) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        // Require authorization from admin
        admin.require_auth();

        Ok(())
    }

    /// Update admin address
    pub fn update_admin(env: &Env, new_admin: &Address) -> Result<(), Error> {
        // Verify current admin
        Self::verify_admin(env)?;

        // Update admin address
        env.storage().instance().set(&DataKey::Admin, new_admin);

        Ok(())
    }

    /// Set points expiry period in days
    pub fn set_points_expiry(env: &Env, days: u64) -> Result<(), Error> {
        // Verify admin
        Self::verify_admin(env)?;

        // Validate days
        if days == 0 {
            return Err(Error::InvalidPointsExpiry);
        }

        // Update expiry period
        env.storage()
            .instance()
            .set(&DataKey::PointsExpiryDays, &days);

        Ok(())
    }

    /// Set maximum redemption percentage (in basis points, e.g. 5000 = 50%)
    pub fn set_max_redemption_percentage(env: &Env, percentage_bps: u32) -> Result<(), Error> {
        // Verify admin
        Self::verify_admin(env)?;

        // Validate percentage (cannot exceed 100%)
        if percentage_bps > 10000 {
            return Err(Error::InvalidAmount);
        }

        // Update max redemption percentage
        env.storage()
            .instance()
            .set(&DataKey::MaxRedemptionPercentage, &percentage_bps);

        Ok(())
    }

    /// Set points per purchase ratio
    pub fn set_points_ratio(env: &Env, ratio: u32) -> Result<(), Error> {
        // Verify admin
        Self::verify_admin(env)?;

        // Validate ratio (cannot be zero)
        if ratio == 0 {
            return Err(Error::InvalidAmount);
        }

        // Update points ratio
        env.storage()
            .instance()
            .set(&DataKey::PointsPerPurchaseRatio, &ratio);

        Ok(())
    }

    /// Set bonus points for a product category
    pub fn set_category_bonus(
        env: &Env,
        category: &soroban_sdk::Symbol,
        bonus_bps: u32,
    ) -> Result<(), Error> {
        // Verify admin
        Self::verify_admin(env)?;

        // Update category bonus
        env.storage()
            .instance()
            .set(&DataKey::ProductCategoryBonus(category.clone()), &bonus_bps);

        Ok(())
    }

    /// Set bonus points for a specific product
    pub fn set_product_bonus(
        env: &Env,
        product_id: &soroban_sdk::Symbol,
        bonus_bps: u32,
    ) -> Result<(), Error> {
        // Verify admin
        Self::verify_admin(env)?;

        // Update product bonus
        env.storage()
            .instance()
            .set(&DataKey::ProductBonus(product_id.clone()), &bonus_bps);

        Ok(())
    }

    /// Get current admin
    pub fn get_admin(env: &Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }

    /// Get current points expiry period
    pub fn get_points_expiry(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::PointsExpiryDays)
            .unwrap_or(365) // Default to 1 year
    }

    /// Get current max redemption percentage
    pub fn get_max_redemption_percentage(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::MaxRedemptionPercentage)
            .unwrap_or(5000) // Default to 50%
    }

    /// Get current points per purchase ratio
    pub fn get_points_ratio(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::PointsPerPurchaseRatio)
            .unwrap_or(100) // Default to 1 point per 100 units
    }
}
