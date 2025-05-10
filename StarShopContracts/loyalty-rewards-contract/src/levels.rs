use crate::points::PointsManager;
use crate::types::{DataKey, Error, LevelCriteria, LevelRequirements, LoyaltyLevel, UserData};
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

        // Validate requirements
        if requirements.gold.min_points <= requirements.silver.min_points
            || requirements.gold.min_purchases <= requirements.silver.min_purchases
            || requirements.gold.min_days_active <= requirements.silver.min_days_active
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
            user_data.level = new_level;
            user_data.level_updated_at = env.ledger().timestamp();

            // Save updated user data
            env.storage()
                .persistent()
                .set(&DataKey::User(user.clone()), &user_data);

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
        // Check Gold requirements
        if Self::meets_criteria(env, user_data, &requirements.gold) {
            return LoyaltyLevel::Gold;
        }

        // Check Silver requirements
        if Self::meets_criteria(env, user_data, &requirements.silver) {
            return LoyaltyLevel::Silver;
        }

        // Default level
        LoyaltyLevel::Bronze
    }

    /// Check if user meets criteria for a level
    fn meets_criteria(env: &Env, user_data: &UserData, criteria: &LevelCriteria) -> bool {
        let current_time = env.ledger().timestamp();
        let days_active = (current_time - user_data.join_date) / (24 * 60 * 60);

        // Count purchases from transaction history
        let purchases = crate::milestones::MilestoneManager::count_purchases(user_data);

        user_data.lifetime_points >= criteria.min_points
            && purchases >= criteria.min_purchases
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
        let current_time = env.ledger().timestamp();

        Ok(current_time - user_data.level_updated_at)
    }

    /// Check if user is eligible for anniversary reward
    pub fn check_anniversary(env: &Env, user: &Address) -> Result<bool, Error> {
        let user_data = PointsManager::get_user_data(env, user)?;
        let current_time = env.ledger().timestamp();

        // Check if it's been approximately a year since level update
        // 365.25 days in seconds = 31,557,600
        let anniversary_seconds = 31_557_600;
        let time_at_level = current_time - user_data.level_updated_at;

        // Check if time at level is a multiple of approximately a year
        // with a small buffer of 1 day (86,400 seconds)
        if time_at_level > anniversary_seconds && time_at_level % anniversary_seconds < 86_400 {
            return Ok(true);
        }

        Ok(false)
    }

    /// Award anniversary bonus points
    pub fn award_anniversary_bonus(env: &Env, user: &Address) -> Result<i128, Error> {
        if !Self::check_anniversary(env, user)? {
            return Ok(0);
        }

        let user_data = PointsManager::get_user_data(env, user)?;

        // Bonus points based on level
        let bonus_points = match user_data.level {
            LoyaltyLevel::Bronze => 100,
            LoyaltyLevel::Silver => 250,
            LoyaltyLevel::Gold => 500,
        };

        // Add bonus points
        crate::points::PointsManager::add_points(
            env,
            user,
            bonus_points,
            Symbol::new(env, "anniversary"),
            crate::types::TransactionType::Bonus,
        )?;

        Ok(bonus_points)
    }
}
