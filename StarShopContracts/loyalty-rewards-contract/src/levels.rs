use crate::points::PointsManager;
use crate::types::{DataKey, Error, LevelCriteria, LevelRequirements, LoyaltyLevel, UserData, TransactionType};
use soroban_sdk::{Address, Env, Symbol};

pub struct LevelManager;

impl LevelManager {
    /// Initialize level requirements
    pub fn init_level_requirements(
        env: &Env,
        requirements: LevelRequirements,
    ) -> Result<(), Error> {
        // Check if admin
        crate::admin::AdminModule::verify_admin(env)?;

        // Validate requirements (Gold > Silver)
        if requirements.gold.min_points <= requirements.silver.min_points
            || requirements.gold.min_purchases <= requirements.silver.min_purchases
        {
            return Err(Error::InvalidLevelRequirements);
        }

        // Store level requirements
        env.storage()
            .instance()
            .set(&DataKey::LevelRequirements, &requirements);

        Ok(())
    }

    /// Get current level requirements
    pub fn get_level_requirements(env: &Env) -> Result<LevelRequirements, Error> {
        env.storage()
            .instance()
            .get(&DataKey::LevelRequirements)
            .ok_or(Error::InvalidLevelRequirements)
    }

    /// Check and update user's loyalty level
    pub fn check_and_update_level(env: &Env, user: &Address) -> Result<bool, Error> {
        let mut user_data = PointsManager::get_user_data(env, user)?;
        let requirements = Self::get_level_requirements(env)?;

        let new_level = Self::calculate_eligible_level(env, &user_data, &requirements);

        // Only update if new level is higher
        if new_level > user_data.level {
            let previous_level = user_data.level.clone();
            user_data.level = new_level.clone();
            user_data.level_updated_at = env.ledger().timestamp();

            // Save updated user data
            env.storage()
                .persistent()
                .set(&DataKey::User(user.clone()), &user_data);
            
            // Publish event for level change
            env.events().publish(
                (Symbol::new(env, "level_changed"), user.clone()),
                ((previous_level, new_level),),
            );

            return Ok(true);
        }

        Ok(false)
    }

    /// Calculate the highest eligible level for a user
    fn calculate_eligible_level(
        env: &Env,
        user_data: &UserData,
        requirements: &LevelRequirements,
    ) -> LoyaltyLevel {
        // Check Gold requirements first
        if Self::meets_criteria(env, user_data, &requirements.gold) {
            return LoyaltyLevel::Gold;
        }

        // Check Silver requirements
        if Self::meets_criteria(env, user_data, &requirements.silver) {
            return LoyaltyLevel::Silver;
        }

        // Default level is Bronze
        LoyaltyLevel::Bronze
    }

    /// Check if user meets criteria for a specific level
    fn meets_criteria(env: &Env, user_data: &UserData, criteria: &LevelCriteria) -> bool {
        let days_active = (env.ledger().timestamp() - user_data.join_date) / (24 * 60 * 60);

        // Count purchases from transaction history
        let purchase_count = crate::milestones::MilestoneManager::count_purchases(user_data);

        user_data.lifetime_points >= criteria.min_points
            && purchase_count >= criteria.min_purchases
            && days_active >= criteria.min_days_active
    }

    /// Get user's current loyalty level
    pub fn get_user_level(env: &Env, user: &Address) -> Result<LoyaltyLevel, Error> {
        let user_data = PointsManager::get_user_data(env, user)?;
        Ok(user_data.level)
    }

    /// Get time spent at current level
    pub fn get_level_duration(env: &Env, user: &Address) -> Result<u64, Error> {
        let user_data = PointsManager::get_user_data(env, user)?;
        Ok(env.ledger().timestamp() - user_data.level_updated_at)
    }

    /// Award anniversary bonus points if eligible
    pub fn award_anniversary_bonus(env: &Env, user: &Address) -> Result<i128, Error> {
        let user_data = PointsManager::get_user_data(env, user)?;
        let current_time = env.ledger().timestamp();
        const ONE_YEAR_IN_SECONDS: u64 = 31_536_000; // 365 days

        // Check if a year has passed since joining and since the last anniversary bonus
        if current_time - user_data.join_date >= ONE_YEAR_IN_SECONDS
            && current_time - user_data.last_anniversary_awarded >= ONE_YEAR_IN_SECONDS
        {
            let bonus_points = match user_data.level {
                LoyaltyLevel::Bronze => 100,
                LoyaltyLevel::Silver => 250,
                LoyaltyLevel::Gold => 500,
            };

            // Add bonus points
            PointsManager::add_points(
                env,
                user,
                bonus_points,
                Symbol::new(env, "anniversary"),
                TransactionType::Bonus,
                None, // No product_id for an anniversary bonus
                None, // No category for an anniversary bonus
            )?;
            
            // **FIXED**: Re-fetch user_data after it was modified by add_points
            let mut updated_user_data = PointsManager::get_user_data(env, user)?;

            // Update the last awarded timestamp on the fresh data to prevent re-claiming
            updated_user_data.last_anniversary_awarded = current_time;
            env.storage()
                .persistent()
                .set(&DataKey::User(user.clone()), &updated_user_data);

            Ok(bonus_points)
        } else {
            Ok(0) // No bonus awarded
        }
    }
}
