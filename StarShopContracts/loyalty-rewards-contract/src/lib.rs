#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec};

pub mod admin;
pub mod levels;
pub mod milestones;
pub mod points;
pub mod rewards;
mod test;
pub mod types;

use admin::AdminModule;
use levels::LevelManager;
use milestones::MilestoneManager;
use points::PointsManager;
use rewards::RewardManager;
use types::{Error, LevelRequirements, LoyaltyLevel, Milestone, Reward, TransactionType};

/// Main contract trait defining all available functions
pub trait LoyaltyRewardsTrait {
    // Admin functions
    fn init(env: Env, admin: Address) -> Result<(), Error>;
    fn set_points_expiry(env: Env, days: u64) -> Result<(), Error>;
    fn set_max_redemption_percentage(env: Env, percentage_bps: u32) -> Result<(), Error>;
    fn set_points_ratio(env: Env, ratio: u32) -> Result<(), Error>;
    fn set_category_bonus(env: Env, category: Symbol, bonus_bps: u32) -> Result<(), Error>;
    fn set_product_bonus(env: Env, product_id: Symbol, bonus_bps: u32) -> Result<(), Error>;

    // Points management
    fn register_user(env: Env, user: Address) -> Result<(), Error>;
    fn add_points(env: Env, user: Address, amount: i128, description: Symbol) -> Result<(), Error>;
    fn get_points_balance(env: Env, user: Address) -> Result<i128, Error>;
    fn get_lifetime_points(env: Env, user: Address) -> Result<i128, Error>;
    fn record_purchase_points(
        env: Env,
        user: Address,
        purchase_amount: i128,
        product_id: Option<Symbol>,
        category: Option<Symbol>,
    ) -> Result<i128, Error>;

    // Levels management
    fn init_level_requirements(env: Env, requirements: LevelRequirements) -> Result<(), Error>;
    fn check_and_update_level(env: Env, user: Address) -> Result<bool, Error>;
    fn get_user_level(env: Env, user: Address) -> Result<LoyaltyLevel, Error>;
    fn award_anniversary_bonus(env: Env, user: Address) -> Result<i128, Error>;

    // Milestones management
    fn create_milestone(env: Env, milestone: Milestone) -> Result<(), Error>;
    fn complete_milestone(env: Env, user: Address, milestone_id: u32) -> Result<i128, Error>;
    fn check_and_complete_milestones(env: Env, user: Address) -> Result<Vec<u32>, Error>;

    // Rewards management
    fn create_reward(env: Env, reward: Reward) -> Result<(), Error>;
    fn redeem_reward(
        env: Env,
        user: Address,
        reward_id: u32,
        purchase_amount: Option<i128>,
    ) -> Result<(), Error>;
    fn get_available_rewards(env: Env, user: Address) -> Result<Vec<Reward>, Error>;
    fn calculate_discount(
        env: Env,
        user: Address,
        reward_id: u32,
        purchase_amount: i128,
    ) -> Result<i128, Error>;
}

#[contract]
pub struct LoyaltyRewards;

#[contractimpl]
impl LoyaltyRewardsTrait for LoyaltyRewards {
    // Admin functions
    fn init(env: Env, admin: Address) -> Result<(), Error> {
        AdminModule::init(&env, &admin)
    }

    fn set_points_expiry(env: Env, days: u64) -> Result<(), Error> {
        AdminModule::set_points_expiry(&env, days)
    }

    fn set_max_redemption_percentage(env: Env, percentage_bps: u32) -> Result<(), Error> {
        AdminModule::set_max_redemption_percentage(&env, percentage_bps)
    }

    fn set_points_ratio(env: Env, ratio: u32) -> Result<(), Error> {
        AdminModule::set_points_ratio(&env, ratio)
    }

    fn set_category_bonus(env: Env, category: Symbol, bonus_bps: u32) -> Result<(), Error> {
        AdminModule::set_category_bonus(&env, &category, bonus_bps)
    }

    fn set_product_bonus(env: Env, product_id: Symbol, bonus_bps: u32) -> Result<(), Error> {
        AdminModule::set_product_bonus(&env, &product_id, bonus_bps)
    }

    // Points management
    fn register_user(env: Env, user: Address) -> Result<(), Error> {
        PointsManager::register_user(&env, &user)
    }

    fn add_points(env: Env, user: Address, amount: i128, description: Symbol) -> Result<(), Error> {
        PointsManager::add_points(&env, &user, amount, description, TransactionType::Earned)
    }

    fn get_points_balance(env: Env, user: Address) -> Result<i128, Error> {
        PointsManager::get_points_balance(&env, &user)
    }

    fn get_lifetime_points(env: Env, user: Address) -> Result<i128, Error> {
        PointsManager::get_lifetime_points(&env, &user)
    }

    fn record_purchase_points(
        env: Env,
        user: Address,
        purchase_amount: i128,
        product_id: Option<Symbol>,
        category: Option<Symbol>,
    ) -> Result<i128, Error> {
        PointsManager::record_purchase_points(&env, &user, purchase_amount, product_id, category)
    }

    // Levels management
    fn init_level_requirements(env: Env, requirements: LevelRequirements) -> Result<(), Error> {
        LevelManager::init_level_requirements(&env, requirements)
    }

    fn check_and_update_level(env: Env, user: Address) -> Result<bool, Error> {
        LevelManager::check_and_update_level(&env, &user)
    }

    fn get_user_level(env: Env, user: Address) -> Result<LoyaltyLevel, Error> {
        LevelManager::get_user_level(&env, &user)
    }

    fn award_anniversary_bonus(env: Env, user: Address) -> Result<i128, Error> {
        LevelManager::award_anniversary_bonus(&env, &user)
    }

    // Milestones management
    fn create_milestone(env: Env, milestone: Milestone) -> Result<(), Error> {
        MilestoneManager::create_milestone(&env, milestone)
    }

    fn complete_milestone(env: Env, user: Address, milestone_id: u32) -> Result<i128, Error> {
        MilestoneManager::complete_milestone(&env, &user, milestone_id)
    }

    fn check_and_complete_milestones(env: Env, user: Address) -> Result<Vec<u32>, Error> {
        MilestoneManager::check_and_complete_milestones(&env, &user)
    }

    // Rewards management
    fn create_reward(env: Env, reward: Reward) -> Result<(), Error> {
        RewardManager::create_reward(&env, reward)
    }

    fn redeem_reward(
        env: Env,
        user: Address,
        reward_id: u32,
        purchase_amount: Option<i128>,
    ) -> Result<(), Error> {
        RewardManager::redeem_reward(&env, &user, reward_id, purchase_amount)
    }

    fn get_available_rewards(env: Env, user: Address) -> Result<Vec<Reward>, Error> {
        RewardManager::get_available_rewards(&env, &user)
    }

    fn calculate_discount(
        env: Env,
        user: Address,
        reward_id: u32,
        purchase_amount: i128,
    ) -> Result<i128, Error> {
        RewardManager::calculate_discount(&env, &user, reward_id, purchase_amount)
    }
}
