use soroban_sdk::{
    contracterror, contracttype, panic_with_error, symbol_short, Address, Env, String, Symbol,
};

// Roles
pub const DEFAULT_ADMIN_ROLE: Symbol = symbol_short!("ADMIN");
pub const MINTER_ROLE: Symbol = symbol_short!("MINTER");

// Maximum tokens allowed per single batch minting call
pub const MAX_BATCH_SIZE: u32 = 500;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    MissingRole = 3,
    Unauthorized = 4,
    TokenNotFound = 5,
    ZeroQuantity = 6,
    BatchLimitExceeded = 7,
    SelfTransfer = 8,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Initialized,
    Admin,
    Name,
    Symbol,
    BaseUri,
    TotalSupply,
    TokenOwner(u32),
    Balance(Address),
    Role(Symbol, Address),
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Initialized)
}

pub fn require_initialized(env: &Env) {
    if !is_initialized(env) {
        panic_with_error!(env, ContractError::NotInitialized);
    }
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("Admin address not set")
}

pub fn get_name(env: &Env) -> String {
    env.storage()
        .instance()
        .get(&DataKey::Name)
        .expect("Name not set")
}

pub fn get_symbol(env: &Env) -> String {
    env.storage()
        .instance()
        .get(&DataKey::Symbol)
        .expect("Symbol not set")
}

pub fn get_base_uri(env: &Env) -> String {
    env.storage()
        .instance()
        .get(&DataKey::BaseUri)
        .expect("Base URI not set")
}

pub fn get_total_supply(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

pub fn set_total_supply(env: &Env, supply: u32) {
    env.storage().instance().set(&DataKey::TotalSupply, &supply);
}

pub fn get_token_owner(env: &Env, token_id: u32) -> Option<Address> {
    env.storage().persistent().get(&DataKey::TokenOwner(token_id))
}

pub fn set_token_owner(env: &Env, token_id: u32, owner: &Address) {
    env.storage()
        .persistent()
        .set(&DataKey::TokenOwner(token_id), owner);
}

pub fn get_balance(env: &Env, owner: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(owner.clone()))
        .unwrap_or(0)
}

pub fn set_balance(env: &Env, owner: &Address, balance: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::Balance(owner.clone()), &balance);
}
