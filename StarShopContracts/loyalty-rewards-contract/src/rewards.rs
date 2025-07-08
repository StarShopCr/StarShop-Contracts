use crate::points::PointsManager;
use crate::types::{DataKey, Error, Reward, RewardType, UserRedemption};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Symbol, Vec};

pub struct RewardManager;

impl RewardManager {
    /// Create a new reward, managed by a central counter
    pub fn create_reward(env: &Env, reward: Reward) -> Result<(), Error> {
        crate::admin::AdminModule::verify_admin(env)?;

        let mut total_rewards: u32 = env.storage().instance().get(&DataKey::TotalRewards).unwrap_or(0);
        let new_reward = Reward { id: total_rewards, ..reward };

        env.storage()
            .instance()
            .set(&DataKey::Reward(new_reward.id), &new_reward);

        total_rewards += 1;
        env.storage().instance().set(&DataKey::TotalRewards, &total_rewards);

        Ok(())
    }

    /// Get a reward by ID
    pub fn get_reward(env: &Env, id: u32) -> Result<Reward, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Reward(id))
            .ok_or(Error::RewardNotFound)
    }

    /// Check if a user is eligible for a reward
    pub fn check_reward_eligibility(
        env: &Env,
        user: &Address,
        reward_id: u32,
    ) -> Result<bool, Error> {
        let user_data = PointsManager::get_user_data(env, user)?;
        let reward = Self::get_reward(env, reward_id)?;

        let balance = PointsManager::get_points_balance(env, user)?;
        if balance < reward.points_cost {
            return Ok(false);
        }

        if user_data.level < reward.min_level {
            return Ok(false);
        }

        // Check redemption limit
        let redemption_key = UserRedemption(user.clone(), reward_id);
        let user_redemptions: u32 = env.storage().persistent().get(&redemption_key).unwrap_or(0);
        
        if reward.max_per_user > 0 && user_redemptions >= reward.max_per_user {
            return Err(Error::RewardLimitReached);
        }
        
        Ok(true)
    }

    /// Redeem a reward, returning the value of the reward (e.g., discount amount)
    pub fn redeem_reward(
        env: &Env,
        user: &Address,
        reward_id: u32,
        purchase_amount: Option<i128>,
    ) -> Result<i128, Error> {
        user.require_auth();

        if !Self::check_reward_eligibility(env, user, reward_id)? {
            return Err(Error::InsufficientPoints); // Or other error from check
        }

        let reward = Self::get_reward(env, reward_id)?;
        let mut reward_value = 0i128;

        // Process the reward based on its type
        match reward.reward_type.clone() {
            RewardType::Discount(discount_bps) => {
                let amount = purchase_amount.ok_or(Error::InvalidAmount)?;
                let discount = Self::calculate_discount_internal(env, amount, discount_bps)?;
                reward_value = discount;
            }
            RewardType::Product(_) => {
                reward_value = 1; // Represents 1 product
            }
            RewardType::XLM(amount) | RewardType::Token(_, amount) => {
                reward_value = amount;
            }
        }
        
        // Deduct points
        PointsManager::spend_points(env, user, reward.points_cost, reward.name.clone())?;

        // Update redemption count for the user
        let redemption_key = UserRedemption(user.clone(), reward_id);
        let mut user_redemptions: u32 = env.storage().persistent().get(&redemption_key).unwrap_or(0);
        user_redemptions += 1;
        env.storage().persistent().set(&redemption_key, &user_redemptions);


        // Handle token/XLM transfers
        match reward.reward_type {
            RewardType::Token(token_address, amount) => {
                let token = TokenClient::new(env, &token_address);
                token.transfer(&env.current_contract_address(), user, &amount);
            }
            RewardType::XLM(_) => {
                // NOTE: Native XLM transfers are not directly supported in vanilla Soroban contracts.
                // This requires a separate contract call to an XLM wrapper or a host function invocation.
                // This part of the logic is a placeholder for that integration.
            }
            _ => {} // Discount and Product rewards are handled off-chain
        }
        
        env.events().publish(
            (Symbol::new(env, "reward_claimed"), user.clone()),
            ((reward.id, reward.name, reward.points_cost, env.ledger().timestamp()),),
        );
        
        Ok(reward_value)
    }

    fn calculate_discount_internal(env: &Env, purchase_amount: i128, discount_bps: u32) -> Result<i128, Error> {
        let max_redemption_bps = crate::admin::AdminModule::get_max_redemption_percentage(env);
        let discount_value = (purchase_amount * discount_bps as i128) / 10000;
        let max_allowed_discount = (purchase_amount * max_redemption_bps as i128) / 10000;

        Ok(discount_value.min(max_allowed_discount))
    }

    /// Calculate discount amount for a purchase
    pub fn calculate_discount(
        env: &Env,
        reward_id: u32,
        purchase_amount: i128,
    ) -> Result<i128, Error> {
        let reward = Self::get_reward(env, reward_id)?;

        if let RewardType::Discount(discount_bps) = reward.reward_type {
            Self::calculate_discount_internal(env, purchase_amount, discount_bps)
        } else {
            Err(Error::RewardNotFound) // Not a discount reward
        }
    }

    /// Get all available rewards in the system
    pub fn get_available_rewards(env: &Env) -> Result<Vec<Reward>, Error> {
        let mut rewards = Vec::new(env);
        let total_rewards: u32 = env.storage().instance().get(&DataKey::TotalRewards).unwrap_or(0);

        for reward_id in 0..total_rewards {
            if let Ok(reward) = Self::get_reward(env, reward_id) {
                rewards.push_back(reward);
            }
        }

        Ok(rewards)
    }
}
