use crate::points::PointsManager;
use crate::types::{DataKey, Error, Reward, RewardType};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{Address, Env, Symbol, Vec};

pub struct RewardManager;

impl RewardManager {
    /// Create a new reward
    pub fn create_reward(env: &Env, reward: Reward) -> Result<(), Error> {
        // Check if admin
        crate::admin::AdminModule::verify_admin(env)?;

        // Store the reward
        env.storage()
            .instance()
            .set(&DataKey::Reward(reward.id), &reward);

        Ok(())
    }

    /// Get a reward by ID
    pub fn get_reward(env: &Env, id: u32) -> Result<Reward, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Reward(id))
            .ok_or(Error::RewardNotFound)
    }

    /// Check if user is eligible for a reward
    pub fn check_reward_eligibility(
        env: &Env,
        user: &Address,
        reward_id: u32,
    ) -> Result<bool, Error> {
        let user_data = PointsManager::get_user_data(env, user)?;
        let reward = Self::get_reward(env, reward_id)?;

        // Check if user has enough points
        if user_data.current_points < reward.points_cost {
            return Ok(false);
        }

        // Check if user has required loyalty level
        if user_data.level < reward.min_level {
            return Ok(false);
        }

        Ok(true)
    }

    /// Redeem a reward
    pub fn redeem_reward(
        env: &Env,
        user: &Address,
        reward_id: u32,
        purchase_amount: Option<i128>,
    ) -> Result<(), Error> {
        // Authenticate user
        user.require_auth();

        // Check eligibility
        if !Self::check_reward_eligibility(env, user, reward_id)? {
            return Err(Error::InsufficientPoints);
        }

        let reward = Self::get_reward(env, reward_id)?;

        // For discount rewards, ensure purchase amount is provided
        if let RewardType::Discount(_) = reward.reward_type {
            if purchase_amount.is_none() {
                return Err(Error::InvalidAmount);
            }
        }

        // Handle maximum redemption percentage for discounts
        let mut discount_value: Option<i128> = None;
        if let RewardType::Discount(discount_bps) = reward.reward_type {
            if let Some(amount) = purchase_amount {
                let max_redemption_bps = env
                    .storage()
                    .instance()
                    .get::<_, u32>(&DataKey::MaxRedemptionPercentage)
                    .unwrap_or(5000); // Default to 50% if not set

                // Calculate discount value
                let calc_discount = (amount * discount_bps as i128) / 10000;

                // Check if discount exceeds maximum allowed
                let max_allowed_discount = (amount * max_redemption_bps as i128) / 10000;

                if calc_discount > max_allowed_discount {
                    return Err(Error::MaxRedemptionExceeded);
                }

                discount_value = Some(calc_discount);
            }
        }

        // Create reward claim data for event
        let claim_data = (
            reward.id,
            reward.name.clone(),
            reward.points_cost,
            env.ledger().timestamp(),
        );

        // Process the reward based on type
        match reward.reward_type {
            RewardType::Discount(_) => {
                // Discount is handled at point of sale, just deduct points
                PointsManager::spend_points(
                    env,
                    user,
                    reward.points_cost,
                    Symbol::new(env, "discount_reward"),
                )?;

                // Publish discount reward event with discount value
                env.events().publish(
                    (Symbol::new(env, "reward_claimed"), user.clone()),
                    ((claim_data, "discount", discount_value),),
                );
            }
            RewardType::Product(product_id) => {
                // Product reward, deduct points
                PointsManager::spend_points(
                    env,
                    user,
                    reward.points_cost,
                    Symbol::new(env, "product_reward"),
                )?;

                // Publish product reward event
                env.events().publish(
                    (Symbol::new(env, "reward_claimed"), user.clone()),
                    ((claim_data, "product", product_id),),
                );

                // In a real implementation, you would integrate with inventory system
                // to mark the product as redeemed for the user
            }
            RewardType::XLM(amount) => {
                // XLM reward, deduct points and transfer XLM
                PointsManager::spend_points(
                    env,
                    user,
                    reward.points_cost,
                    Symbol::new(env, "xlm_reward"),
                )?;

                // Publish XLM reward event
                env.events().publish(
                    (Symbol::new(env, "reward_claimed"), user.clone()),
                    ((claim_data, "xlm", amount),),
                );

                // Transfer XLM to user
                // This would require integration with Stellar's native asset
                // In a real implementation, you would use the Stellar SDK
            }
            RewardType::Token(token_address, amount) => {
                // Token reward, deduct points and transfer tokens
                PointsManager::spend_points(
                    env,
                    user,
                    reward.points_cost,
                    Symbol::new(env, "token_reward"),
                )?;

                // Transfer tokens to user
                let token = TokenClient::new(env, &token_address);
                token.transfer(&env.current_contract_address(), user, &amount);

                // Publish token reward event
                env.events().publish(
                    (Symbol::new(env, "reward_claimed"), user.clone()),
                    ((claim_data, "token", (token_address, amount)),),
                );
            }
        }

        Ok(())
    }

    /// Calculate discount amount for a purchase
    pub fn calculate_discount(
        env: &Env,
        _user: &Address,
        reward_id: u32,
        purchase_amount: i128,
    ) -> Result<i128, Error> {
        let reward = Self::get_reward(env, reward_id)?;

        // Ensure reward is a discount type
        if let RewardType::Discount(discount_bps) = reward.reward_type {
            // Calculate discount value
            let discount_value = (purchase_amount * discount_bps as i128) / 10000;

            // Check maximum redemption percentage
            let max_redemption_bps = env
                .storage()
                .instance()
                .get::<_, u32>(&DataKey::MaxRedemptionPercentage)
                .unwrap_or(5000); // Default to 50% if not set

            let max_allowed_discount = (purchase_amount * max_redemption_bps as i128) / 10000;

            // Return the lower of the two values
            if discount_value > max_allowed_discount {
                Ok(max_allowed_discount)
            } else {
                Ok(discount_value)
            }
        } else {
            Err(Error::RewardNotFound)
        }
    }

    /// Get available rewards for a user based on their level
    pub fn get_available_rewards(env: &Env, user: &Address) -> Result<Vec<Reward>, Error> {
        let user_data = PointsManager::get_user_data(env, user)?;
        let mut rewards = Vec::new(env);
        let mut reward_id = 0;

        // Iterate through all rewards
        while env.storage().instance().has(&DataKey::Reward(reward_id)) {
            let reward = Self::get_reward(env, reward_id)?;

            // Include reward if user level is sufficient
            if user_data.level >= reward.min_level {
                rewards.push_back(reward);
            }

            reward_id += 1;
        }

        Ok(rewards)
    }
}
