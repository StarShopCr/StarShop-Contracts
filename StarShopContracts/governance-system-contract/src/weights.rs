use crate::types::{
    Error, ProposalRequirements, WeightSnapshot, REFERRAL_KEY, REQUIREMENTS_KEY, TOKEN_KEY,
};
use soroban_sdk::{symbol_short, token, vec, Address, Env, Symbol, Val, Vec};

pub struct WeightCalculator;

impl WeightCalculator {
    pub fn take_snapshot(env: &Env, proposal_id: u32) -> Result<(), Error> {
        let referral: Address = env
            .storage()
            .instance()
            .get(&REFERRAL_KEY)
            .ok_or(Error::NotInitialized)?;
        let snapshot = WeightSnapshot {
            proposal_id,
            snapshot_at: env.ledger().timestamp(),
        };
        let key = Self::get_snapshot_key(env, proposal_id);
        env.storage().instance().set(&key, &snapshot);
        let args = vec![&env];
        let result: Val =
            env.invoke_contract(&referral, &Symbol::new(env, "get_total_users"), args);
        let total_users: u32 = env.storage().instance().get(&result).unwrap_or(0);
        let mut total_power = 0i128;
        // Placeholder: Need user registry
        for _ in 0..total_users {
            total_power += 1000; // Simulate balance
        }
        env.storage().instance().set(
            &Symbol::new(&env, &format!("TOTAL_VOTING_POWER_{}", proposal_id)),
            &total_power,
        );

        env.events().publish(
            (symbol_short!("weight"), symbol_short!("snapshot")),
            (proposal_id, total_power),
        );
        Ok(())
    }

    pub fn get_weight(env: &Env, voter: &Address, proposal_id: u32) -> Result<i128, Error> {
        if let Some(_delegatee) = Self::get_delegation(env, voter) {
            return Ok(0);
        }
        let requirements: ProposalRequirements =
            env.storage().instance().get(&REQUIREMENTS_KEY).unwrap();
        let base_weight = Self::get_base_weight(env, voter);
        let delegated_weight = Self::get_delegated_weight(env, voter, proposal_id);
        let total_weight = base_weight + delegated_weight;
        Ok(total_weight.min(requirements.max_voting_power))
    }

    pub fn get_base_weight(env: &Env, voter: &Address) -> i128 {
        let token: Address = env.storage().instance().get(&TOKEN_KEY).unwrap();
        let token_client = token::TokenClient::new(env, &token);
        token_client.balance(voter)
    }

    fn get_delegated_weight(env: &Env, delegatee: &Address, _proposal_id: u32) -> i128 {
        let delegators = Self::get_delegators(env, delegatee);
        delegators
            .iter()
            .map(|delegator| Self::get_base_weight(env, &delegator))
            .sum()
    }

    pub fn delegate(env: &Env, delegator: &Address, delegatee: &Address) -> Result<(), Error> {
        delegator.require_auth();
        if delegator == delegatee {
            return Err(Error::SelfDelegationNotAllowed);
        }
        if let Some(current_delegatee) = Self::get_delegation(env, delegatee) {
            if current_delegatee == *delegator {
                return Err(Error::InvalidDelegation);
            }
        }
        let key = Self::get_delegator_key(env, delegator);
        env.storage().instance().set(&key, delegatee);
        Self::add_to_delegators_list(env, delegatee, delegator);
        env.events().publish(
            (symbol_short!("vote"), symbol_short!("delegated")),
            (delegator, delegatee),
        );
        Ok(())
    }

    pub fn get_delegation(env: &Env, delegator: &Address) -> Option<Address> {
        let key = Self::get_delegator_key(env, delegator);
        env.storage().instance().get(&key)
    }

    pub fn get_delegators(env: &Env, delegatee: &Address) -> Vec<Address> {
        let key = Self::get_delegatees_key(env, delegatee);
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| vec![env])
    }

    fn get_snapshot_key(env: &Env, proposal_id: u32) -> Symbol {
        Symbol::new(&env, &format!("SNAP_{}", proposal_id))
    }

    fn get_delegator_key(env: &Env, delegator: &Address) -> Symbol {
        Symbol::new(&env, &format!("DELG:{:?}", delegator.to_string()))
    }

    fn get_delegatees_key(env: &Env, delegatee: &Address) -> Symbol {
        Symbol::new(&env, &format!("DELG_TO:{:?}", delegatee.to_string()))
    }

    fn add_to_delegators_list(env: &Env, delegatee: &Address, delegator: &Address) {
        let key = Self::get_delegatees_key(env, delegatee);
        let mut list: Vec<Address> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| vec![env]);
        if !list.iter().any(|a| a == *delegator) {
            list.push_back(delegator.clone());
            env.storage().instance().set(&key, &list);
        }
    }

    fn remove_from_delegators_list(env: &Env, delegatee: &Address, delegator: &Address) {
        let key = Self::get_delegatees_key(env, delegatee);
        let list: Vec<Address> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| vec![env]);

        let mut new_list = Vec::new(env);
        for addr in list.iter() {
            if addr != *delegator {
                new_list.push_back(addr.clone());
            }
        }

        env.storage().instance().set(&key, &new_list);
    }
}
