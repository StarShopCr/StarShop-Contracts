use crate::types::{DataKey, VoteType};
use crate::vote::VoteManager;
use soroban_sdk::{Env, Map, Symbol, Vec};

pub struct RankingCalculator;

const TRENDING_WINDOW: u64 = 48 * 60 * 60; // 48 hours in seconds
const MAX_TRENDING_RESULTS: u32 = 100; // SECURITY FIX: Limit results to prevent DoS
const MAX_SCORE: i32 = 1_000_000; // SECURITY FIX: Prevent integer overflow
const MIN_SCORE: i32 = -1_000_000;

impl RankingCalculator {
    pub fn init(env: &Env) {
        let rankings: Map<Symbol, i32> = Map::new(env);
        env.storage()
            .instance()
            .set(&DataKey::Rankings, &rankings);
    }

    pub fn update_ranking(env: &Env, product_id: Symbol) {
        // SECURITY FIX: Safe error handling - don't panic on missing product
        if let Some(score) = Self::calculate_score_safe(env, product_id.clone()) {
            let mut rankings: Map<Symbol, i32> = env
                .storage()
                .instance()
                .get(&DataKey::Rankings)
                .unwrap_or_else(|| Map::new(env));
            
            rankings.set(product_id, score);
            env.storage()
                .instance()
                .set(&DataKey::Rankings, &rankings);
        }
    }

    pub fn get_score(env: &Env, product_id: Symbol) -> i32 {
        let rankings: Map<Symbol, i32> = env
            .storage()
            .instance()
            .get(&DataKey::Rankings)
            .unwrap_or_else(|| Map::new(env));
        rankings.get(product_id).unwrap_or(0)
    }

    pub fn get_trending(env: &Env) -> Vec<Symbol> {
        let rankings: Map<Symbol, i32> = env
            .storage()
            .instance()
            .get(&DataKey::Rankings)
            .unwrap_or_else(|| Map::new(env));
        
        let mut result = Vec::new(env);

        // SECURITY FIX: Efficient sorting using insertion sort (better than bubble sort)
        // Collect all rankings into a vector first
        let mut scored_products = Vec::new(env);
        for (id, score) in rankings.iter() {
            scored_products.push_back((id, score));
            
            // SECURITY FIX: Limit number of results to prevent DoS
            if scored_products.len() >= MAX_TRENDING_RESULTS {
                break;
            }
        }

        // SECURITY FIX: Use insertion sort (O(n²) worst case but O(n) best case)
        // Much more efficient than bubble sort for partially sorted data
        Self::insertion_sort(&mut scored_products);

        // Extract only the product IDs
        for pair in scored_products.iter() {
            result.push_back(pair.0);
        }

        result
    }

    // SECURITY FIX: Efficient insertion sort instead of bubble sort
    fn insertion_sort(arr: &mut Vec<(Symbol, i32)>) {
        let len = arr.len();
        for i in 1..len {
            if let Some(current) = arr.get(i) {
                let key = current;
                let mut j = i;
                
                // Move elements that are smaller than key one position ahead
                while j > 0 {
                    if let Some(prev) = arr.get(j - 1) {
                        if prev.1 >= key.1 {
                            break;
                        }
                        arr.set(j, prev);
                        j -= 1;
                    } else {
                        break;
                    }
                }
                arr.set(j, key);
            }
        }
    }

    // SECURITY FIX: Safe score calculation without panics
    fn calculate_score_safe(env: &Env, product_id: Symbol) -> Option<i32> {
        let product = VoteManager::get_product(env, product_id)?;

        let now = env.ledger().timestamp();
        
        // SECURITY FIX: Prevent overflow in age calculation
        let age_hours = if now > product.created_at {
            (now - product.created_at).saturating_div(3600)
        } else {
            0
        };

        // Calculate base score from votes with overflow protection
        let mut base_score = 0i32;
        let mut total_votes = 0u32;
        let votes = product.votes.values();
        
        for i in 0..votes.len() {
            if let Some(vote) = votes.get(i) {
                match vote.vote_type {
                    VoteType::Upvote => {
                        base_score = base_score.saturating_add(1);
                    }
                    VoteType::Downvote => {
                        base_score = base_score.saturating_sub(1);
                    }
                    VoteType::None => {
                        // Skip None votes - they don't affect the score
                        continue;
                    }
                }
                total_votes = total_votes.saturating_add(1);
                
                // SECURITY FIX: Prevent processing too many votes (DoS protection)
                if total_votes > 10_000 {
                    break;
                }
            }
        }

        // Count recent votes for trending factor with overflow protection
        let mut recent_votes = 0u32;
        for i in 0..votes.len() {
            if let Some(vote) = votes.get(i) {
                if now >= vote.timestamp && (now - vote.timestamp) <= TRENDING_WINDOW {
                    recent_votes = recent_votes.saturating_add(1);
                }
            }
        }

        // SECURITY FIX: Safe arithmetic with bounds checking
        let time_decay_factor = if age_hours > 0 {
            1i32.saturating_add((age_hours / 24) as i32)
        } else {
            1i32
        };

        let decayed_score = if time_decay_factor > 0 {
            base_score / time_decay_factor
        } else {
            base_score
        };

        let trending_bonus = (recent_votes / 2) as i32;
        let final_score = decayed_score.saturating_add(trending_bonus);

        // SECURITY FIX: Clamp final score to prevent overflow issues
        let clamped_score = final_score.max(MIN_SCORE).min(MAX_SCORE);
        
        Some(clamped_score)
    }

    // ADMIN FUNCTION: Reset rankings
    pub fn reset_rankings(env: &Env) -> Result<(), crate::types::Error> {
        // Verify admin access through VoteManager
        VoteManager::get_admin_config(env)?;
        
        let rankings: Map<Symbol, i32> = Map::new(env);
        env.storage()
            .instance()
            .set(&DataKey::Rankings, &rankings);
        Ok(())
    }

    // ANALYTICS: Get ranking statistics
    pub fn get_ranking_stats(env: &Env) -> Option<(u32, i32, i32)> {
        let rankings: Map<Symbol, i32> = env
            .storage()
            .instance()
            .get(&DataKey::Rankings)?;

        let mut count = 0u32;
        let mut max_score = MIN_SCORE;
        let mut min_score = MAX_SCORE;

        for (_, score) in rankings.iter() {
            count = count.saturating_add(1);
            max_score = max_score.max(score);
            min_score = min_score.min(score);
        }

        if count > 0 {
            Some((count, min_score, max_score))
        } else {
            Some((0, 0, 0))
        }
    }
}
