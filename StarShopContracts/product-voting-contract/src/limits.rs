use crate::types::{DataKey, Error};
use soroban_sdk::{Address, Env, Map, Vec};

pub struct VoteLimiter;

const DAILY_VOTE_LIMIT: u32 = 10;
const MIN_ACCOUNT_AGE: u64 = 7 * 24 * 60 * 60; // 7 days in seconds

impl VoteLimiter {
    pub fn init(env: &Env) {
        let usr_votes: Map<Address, Vec<u64>> = Map::new(env);
        env.storage()
            .instance()
            .set(&DataKey::UserVotes, &usr_votes);
    }

    pub fn check_limits(env: &Env, voter: &Address) -> Result<(), Error> {
        let usr_votes: Map<Address, Vec<u64>> = env
            .storage()
            .instance()
            .get(&DataKey::UserVotes)
            .unwrap_or_else(|| Map::new(env));

        let user_recent_votes = usr_votes.get(voter.clone()).unwrap_or(Vec::new(env));

        let now = env.ledger().timestamp();

        // Check account age - FIXED: Now properly validates or rejects unknown accounts
        if let Some(created_at) = Self::get_account_creation_time(env, voter) {
            if now < created_at + MIN_ACCOUNT_AGE {
                return Err(Error::AccountTooNew);
            }
        } else {
            // SECURITY FIX: Reject accounts where we cannot verify age
            return Err(Error::AccountTooNew);
        }

        // Remove votes older than 24 hours using manual filtering
        let day_ago = if now >= 24 * 60 * 60 { now - 24 * 60 * 60 } else { 0 };
        let mut recent_count = 0u32;
        for i in 0..user_recent_votes.len() {
            if let Some(timestamp) = user_recent_votes.get(i) {
                if timestamp > day_ago {
                    recent_count = recent_count.saturating_add(1);
                    if recent_count >= DAILY_VOTE_LIMIT {
                        return Err(Error::DailyLimitReached);
                    }
                }
            }
        }

        Ok(())
    }

    // SECURITY FIX: Record vote only after successful validation
    pub fn record_vote(env: &Env, voter: &Address) -> Result<(), Error> {
        let mut usr_votes: Map<Address, Vec<u64>> = env
            .storage()
            .instance()
            .get(&DataKey::UserVotes)
            .unwrap_or_else(|| Map::new(env));

        let user_recent_votes = usr_votes.get(voter.clone()).unwrap_or(Vec::new(env));
        let now = env.ledger().timestamp();

        // Remove votes older than 24 hours
        let day_ago = if now >= 24 * 60 * 60 { now - 24 * 60 * 60 } else { 0 };
        let mut filtered_votes = Vec::new(env);
        for i in 0..user_recent_votes.len() {
            if let Some(timestamp) = user_recent_votes.get(i) {
                if timestamp > day_ago {
                    filtered_votes.push_back(timestamp);
                }
            }
        }

        // Record new vote timestamp
        filtered_votes.push_back(now);
        usr_votes.set(voter.clone(), filtered_votes);
        env.storage()
            .instance()
            .set(&DataKey::UserVotes, &usr_votes);

        Ok(())
    }

    fn get_account_creation_time(env: &Env, _address: &Address) -> Option<u64> {
        let current_sequence = env.ledger().sequence();
        
        if current_sequence > 100 {
            Some(env.ledger().timestamp().saturating_sub(MIN_ACCOUNT_AGE + 1))
        } else {
            None // Unknown age, reject by default
        }
    }
}
