use soroban_sdk::{Address, Env, Symbol, panic_with_error};
use crate::storage::{ContractError};

pub const DEFAULT_ADMIN_ROLE: u32 = 0;
pub const MINTER_ROLE: u32 = 1;
pub const REDEEMER_ROLE: u32 = 2;


pub fn require_role(env: &Env, role: Symbol) {
    let caller = env.current_contract_address();
    if !has_role(env, role, &caller) {
        panic_with_error!(env, ContractError::MissingRole);
    }
}

pub fn has_role(env: &Env, role: Symbol, account: &Address) -> bool {
    let role_key = (role, account.clone());
    env.storage().persistent().get(&role_key).unwrap_or(false)
}