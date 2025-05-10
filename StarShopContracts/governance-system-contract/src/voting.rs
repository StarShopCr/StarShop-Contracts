use crate::proposals::ProposalManager;
use crate::types::{Error, ProposalType, UserLevel, Vote, VotingConfig, REFERRAL_KEY};
use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal, Symbol, Val, Vec};

pub struct VotingSystem;

impl VotingSystem {
    pub fn cast_vote(
        env: &Env,
        proposal_id: u32,
        voter: &Address,
        support: bool,
        weight: i128,
    ) -> Result<(), Error> {
        voter.require_auth();
        let proposal = ProposalManager::get_proposal(env, proposal_id)?;
        let referral: Address = env
            .storage()
            .instance()
            .get(&REFERRAL_KEY)
            .ok_or(Error::NotInitialized)?;

        // Check KYC/verification
        let result: Val = env.invoke_contract(
            &referral,
            &Symbol::new(&env, "is_user_verified"),
            Vec::from_array(env, [voter.into_val(env)]),
        );
        let is_verified: bool = env.storage().instance().get(&result).unwrap_or(false);
        if !is_verified {
            return Err(Error::NotVerified);
        }

        // Check referral level for economic changes
        if matches!(proposal.proposal_type, ProposalType::EconomicChange) {
            let args = vec![&env, voter.to_val()];
            let result: Val =
                env.invoke_contract(&referral, &Symbol::new(&env, "get_user_level"), args);
            let user_level: UserLevel = env
                .storage()
                .instance()
                .get(&result)
                .unwrap_or(UserLevel::Basic);
            if !matches!(user_level, UserLevel::Platinum) {
                return Err(Error::InsufficientReferralLevel);
            }
        }

        let key = Self::get_vote_key(env, proposal_id, voter);
        if env.storage().instance().has(&key) {
            return Err(Error::AlreadyVoted);
        }
        let vote = Vote {
            voter: voter.clone(),
            support,
            weight: if proposal.voting_config.one_address_one_vote {
                1
            } else {
                weight
            },
            timestamp: env.ledger().timestamp(),
        };
        env.storage().instance().set(&key, &vote);
        Self::update_vote_totals(env, proposal_id, support, vote.weight);
        env.events().publish(
            (symbol_short!("vote"), symbol_short!("cast")),
            (proposal_id, voter, support, vote.weight),
        );
        Ok(())
    }

    fn update_vote_totals(env: &Env, proposal_id: u32, support: bool, weight: i128) {
        let key_for = Self::get_vote_totals_key(env, proposal_id, true);
        let key_against = Self::get_vote_totals_key(env, proposal_id, false);
        let key_total = Self::get_vote_total_key(env, proposal_id);
        let mut for_votes: i128 = env.storage().instance().get(&key_for).unwrap_or(0);
        let mut against_votes: i128 = env.storage().instance().get(&key_against).unwrap_or(0);
        let mut total_votes: i128 = env.storage().instance().get(&key_total).unwrap_or(0);
        if support {
            for_votes += weight;
        } else {
            against_votes += weight;
        }
        total_votes += weight;
        env.storage().instance().set(&key_for, &for_votes);
        env.storage().instance().set(&key_against, &against_votes);
        env.storage().instance().set(&key_total, &total_votes);
        let key_voter_count = Self::get_voter_count_key(env, proposal_id);
        let voter_count: u32 = env.storage().instance().get(&key_voter_count).unwrap_or(0) + 1;
        env.storage().instance().set(&key_voter_count, &voter_count);
    }

