use crate::types::VoteType;
use crate::vote::VoteManager;
use soroban_sdk::{symbol_short, Env, Map, Symbol, Vec};

pub struct RankingCalculator;

const TRENDING_WINDOW: u64 = 48 * 60 * 60; // 48 hours in seconds

impl RankingCalculator {
    pub fn init(env: &Env) {
        let rankings: Map<Symbol, i32> = Map::new(env);
        env.storage()
            .instance()
            .set(&symbol_short!("rankings"), &rankings);
    }

    pub fn update_ranking(env: &Env, product_id: Symbol) {
        let score = Self::calculate_score(env, product_id.clone());

        let mut rankings: Map<Symbol, i32> = env
            .storage()
            .instance()
            .get(&symbol_short!("rankings"))
            .unwrap_or_else(|| Map::new(env));
        rankings.set(product_id, score);
        env.storage()
            .instance()
            .set(&symbol_short!("rankings"), &rankings);
    }

    pub fn get_score(env: &Env, product_id: Symbol) -> i32 {
        let rankings: Map<Symbol, i32> = env
            .storage()
            .instance()
            .get(&symbol_short!("rankings"))
            .unwrap_or_else(|| Map::new(env));
        rankings.get(product_id).unwrap_or(0)
    }

    pub fn get_trending(env: &Env) -> Vec<Symbol> {
        let rankings: Map<Symbol, i32> = env
            .storage()
            .instance()
            .get(&symbol_short!("rankings"))
            .unwrap_or_else(|| Map::new(env));
        let mut result = Vec::new(env);

        // Convert to vector of tuples
        let mut pairs = Vec::new(env);
        for (id, score) in rankings.iter() {
            pairs.push_back((id, score));
        }

        // Manual sorting since Soroban Vec doesn't have sort_by
        let n = pairs.len();
        for i in 0..n {
            for j in 0..(n - i - 1) {
                if pairs.get(j).unwrap().1 < pairs.get(j + 1).unwrap().1 {
                    let temp = pairs.get(j).unwrap();
                    pairs.set(j, pairs.get(j + 1).unwrap());
                    pairs.set(j + 1, temp);
                }
            }
        }

        // Extract only the IDs
        for pair in pairs.iter() {
            result.push_back(pair.0);
        }

        result
    }

    fn calculate_score(env: &Env, product_id: Symbol) -> i32 {
        let product = VoteManager::get_product(env, product_id).expect("Product should exist");

        let now = env.ledger().timestamp();
        let age_hours = (now - product.created_at) / 3600;

        // Calculate base score from votes
        let mut base_score = 0i32;
        let votes = product.votes.values();
        for i in 0..votes.len() {
            let vote = votes.get(i).unwrap();
            match vote.vote_type {
                VoteType::Upvote => base_score += 1,
                VoteType::Downvote => base_score -= 1,
            }
        }

        // Count recent votes for trending factor
        let mut recent_votes = 0;
        for i in 0..votes.len() {
            let vote = votes.get(i).unwrap();
            if now - vote.timestamp <= TRENDING_WINDOW {
                recent_votes += 1;
            }
        }

        // Apply time decay and add trending bonus
        base_score / (1 + (age_hours / 24) as i32) + (recent_votes / 2)
    }
}
