use crate::points::PointsManager;
use crate::types::{DataKey, Error, Milestone, MilestoneRequirement, TransactionType, UserData};
use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};

pub struct MilestoneManager;

impl MilestoneManager {
    /// Create a new milestone
    pub fn create_milestone(env: &Env, milestone: Milestone) -> Result<(), Error> {
        crate::admin::AdminModule::verify_admin(env)?;

        let mut total_milestones: u32 = env.storage().instance().get(&DataKey::TotalMilestones).unwrap_or(0);
        
        // Use the counter as the ID
        let new_milestone = Milestone { id: total_milestones, ..milestone };

        env.storage()
            .instance()
            .set(&DataKey::Milestone(new_milestone.id), &new_milestone);
        
        total_milestones += 1;
        env.storage().instance().set(&DataKey::TotalMilestones, &total_milestones);

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
        user_data: &UserData,
        milestone: &Milestone,
    ) -> Result<bool, Error> {
        if user_data.completed_milestones.contains(&milestone.id) {
            return Err(Error::MilestoneAlreadyCompleted);
        }

        let ratio = crate::admin::AdminModule::get_points_ratio(env);

        let meets_requirement = match &milestone.requirement {
            MilestoneRequirement::TotalPurchases(required) => {
                Self::count_purchases(user_data) >= *required
            }
            MilestoneRequirement::SpendAmount(required) => {
                Self::calculate_total_spend(user_data, ratio) >= *required
            }
            MilestoneRequirement::PointsEarned(required) => user_data.lifetime_points >= *required,
            MilestoneRequirement::SpecificProduct(product_id) => {
                Self::has_purchased_product(user_data, product_id)
            }
            MilestoneRequirement::SpecificCategory(category) => {
                Self::has_purchased_category(user_data, category)
            }
            MilestoneRequirement::DaysActive(required) => {
                let days_active = (env.ledger().timestamp() - user_data.join_date) / (24 * 60 * 60);
                days_active >= *required
            }
        };

        Ok(meets_requirement)
    }

    /// Complete a milestone and award points
    pub fn complete_milestone(env: &Env, user: &Address, milestone_id: u32) -> Result<i128, Error> {
        let user_data = PointsManager::get_user_data(env, user)?;
        let milestone = Self::get_milestone(env, milestone_id)?;

        if !Self::check_milestone_eligibility(env, &user_data, &milestone)? {
            return Err(Error::MilestoneNotEligible);
        }

        let milestone_name = milestone.name.clone();
        let points_reward = milestone.points_reward;
        
        PointsManager::add_points(
            env,
            user,
            points_reward,
            milestone_name.clone(),
            TransactionType::Bonus,
            None,
            None
        )?;

        // Re-fetch user_data after it was modified by add_points
        let mut updated_user_data = PointsManager::get_user_data(env, user)?;
        
        updated_user_data.completed_milestones.push_back(milestone_id);
        env.storage()
            .persistent()
            .set(&DataKey::User(user.clone()), &updated_user_data);

        env.events().publish(
            (Symbol::new(env, "milestone_completed"), user.clone()),
            ((milestone_id, milestone_name, points_reward, env.ledger().timestamp()),),
        );

        Ok(points_reward)
    }

    /// Check all milestones for a user and complete eligible ones
    pub fn check_and_complete_milestones(env: &Env, user: &Address) -> Result<Vec<u32>, Error> {
        let mut completed = Vec::new(env);
        let total_milestones: u32 = env.storage().instance().get(&DataKey::TotalMilestones).unwrap_or(0);

        for milestone_id in 0..total_milestones {
            let user_data = PointsManager::get_user_data(env, user)?;
            if let Ok(milestone) = Self::get_milestone(env, milestone_id) {
                if !user_data.completed_milestones.contains(&milestone_id) {
                    if let Ok(true) = Self::check_milestone_eligibility(env, &user_data, &milestone) {
                         if Self::complete_milestone(env, user, milestone_id).is_ok() {
                            completed.push_back(milestone_id);
                        }
                    }
                }
            }
        }
        Ok(completed)
    }

    // Helper Functions

    pub fn count_purchases(user_data: &UserData) -> u32 {
        user_data.transactions.iter().filter(|tx| 
            tx.transaction_type == TransactionType::Earned && tx.description == symbol_short!("purchase")
        ).count() as u32
    }

    fn calculate_total_spend(user_data: &UserData, points_ratio: u32) -> i128 {
        let mut total_points_from_purchase = 0;
        for tx in user_data.transactions.iter() {
            if tx.transaction_type == TransactionType::Earned && tx.description == symbol_short!("purchase") {
                total_points_from_purchase += tx.amount;
            }
        }
        // Approximate spend based on points earned and the base ratio
        total_points_from_purchase * (points_ratio as i128)
    }

    fn has_purchased_product(user_data: &UserData, product_id: &Symbol) -> bool {
        user_data.transactions.iter().any(|tx| {
            tx.transaction_type == TransactionType::Earned && tx.product_id.as_ref() == Some(product_id)
        })
    }

    fn has_purchased_category(user_data: &UserData, category: &Symbol) -> bool {
        user_data.transactions.iter().any(|tx| {
            tx.transaction_type == TransactionType::Earned && tx.category.as_ref() == Some(category)
        })
    }
}
