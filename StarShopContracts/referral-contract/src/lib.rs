#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

use admin::*;
use referral::*;
use rewards::*;
use types::*;
use verification::*;

mod admin;
mod level;
mod referral;
mod rewards;
mod types;
mod verification;

#[contract]
pub struct ReferralContract;

#[contractimpl]
impl ReferralContract {
    /// Initializes the referral contract with an admin address, reward token, and default level requirements
    ///
    /// # Arguments
    /// * `admin` - The address of the contract administrator
    /// * `reward_token` - The address of the token used for rewards
    pub fn initialize(env: Env, admin: Address, reward_token: Address) -> Result<(), Error> {
        let default_requirements = LevelRequirements {
            silver: LevelCriteria {
                required_direct_referrals: 5,
                required_team_size: 15,
                required_total_rewards: 1000,
            },
            gold: LevelCriteria {
                required_direct_referrals: 10,
                required_team_size: 50,
                required_total_rewards: 5000,
            },
            platinum: LevelCriteria {
                required_direct_referrals: 20,
                required_team_size: 100,
                required_total_rewards: 20000,
            },
        };
        ReferralModule::initialize(&env, &admin)?;
        AdminModule::initialize(env, admin, reward_token, default_requirements)
    }

    /// Sets the reward rates for different referral levels
    ///
    /// # Arguments
    /// * `rates` - The new reward rates structure
    pub fn set_reward_rates(env: Env, rates: RewardRates) -> Result<(), Error> {
        AdminModule::set_reward_rates(env, rates)
    }

    /// get admin address
    pub fn get_admin(env: Env) -> Result<Address, Error> {
        AdminModule::get_admin(env)
    }

    /// Adds a new milestone to the referral program
    ///
    /// # Arguments
    /// * `milestone` - The milestone to be added
    pub fn add_milestone(env: Env, milestone: Milestone) -> Result<(), Error> {
        AdminModule::add_milestone(env, milestone)
    }

    /// Removes a milestone from the referral program
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone to remove
    pub fn remove_milestone(env: Env, milestone_id: u32) -> Result<(), Error> {
        AdminModule::remove_milestone(env, milestone_id)
    }

    /// Updates an existing milestone
    ///
    /// # Arguments
    /// * `milestone_id` - The ID of the milestone to update
    /// * `milestone` - The new milestone data
    pub fn update_milestone(
        env: Env,
        milestone_id: u32,
        milestone: Milestone,
    ) -> Result<(), Error> {
        AdminModule::update_milestone(env, milestone_id, milestone)
    }

    /// Pauses all contract operations
    pub fn pause_contract(env: Env) -> Result<(), Error> {
        AdminModule::pause_contract(env)
    }

    /// Resumes contract operations after being paused
    pub fn resume_contract(env: Env) -> Result<(), Error> {
        AdminModule::resume_contract(env)
    }

    /// Check if contract is paused
    pub fn get_paused_state(env: Env) -> Result<bool, Error> {
        AdminModule::get_paused_state(env)
    }

    /// Transfers admin rights to a new address
    ///
    /// # Arguments
    /// * `new_admin` - The address of the new administrator
    pub fn transfer_admin(env: Env, new_admin: Address) -> Result<(), Error> {
        AdminModule::transfer_admin(env, new_admin)
    }

    /// Sets or updates the reward token address
    ///
    /// # Arguments
    /// * `token` - The address of the new reward token
    pub fn set_reward_token(env: Env, token: Address) -> Result<(), Error> {
        AdminModule::set_reward_token(env, token)
    }

    /// Updates the requirements for different referral levels
    ///
    /// # Arguments
    /// * `requirements` - The new level requirements
    pub fn set_level_requirements(env: Env, requirements: LevelRequirements) -> Result<(), Error> {
        AdminModule::set_level_requirements(env, requirements)
    }

    /// Submits a verification request for a user
    ///
    /// # Arguments
    /// * `user` - The address of the user to verify
    /// * `identity_proof` - Proof of identity for verification
    pub fn submit_verification(
        env: Env,
        user: Address,
        identity_proof: String,
    ) -> Result<(), Error> {
        VerificationModule::submit_verification(env, user, identity_proof)
    }

    /// Approves a user's verification request
    ///
    /// # Arguments
    /// * `user` - The address of the user to approve
    pub fn approve_verification(env: Env, user: Address) -> Result<(), Error> {
        VerificationModule::approve_verification(env, user)
    }

    /// Rejects a user's verification request with a reason
    ///
    /// # Arguments
    /// * `user` - The address of the user to reject
    /// * `reason` - The reason for rejection
    pub fn reject_verification(env: Env, user: Address, reason: String) -> Result<(), Error> {
        VerificationModule::reject_verification(env, user, reason)
    }

