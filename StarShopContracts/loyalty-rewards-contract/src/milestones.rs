use crate::points::PointsManager;
use crate::types::{DataKey, Error, Milestone, MilestoneRequirement, TransactionType, UserData};
use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};

pub struct MilestoneManager;

impl MilestoneManager {
    /// Create a new milestone
    pub fn create_milestone(env: &Env, milestone: Milestone) -> Result<(), Error> {
        // Check if admin
        crate::admin::AdminModule::verify_admin(env)?;

        // Store the milestone
        env.storage()
            .instance()
            .set(&DataKey::Milestone(milestone.id), &milestone);

        Ok(())
    }

    /// Get a milestone by ID
    pub fn get_milestone(env: &Env, id: u32) -> Result<Milestone, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Milestone(id))
            .ok_or(Error::MilestoneNotFound)
    }

    /// Check if user has completed a milestone
    pub fn has_completed_milestone(env: &Env, user: &Address, milestone_id: u32) -> bool {
        if let Ok(user_data) = PointsManager::get_user_data(env, user) {
            user_data.completed_milestones.contains(&milestone_id)
        } else {
            false
        }
    }

    /// Check if user meets milestone requirements
    pub fn check_milestone_eligibility(
        env: &Env,
        user: &Address,
        milestone_id: u32,
    ) -> Result<bool, Error> {
        // Get user data
        let user_data = PointsManager::get_user_data(env, user)?;

        // Get milestone
        let milestone = Self::get_milestone(env, milestone_id)?;

        // Check if already completed
        if Self::has_completed_milestone(env, user, milestone_id) {
            return Err(Error::MilestoneAlreadyCompleted);
        }

        // Check requirements
        let meets_requirement = match milestone.requirement {
            MilestoneRequirement::TotalPurchases(required) => {
                Self::count_purchases(&user_data) >= required
            }
            MilestoneRequirement::SpendAmount(required) => {
                Self::calculate_total_spend(&user_data) >= required
            }
            MilestoneRequirement::PointsEarned(required) => user_data.lifetime_points >= required,
            MilestoneRequirement::SpecificProduct(product_id) => {
                Self::has_purchased_product(&user_data, &product_id)
            }
            MilestoneRequirement::SpecificCategory(category) => {
                Self::has_purchased_category(&user_data, &category)
            }
            MilestoneRequirement::DaysActive(required) => {
                let current_time = env.ledger().timestamp();
                let days_active = (current_time - user_data.join_date) / (24 * 60 * 60);
                days_active >= required
            }
        };

        Ok(meets_requirement)
    }

    /// Complete a milestone and award points
    pub fn complete_milestone(env: &Env, user: &Address, milestone_id: u32) -> Result<i128, Error> {
        // Check eligibility
        if !Self::check_milestone_eligibility(env, user, milestone_id)? {
            return Err(Error::MilestoneNotFound);
        }

        // Get milestone
        let milestone = Self::get_milestone(env, milestone_id)?;

        // Clone the name for later use in the event
        let milestone_name = milestone.name.clone();

        // Award points
        PointsManager::add_points(
            env,
            user,
            milestone.points_reward,
            milestone_name.clone(),
            TransactionType::Bonus,
        )?;

        // Update user's completed milestones
        let mut user_data = PointsManager::get_user_data(env, user)?;
        user_data.completed_milestones.push_back(milestone_id);

        // Save updated user data
        env.storage()
            .persistent()
            .set(&DataKey::User(user.clone()), &user_data);

        // Publish milestone completion event
        env.events().publish(
            (Symbol::new(env, "milestone_completed"), user.clone()),
            ((
                milestone_id,
                milestone_name,
                milestone.points_reward,
                env.ledger().timestamp(),
            ),),
        );

        Ok(milestone.points_reward)
    }

    /// Check all milestones for a user and complete eligible ones
    pub fn check_and_complete_milestones(env: &Env, user: &Address) -> Result<Vec<u32>, Error> {
        let mut completed = Vec::new(env);
        let mut milestone_id = 0;

        // Iterate through all milestones
        while env
            .storage()
            .instance()
            .has(&DataKey::Milestone(milestone_id))
        {
            // Try to complete milestone if eligible
            if !Self::has_completed_milestone(env, user, milestone_id) {
                if let Ok(true) = Self::check_milestone_eligibility(env, user, milestone_id) {
                    if let Ok(_) = Self::complete_milestone(env, user, milestone_id) {
                        completed.push_back(milestone_id);
                    }
                }
            }

            milestone_id += 1;
        }

        Ok(completed)
    }

    /// Helper: Count total purchases from transaction history
    pub fn count_purchases(user_data: &UserData) -> u32 {
        let mut count = 0;

        for transaction in user_data.transactions.iter() {
            if transaction.transaction_type == TransactionType::Earned
                && transaction.description == symbol_short!("purchase")
            {
                count += 1;
            }
        }

        count
    }

    /// Helper: Calculate total spend from transaction history
    /// This is an approximation based on points earned from purchases
    fn calculate_total_spend(user_data: &UserData) -> i128 {
        let mut total = 0;

        for transaction in user_data.transactions.iter() {
            if transaction.transaction_type == TransactionType::Earned
                && transaction.description == symbol_short!("purchase")
            {
                // Assuming points ratio is stored elsewhere, we use a simple approximation here
                // In a real implementation, you might want to store the actual purchase amount
                total += transaction.amount * 100; // Assuming 1 point = 100 currency units
            }
        }

        total
    }

    /// Helper: Check if user has purchased a specific product
    fn has_purchased_product(user_data: &UserData, product_id: &Symbol) -> bool {
        for transaction in user_data.transactions.iter() {
            if transaction.transaction_type == TransactionType::Earned
                && transaction.description == *product_id
            {
                return true;
            }
        }

        false
    }

    /// Helper: Check if user has purchased from a specific category
    fn has_purchased_category(user_data: &UserData, category: &Symbol) -> bool {
        for transaction in user_data.transactions.iter() {
            if transaction.transaction_type == TransactionType::Earned
                && transaction.description == *category
            {
                return true;
            }
        }

        false
    }
}
