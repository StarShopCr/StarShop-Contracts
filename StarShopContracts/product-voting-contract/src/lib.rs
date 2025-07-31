#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec};

pub mod limits;
pub mod ranking;
mod test;
pub mod types;
pub mod vote;
use limits::VoteLimiter;
use ranking::RankingCalculator;
use types::{AdminConfig, Error, VoteType};
use vote::VoteManager;

pub trait ProductVotingTrait {
    // Core Functions
    fn init(env: Env);
    fn create_product(env: Env, id: Symbol, name: Symbol, creator: Address) -> Result<(), Error>;
    fn cast_vote(
        env: Env,
        product_id: Symbol,
        vote_type: VoteType,
        voter: Address,
    ) -> Result<(), Error>;
    fn get_product_score(env: Env, product_id: Symbol) -> i32;
    fn get_trending_products(env: Env) -> Vec<Symbol>;

    // SECURITY FIX: Admin Functions
    fn init_admin(
        env: Env,
        admin: Address,
        max_products_per_user: u32,
        voting_period_days: u32,
        reversal_window_hours: u32,
    ) -> Result<(), Error>;
    fn get_admin_config(env: Env) -> Result<AdminConfig, Error>;
    fn deactivate_product(env: Env, product_id: Symbol) -> Result<(), Error>;
    fn reset_rankings(env: Env) -> Result<(), Error>;
    
    // ANALYTICS: Statistics and monitoring
    fn get_ranking_stats(env: Env) -> Option<(u32, i32, i32)>;
    
    // TRANSPARENCY: Vote history access
    fn get_vote_history(env: Env, product_id: Symbol) -> Option<Vec<types::VoteHistoryEntry>>;
}

#[contract]
pub struct ProductVoting;

#[contractimpl]
impl ProductVotingTrait for ProductVoting {
    fn init(env: Env) {
        VoteManager::init(&env);
        RankingCalculator::init(&env);
        VoteLimiter::init(&env);
    }

    // SECURITY FIX: Enhanced product creation with authorization
    fn create_product(env: Env, id: Symbol, name: Symbol, creator: Address) -> Result<(), Error> {
        VoteManager::create_product(&env, id, name, creator)
    }

    fn cast_vote(
        env: Env,
        product_id: Symbol,
        vote_type: VoteType,
        voter: Address,
    ) -> Result<(), Error> {
        // SECURITY FIX: Check limits without recording the vote first
        VoteLimiter::check_limits(&env, &voter)?;

        // SECURITY FIX: Cast the vote and only proceed if successful
        VoteManager::cast_vote(&env, product_id.clone(), vote_type, voter.clone())?;

        // SECURITY FIX: Only record the vote count AFTER successful casting
        VoteLimiter::record_vote(&env, &voter)?;

        // Update rankings after successful vote
        RankingCalculator::update_ranking(&env, product_id);

        Ok(())
    }

    fn get_product_score(env: Env, product_id: Symbol) -> i32 {
        RankingCalculator::get_score(&env, product_id)
    }

    fn get_trending_products(env: Env) -> Vec<Symbol> {
        RankingCalculator::get_trending(&env)
    }

    // ADMIN FUNCTIONS: Require admin authorization
    fn init_admin(
        env: Env,
        admin: Address,
        max_products_per_user: u32,
        voting_period_days: u32,
        reversal_window_hours: u32,
    ) -> Result<(), Error> {
        VoteManager::init_admin(
            &env,
            admin,
            max_products_per_user,
            voting_period_days,
            reversal_window_hours,
        )
    }

    fn get_admin_config(env: Env) -> Result<AdminConfig, Error> {
        VoteManager::get_admin_config(&env)
    }

    fn deactivate_product(env: Env, product_id: Symbol) -> Result<(), Error> {
        VoteManager::deactivate_product(&env, product_id)
    }

    fn reset_rankings(env: Env) -> Result<(), Error> {
        RankingCalculator::reset_rankings(&env)
    }

    // ANALYTICS: Performance monitoring
    fn get_ranking_stats(env: Env) -> Option<(u32, i32, i32)> {
        RankingCalculator::get_ranking_stats(&env)
    }

    // TRANSPARENCY: Audit trail access
    fn get_vote_history(env: Env, product_id: Symbol) -> Option<Vec<types::VoteHistoryEntry>> {
        let product = VoteManager::get_product(&env, product_id)?;
        Some(product.vote_history)
    }
}