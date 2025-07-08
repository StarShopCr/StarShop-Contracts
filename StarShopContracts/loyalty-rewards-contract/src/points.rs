use crate::types::{DataKey, Error, PointsTransaction, TransactionType, UserData};
use soroban_sdk::{symbol_short, Address, Env, Symbol};

pub struct PointsManager;

impl PointsManager {
    /// Initialize a new user in the system
    pub fn register_user(env: &Env, user: &Address) -> Result<(), Error> {
        if Self::user_exists(env, user) {
            return Err(Error::AlreadyInitialized); // User already exists
        }

        let user_data = UserData {
            address: user.clone(),
            level: crate::types::LoyaltyLevel::Bronze,
            level_updated_at: env.ledger().timestamp(),
            transactions: soroban_sdk::vec![env],
            completed_milestones: soroban_sdk::vec![env],
            join_date: env.ledger().timestamp(),
            last_anniversary_awarded: env.ledger().timestamp(),
            lifetime_points: 0
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
        product_id: Option<Symbol>,
        category: Option<Symbol>,
    ) -> Result<(), Error> {
        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let mut user_data = Self::get_user_data(env, user)?;

        let expiry_days = crate::admin::AdminModule::get_points_expiry(env);
        let current_time = env.ledger().timestamp();
        let expiration = current_time + (expiry_days * 24 * 60 * 60);

        let transaction = PointsTransaction {
            amount,
            transaction_type: transaction_type.clone(),
            timestamp: current_time,
            description,
            expiration,
            product_id,
            category,
        };

        user_data.transactions.push_back(transaction.clone());
        
        if transaction_type == TransactionType::Earned || transaction_type == TransactionType::Bonus {
            user_data.lifetime_points += amount;
        }

        env.storage()
            .persistent()
            .set(&DataKey::User(user.clone()), &user_data);

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
        let balance = Self::get_points_balance_internal(env, &user_data)?;
        
        if balance < amount {
            return Err(Error::InsufficientPoints);
        }

        let transaction = PointsTransaction {
            amount: -amount, // Negative for spending
            transaction_type: TransactionType::Spent,
            timestamp: env.ledger().timestamp(),
            description,
            expiration: 0, 
            product_id: None,
            category: None,
        };

        user_data.transactions.push_back(transaction.clone());

        env.storage()
            .persistent()
            .set(&DataKey::User(user.clone()), &user_data);

        env.events().publish(
            (Symbol::new(env, "points_spent"), user.clone()),
            (transaction,),
        );

        Ok(())
    }
    
    /// Internal function to calculate balance from transaction history
    fn get_points_balance_internal(env: &Env, user_data: &UserData) -> Result<i128, Error> {
        let current_time = env.ledger().timestamp();
        let mut balance = 0i128;

        for transaction in user_data.transactions.iter() {
            match transaction.transaction_type {
                TransactionType::Earned | TransactionType::Bonus => {
                    // Add points only if they have not expired
                    if transaction.expiration > current_time {
                        balance += transaction.amount;
                    }
                }
                TransactionType::Spent => {
                    // Spent points are represented as negative amounts
                    balance += transaction.amount; 
                }
                TransactionType::Expired => {
                    // Expired transactions are markers and don't affect live calculation
                }
            }
        }
        Ok(balance)
    }

    /// Get user's current, valid points balance
    pub fn get_points_balance(env: &Env, user: &Address) -> Result<i128, Error> {
        let user_data = Self::get_user_data(env, user)?;
        Self::get_points_balance_internal(env, &user_data)
    }

    /// Get user's lifetime points (total ever earned)
    pub fn get_lifetime_points(env: &Env, user: &Address) -> Result<i128, Error> {
        let user_data = Self::get_user_data(env, user)?;
        Ok(user_data.lifetime_points)
    }

    /// Calculate points for a purchase, including bonuses
    pub fn calculate_purchase_points(
        env: &Env,
        purchase_amount: i128,
        product_id: &Option<Symbol>,
        category: &Option<Symbol>,
    ) -> i128 {
        let base_ratio = crate::admin::AdminModule::get_points_ratio(env);
        if base_ratio == 0 { return 0; }

        let base_points = purchase_amount / (base_ratio as i128);
        let mut total_points = base_points;

        if let Some(cat) = category {
            if let Some(bonus_bps) = env.storage().instance().get::<_, u32>(&DataKey::ProductCategoryBonus(cat.clone())) {
                total_points += (base_points * bonus_bps as i128) / 10000;
            }
        }

        if let Some(prod) = product_id {
            if let Some(bonus_bps) = env.storage().instance().get::<_, u32>(&DataKey::ProductBonus(prod.clone())) {
                total_points += (base_points * bonus_bps as i128) / 10000;
            }
        }

        total_points
    }

    /// Record points from a purchase transaction
    pub fn record_purchase_points(
        env: &Env,
        user: &Address,
        purchase_amount: i128,
        product_id: Option<Symbol>,
        category: Option<Symbol>,
    ) -> Result<i128, Error> {
        if !Self::user_exists(env, user) {
            Self::register_user(env, user)?;
        }

        let points_to_add = Self::calculate_purchase_points(env, purchase_amount, &product_id, &category);

        if points_to_add > 0 {
            Self::add_points(
                env,
                user,
                points_to_add,
                symbol_short!("purchase"),
                TransactionType::Earned,
                product_id,
                category,
            )?;
        }

        Ok(points_to_add)
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
}
