#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec,
};

mod distribution;
mod metadata;
mod minting;

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const COUNTER_KEY: Symbol = symbol_short!("COUNTER");

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct NFTMetadata {
    pub name: String,
    pub description: String,
    pub attributes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct NFTDetail {
    pub owner: Address,
    pub metadata: NFTMetadata,
}

#[contract]
pub struct NFTContract;

#[contractimpl]
impl NFTContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&COUNTER_KEY, &0u32);
    }

    fn check_admin(env: &Env, caller: &Address) {
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        if caller != &admin {
            panic!("Unauthorized");
        }
    }

    pub fn verify_admin(env: Env, caller: Address) {
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap_or_else(|| {
            panic!("Contract not initialized");
        });
        
        if caller != admin {
            panic!("Unauthorized: Only admin can perform this action");
        }
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN_KEY).unwrap()
    }

    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&ADMIN_KEY)
    }
}

#[cfg(test)]
mod test;
