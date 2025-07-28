use soroban_sdk::{Address, Env, Symbol, String, contracterror, panic_with_error, symbol_short};
use stellar_tokens::non_fungible::{Base};

// Storage keys
pub const INITIALIZED_KEY: Symbol = symbol_short!("INIT");
pub const TOTAL_SUPPLY_KEY: Symbol = symbol_short!("SUPPLY");
pub const OWNER_KEY: Symbol = symbol_short!("OWNER");
pub const REDEEMED_KEY: Symbol = symbol_short!("REDEEM");
pub const METADATA_URI_KEY: Symbol = symbol_short!("META_URI");


// Roles
pub const DEFAULT_ADMIN_ROLE: Symbol = symbol_short!("ADMIN");
pub const MINTER_ROLE: Symbol = symbol_short!("MINTER");
pub const REDEEMER_ROLE: Symbol = symbol_short!("REDEEMER");

// Event topics
pub const CONTRACT_INITIALIZED_EVENT: Symbol = symbol_short!("INIT_EVT");
pub const PROMO_MINTED_EVENT: Symbol = symbol_short!("MINT_EVT");
pub const REDEEMED_EVENT: Symbol = symbol_short!("REDM_EVT");
pub const TRANSFER_EVENT: Symbol = symbol_short!("XFER_EVT");
pub const ROLE_GRANTED_EVENT: Symbol = symbol_short!("ROLE_GRT");
pub const ROLE_REVOKED_EVENT: Symbol = symbol_short!("ROLE_REV");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    TokenNotFound = 3,
    Unauthorized = 4,
    AlreadyRedeemed = 5,
    MissingRole = 6,
    TransferRestricted = 7,
}

pub fn initialize_contract(env: &Env, admin: &Address, name: String, symbol: String, base_uri: String){
    // Check if already initialized
    if env.storage().persistent().has(&INITIALIZED_KEY) {
        panic_with_error!(&env, ContractError::AlreadyInitialized);
    }
    Base::set_metadata(env, base_uri, name, symbol);
    
    // Grant admin role to deployer
    let role_key = (DEFAULT_ADMIN_ROLE, admin.clone());
    env.storage().persistent().set(&role_key, &true);

    // Emit initialization event
    env.events().publish(
        (CONTRACT_INITIALIZED_EVENT,),
        admin.clone()
    );
}

pub fn require_initialized(env: &Env) {
    if !env.storage().persistent().has(&INITIALIZED_KEY) {
        panic_with_error!(&env, ContractError::NotInitialized);
    }
}