    /// Retrieves the verification status of a user
    ///
    /// # Arguments
    /// * `user` - The address of the user to check
    pub fn get_verification_status(env: Env, user: Address) -> Result<VerificationStatus, Error> {
        VerificationModule::get_verification_status(env, user)
    }

    /// Returns a list of all pending verification requests
    pub fn get_pending_verifications(env: Env) -> Result<Vec<Address>, Error> {
        VerificationModule::get_pending_verifications(env)
    }

    /// Registers a new user with a referrer
    ///
    /// # Arguments
    /// * `user` - The address of the new user
    /// * `referrer_address` - The address of the referrer
    /// * `identity_proof` - Proof of identity for verification
    pub fn register_with_referral(
        env: Env,
        user: Address,
        referrer_address: Address,
        identity_proof: String,
    ) -> Result<(), Error> {
        ReferralModule::register_with_referral(env, user, referrer_address, identity_proof)
    }

    /// Checks if a user is verified
    ///
    /// # Arguments
    /// * `user` - The address of the user to check
    pub fn is_user_verified(env: Env, user: Address) -> Result<bool, Error> {
        ReferralModule::is_user_verified(env, user)
    }

    /// Checks if a user is registered in the system
    ///
    /// # Arguments
    /// * `user` - The address of the user to check
    pub fn is_user_registered(env: Env, user: Address) -> Result<bool, Error> {
        ReferralModule::is_user_registered(env, user)
    }

    /// Retrieves detailed information about a user
    ///
    /// # Arguments
    /// * `user` - The address of the user
    pub fn get_user_info(env: Env, user: Address) -> Result<UserData, Error> {
        ReferralModule::get_user_info(env, user)
    }

    /// Gets a list of direct referrals for a user
    ///
    /// # Arguments
    /// * `user` - The address of the user
    pub fn get_direct_referrals(env: Env, user: Address) -> Result<Vec<Address>, Error> {
        ReferralModule::get_direct_referrals(env, user)
    }

    /// Gets the total team size (direct and indirect referrals) for a user
    ///
    /// # Arguments
    /// * `user` - The address of the user
    pub fn get_team_size(env: Env, user: Address) -> Result<u32, Error> {
        ReferralModule::get_team_size(env, user)
    }

    /// Distributes rewards to a user and their upline
    ///
    /// # Arguments
    /// * `user` - The address of the user
    /// * `amount` - The amount of rewards to distribute
    pub fn distribute_rewards(env: Env, user: Address, amount: i128) -> Result<(), Error> {
        RewardModule::distribute_rewards(env, user, amount)
    }

    /// Allows a user to claim their accumulated rewards
    ///
    /// # Arguments
    /// * `user` - The address of the user claiming rewards
    pub fn claim_rewards(env: Env, user: Address) -> Result<i128, Error> {
        RewardModule::claim_rewards(env, user)
    }

    /// Gets the amount of pending rewards for a user
    ///
    /// # Arguments
    /// * `user` - The address of the user
    pub fn get_pending_rewards(env: Env, user: Address) -> Result<i128, Error> {
        RewardModule::get_pending_rewards(env, user)
    }

    /// Gets the total rewards earned by a user
    ///
    /// # Arguments
    /// * `user` - The address of the user
    pub fn get_total_rewards(env: Env, user: Address) -> Result<i128, Error> {
        RewardModule::get_total_rewards(env, user)
    }

    /// Checks and rewards any achieved milestones for a user
    ///
    /// # Arguments
    /// * `user` - The address of the user
    pub fn check_and_reward_milestone(env: Env, user: Address) -> Result<(), Error> {
        RewardModule::check_and_reward_milestone(env, user)
    }

    /// Gets the total number of users in the system
    pub fn get_total_users(env: Env) -> Result<u32, Error> {
        ReferralModule::get_total_users(env)
    }

    /// Gets the total amount of rewards distributed
    pub fn get_total_distributed_rewards(env: Env) -> Result<i128, Error> {
        RewardModule::get_total_distributed_rewards(env)
    }

    /// Gets various system metrics as key-value pairs
    /// total_users, total_rewards, average_reward_per_user
    ///
    pub fn get_system_metrics(env: Env) -> Result<Vec<(String, i128)>, Error> {
        ReferralModule::get_system_metrics(env)
    }

    /// Gets the referral conversion rate for a user. verified users/registered users
    ///
    /// # Arguments
    /// * `user` - The address of the user
    pub fn get_referral_conversion_rate(env: Env, user: Address) -> Result<u32, Error> {
        ReferralModule::get_referral_conversion_rate(env, user)
    }

    /// Gets the level of a user
    ///
    /// # Arguments
    /// * `user` - The address of the user
    pub fn get_user_level(env: Env, user: Address) -> Result<UserLevel, Error> {
        ReferralModule::get_user_level(env, user)
    }
}

#[cfg(test)]
mod test;
mod break_test;