    pub fn check_voting_ended(
        env: &Env,
        proposal_id: u32,
        config: &VotingConfig,
    ) -> Result<bool, Error> {
        let proposal = ProposalManager::get_proposal(env, proposal_id)?;
        if proposal.activated_at == 0 {
            return Ok(false);
        }
        let current_time = env.ledger().timestamp();
        if current_time >= proposal.activated_at + config.duration {
            return Ok(true);
        }
        if config.one_address_one_vote {
            let voter_count = Self::get_voter_count(env, proposal_id);
            let total_voters = Self::get_total_voters(env)?;
            if voter_count * 10000 / total_voters >= config.quorum {
                let for_votes = Self::get_for_votes(env, proposal_id);
                let total_votes = Self::get_total_votes(env, proposal_id);
                if for_votes * 10000 / total_votes >= config.threshold {
                    return Ok(true);
                }
                if (total_votes - for_votes) * 10000 / total_votes > (10000 - config.threshold) {
                    return Ok(true);
                }
            }
        } else {
            let total_votes = Self::get_total_votes(env, proposal_id);
            let total_voting_power = Self::get_total_voting_power(env, proposal_id);
            if total_votes * 10000 / total_voting_power >= config.quorum {
                let for_votes = Self::get_for_votes(env, proposal_id);
                let against_votes = Self::get_against_votes(env, proposal_id);
                if for_votes * 10000 / total_votes >= config.threshold {
                    return Ok(true);
                }
                if against_votes * 10000 / total_votes > (10000 - config.threshold) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub fn tally_votes(env: &Env, proposal_id: u32, config: &VotingConfig) -> Result<bool, Error> {
        if config.one_address_one_vote {
            let voter_count = Self::get_voter_count(env, proposal_id);
            let total_voters = Self::get_total_voters(env)?;
            if voter_count * 10000 / total_voters < config.quorum {
                return Ok(false);
            }
            let for_votes = Self::get_for_votes(env, proposal_id);
            let total_votes = Self::get_total_votes(env, proposal_id);
            if for_votes * 10000 / total_votes >= config.threshold {
                return Ok(true);
            }
        } else {
            let for_votes = Self::get_for_votes(env, proposal_id);
            let total_votes = Self::get_total_votes(env, proposal_id);
            let total_voting_power = Self::get_total_voting_power(env, proposal_id);
            if total_votes * 10000 / total_voting_power < config.quorum {
                return Ok(false);
            }
            if for_votes * 10000 / total_votes >= config.threshold {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn get_for_votes(env: &Env, proposal_id: u32) -> u128 {
        let key = Self::get_vote_totals_key(env, proposal_id, true);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    pub fn get_against_votes(env: &Env, proposal_id: u32) -> u128 {
        let key = Self::get_vote_totals_key(env, proposal_id, false);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    pub fn get_total_votes(env: &Env, proposal_id: u32) -> u128 {
        let key = Self::get_vote_total_key(env, proposal_id);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    pub fn get_voter_count(env: &Env, proposal_id: u32) -> u128 {
        let key = Self::get_voter_count_key(env, proposal_id);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    pub fn get_total_voters(env: &Env) -> Result<u128, Error> {
        let referral: Address = env
            .storage()
            .instance()
            .get(&REFERRAL_KEY)
            .ok_or(Error::NotInitialized)?;
        let args = vec![&env];
        let result: Val =
            env.invoke_contract(&referral, &Symbol::new(&env, "get_total_users"), args);
        let total_users: u32 = env.storage().instance().get(&result).unwrap_or(0);
        Ok(total_users as u128)
    }

    pub fn get_total_voting_power(env: &Env, proposal_id: u32) -> u128 {
        let key = Symbol::new(&env, &format!("TOTAL_VOTING_POWER_{}", proposal_id));
        env.storage().instance().get(&key).unwrap_or(10000)
    }

    fn get_vote_key(env: &Env, proposal_id: u32, voter: &Address) -> Symbol {
        Symbol::new(
            &env,
            &format!("VOTE_{}:{:?}", proposal_id, voter.to_string()),
        )
    }

    fn get_vote_totals_key(env: &Env, proposal_id: u32, support: bool) -> Symbol {
        Symbol::new(
            &env,
            &format!(
                "VOTE_{}:{}",
                proposal_id,
                if support { "FOR" } else { "AGAINST" }
            ),
        )
    }

    fn get_vote_total_key(env: &Env, proposal_id: u32) -> Symbol {
        Symbol::new(&env, &format!("VOTE_{}:TOTAL", proposal_id))
    }

    fn get_voter_count_key(env: &Env, proposal_id: u32) -> Symbol {
        Symbol::new(&env, &format!("VOTE_{}:COUNT", proposal_id))
    }
}
