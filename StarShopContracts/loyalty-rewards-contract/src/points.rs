use crate::types::{DataKey, Error, PointsTransaction, TransactionType, UserData};
use soroban_sdk::{symbol_short, Address, Env, Symbol};

pub struct PointsManager;

impl PointsManager {
    /// Initialize a new user in the system
    pub fn register_user(env: &Env, user: &Address) -> Result<(), Error> {
        if Self::user_exists(env, user) {
            return Ok(());
        }

        let user_data = UserData {
            address: user.clone(),
            current_points: 0,
            lifetime_points: 0,
            level: crate::types::LoyaltyLevel::Bronze,
            level_updated_at: env.ledger().timestamp(),
            transactions: soroban_sdk::vec![env],
            completed_milestones: soroban_sdk::vec![env],
            join_date: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::User(user.clone()), &user_data);
        Ok(())
    }

    /// Add points to a user's account
    pub fn add_points(
        env: &Env,
        user: &Address,
        amount: i128,
        description: Symbol,
        transaction_type: TransactionType,
    ) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut user_data = Self::get_user_data(env, user)?;

        // Calculate expiration date for these points
        let expiry_days = env
            .storage()
            .instance()
            .get::<_, u64>(&DataKey::PointsExpiryDays)
            .unwrap_or(365); // Default to 1 year if not set

        let current_time = env.ledger().timestamp();
        let expiration = current_time + (expiry_days * 24 * 60 * 60); // Convert days to seconds

        // Create transaction record
        let transaction = PointsTransaction {
            user: user.clone(),
            amount,
            transaction_type: transaction_type.clone(),
            timestamp: current_time,
            description,
            expiration,
        };

        // Update user data
        user_data.transactions.push_back(transaction.clone());
        user_data.current_points += amount;

        // Only update lifetime points for earned or bonus points
        if transaction_type == TransactionType::Earned || transaction_type == TransactionType::Bonus
        {
            user_data.lifetime_points += amount;
        }

        // Save updated user data
        env.storage()
            .persistent()
            .set(&DataKey::User(user.clone()), &user_data);

        // Publish event for points added
        env.events().publish(
            (Symbol::new(env, "points_added"), user.clone()),
            (transaction,),
        );

        Ok(())
    }

    /// Spend points from a user's account
    pub fn spend_points(
        env: &Env,
        user: &Address,
        amount: i128,
        description: Symbol,
    ) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut user_data = Self::get_user_data(env, user)?;

        // Check if user has enough points
        if user_data.current_points < amount {
            return Err(Error::InsufficientPoints);
        }

        // Create transaction record for spent points
        let transaction = PointsTransaction {
            user: user.clone(),
            amount: -amount, // Negative amount for spending
            transaction_type: TransactionType::Spent,
            timestamp: env.ledger().timestamp(),
            description,
            expiration: 0, // No expiration for spent points
        };

        // Update user data
        user_data.transactions.push_back(transaction.clone());
        user_data.current_points -= amount;

        // Save updated user data
        env.storage()
            .persistent()
            .set(&DataKey::User(user.clone()), &user_data);

        // Publish event for points spent
        env.events().publish(
            (Symbol::new(env, "points_spent"), user.clone()),
            (transaction,),
        );

        Ok(())
    }

    /// Process expired points
    pub fn process_expired_points(env: &Env, user: &Address) -> Result<i128, Error> {
        let mut user_data = Self::get_user_data(env, user)?;
        let current_time = env.ledger().timestamp();
        let mut total_expired = 0;

        // Create a new transaction for expired points if any are found
        let mut has_expired = false;

        // Calculate points to expire
        for transaction in user_data.transactions.iter() {
            // Only check earned or bonus points that have an expiration
            if (transaction.transaction_type == TransactionType::Earned
                || transaction.transaction_type == TransactionType::Bonus)
                && transaction.expiration > 0
                && transaction.expiration < current_time
                && transaction.amount > 0
            {
                total_expired += transaction.amount;
                has_expired = true;
            }
        }

        if has_expired && total_expired > 0 {
            // Create an expiration transaction
            let expiration_transaction = PointsTransaction {
                user: user.clone(),
                amount: -total_expired, // Negative amount for expiration
                transaction_type: TransactionType::Expired,
                timestamp: current_time,
                description: Symbol::new(env, "points_expired"),
                expiration: 0, // No expiration for this record
            };

            user_data.transactions.push_back(expiration_transaction);
            user_data.current_points -= total_expired;

            // Save updated user data
            env.storage()
                .persistent()
                .set(&DataKey::User(user.clone()), &user_data);
        }

        Ok(total_expired)
    }

    /// Get user's current points balance
    pub fn get_points_balance(env: &Env, user: &Address) -> Result<i128, Error> {
        // Process any expired points first
        Self::process_expired_points(env, user)?;

        let user_data = Self::get_user_data(env, user)?;
        Ok(user_data.current_points)
    }

    /// Get user's lifetime points (total earned)
    pub fn get_lifetime_points(env: &Env, user: &Address) -> Result<i128, Error> {
        let user_data = Self::get_user_data(env, user)?;
        Ok(user_data.lifetime_points)
    }

    /// Calculate points for a purchase
    pub fn calculate_purchase_points(
        env: &Env,
        purchase_amount: i128,
        product_id: Option<Symbol>,
        category: Option<Symbol>,
    ) -> i128 {
        let base_ratio = env
            .storage()
            .instance()
            .get::<_, u32>(&DataKey::PointsPerPurchaseRatio)
            .unwrap_or(100); // Default 1 point per 100 units

        // Calculate base points
        let mut points = (purchase_amount * 10000) / (base_ratio as i128);

        // Add category bonus if applicable
        if let Some(cat) = category {
            if let Some(bonus) = env
                .storage()
                .instance()
                .get::<_, u32>(&DataKey::ProductCategoryBonus(cat))
            {
                points += (points * bonus as i128) / 10000; // Bonus as percentage in basis points
            }
        }

        // Add product bonus if applicable
        if let Some(prod) = product_id {
            if let Some(bonus) = env
                .storage()
                .instance()
                .get::<_, u32>(&DataKey::ProductBonus(prod))
            {
                points += (points * bonus as i128) / 10000; // Bonus as percentage in basis points
            }
        }

        points
    }

    /// Record points for a purchase
    pub fn record_purchase_points(
        env: &Env,
        user: &Address,
        purchase_amount: i128,
        product_id: Option<Symbol>,
        category: Option<Symbol>,
    ) -> Result<i128, Error> {
        // Ensure user exists
        if !Self::user_exists(env, user) {
            Self::register_user(env, user)?;
        }

        // Calculate points
        let points = Self::calculate_purchase_points(env, purchase_amount, product_id, category);

        // Add points to user account
        Self::add_points(
            env,
            user,
            points,
            symbol_short!("purchase"),
            TransactionType::Earned,
        )?;

        Ok(points)
    }

    /// Check if user exists in the system
    pub fn user_exists(env: &Env, user: &Address) -> bool {
        env.storage().persistent().has(&DataKey::User(user.clone()))
    }

    /// Get user data from storage
    pub fn get_user_data(env: &Env, user: &Address) -> Result<UserData, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::User(user.clone()))
            .ok_or(Error::UserNotFound)
    }

    /// Change user's loyalty level
    pub fn change_level(
        env: &Env,
        user: &Address,
        new_level: crate::types::LoyaltyLevel,
    ) -> Result<(), Error> {
        let mut user_data = Self::get_user_data(env, user)?;

        // Don't update if level is the same
        if user_data.level == new_level {
            return Ok(());
        }

        // Record previous level for the event
        let previous_level = user_data.level;

        // Clone new_level before moving it
        let new_level_clone = new_level.clone();

        // Update level and timestamp
        user_data.level = new_level;
        user_data.level_updated_at = env.ledger().timestamp();

        // Save updated user data
        env.storage()
            .persistent()
            .set(&DataKey::User(user.clone()), &user_data);

        // Publish event for level change
        env.events().publish(
            (Symbol::new(env, "level_changed"), user.clone()),
            ((previous_level, new_level_clone),),
        );

        Ok(())
    }
}
