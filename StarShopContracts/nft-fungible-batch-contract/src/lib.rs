#![no_std]

pub mod events;
pub mod roles;
pub mod storage;

#[cfg(test)]
mod test;

use crate::storage::{
    get_balance, get_base_uri, get_name, get_symbol, get_token_owner, get_total_supply,
    is_initialized, require_initialized, set_balance, set_token_owner, set_total_supply,
    ContractError, DataKey, DEFAULT_ADMIN_ROLE, MAX_BATCH_SIZE, MINTER_ROLE,
};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, Env, String, Symbol, Vec,
};

pub trait BatchNFTContractTrait {
    fn initialize(env: Env, admin: Address, name: String, symbol: String, base_uri: String);
    fn mint_batch(env: Env, caller: Address, recipient: Address, quantity: u32) -> Vec<u32>;
    fn transfer(env: Env, from: Address, to: Address, token_id: u32);
    fn balance_of(env: Env, owner: Address) -> u32;
    fn owner_of(env: Env, token_id: u32) -> Address;
    fn total_supply(env: Env) -> u32;
    fn name(env: Env) -> String;
    fn symbol(env: Env) -> String;
    fn base_uri(env: Env) -> String;
    fn token_uri(env: Env, token_id: u32) -> String;
    fn has_role(env: Env, role: Symbol, account: Address) -> bool;
    fn grant_role(env: Env, caller: Address, role: Symbol, account: Address);
    fn revoke_role(env: Env, caller: Address, role: Symbol, account: Address);
}

#[contract]
pub struct BatchNFTContract;

#[contractimpl]
impl BatchNFTContractTrait for BatchNFTContract {
    fn initialize(env: Env, admin: Address, name: String, symbol: String, base_uri: String) {
        if is_initialized(&env) {
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        env.storage().instance().set(&DataKey::BaseUri, &base_uri);
        env.storage().instance().set(&DataKey::TotalSupply, &0u32);

        // Grant DEFAULT_ADMIN_ROLE and MINTER_ROLE to the initial admin
        let admin_role_key = DataKey::Role(DEFAULT_ADMIN_ROLE, admin.clone());
        env.storage().persistent().set(&admin_role_key, &true);

        let minter_role_key = DataKey::Role(MINTER_ROLE, admin.clone());
        env.storage().persistent().set(&minter_role_key, &true);

        events::emit_contract_initialized(&env, &admin, &name, &symbol);
    }

    fn mint_batch(env: Env, caller: Address, recipient: Address, quantity: u32) -> Vec<u32> {
        require_initialized(&env);
        caller.require_auth();
        roles::require_role(&env, &caller, MINTER_ROLE);

        if quantity == 0 {
            panic_with_error!(&env, ContractError::ZeroQuantity);
        }
        if quantity > MAX_BATCH_SIZE {
            panic_with_error!(&env, ContractError::BatchLimitExceeded);
        }

        let current_supply = get_total_supply(&env);
        let start_id = current_supply + 1;
        let mut minted_ids = Vec::new(&env);

        for i in 0..quantity {
            let token_id = start_id + i;
            set_token_owner(&env, token_id, &recipient);
            events::emit_nft_minted(&env, &recipient, token_id);
            minted_ids.push_back(token_id);
        }

        let new_supply = current_supply + quantity;
        set_total_supply(&env, new_supply);

        let current_balance = get_balance(&env, &recipient);
        set_balance(&env, &recipient, current_balance + quantity);

        events::emit_nft_batch_minted(&env, &caller, &recipient, start_id, quantity);

        minted_ids
    }

    fn transfer(env: Env, from: Address, to: Address, token_id: u32) {
        require_initialized(&env);
        from.require_auth();

        if from == to {
            panic_with_error!(&env, ContractError::SelfTransfer);
        }

        let current_owner = match get_token_owner(&env, token_id) {
            Some(owner) => owner,
            None => panic_with_error!(&env, ContractError::TokenNotFound),
        };

        if current_owner != from {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        set_token_owner(&env, token_id, &to);

        let from_bal = get_balance(&env, &from);
        set_balance(&env, &from, from_bal.saturating_sub(1));

        let to_bal = get_balance(&env, &to);
        set_balance(&env, &to, to_bal + 1);

        events::emit_transfer(&env, &from, &to, token_id);
    }

    fn balance_of(env: Env, owner: Address) -> u32 {
        require_initialized(&env);
        get_balance(&env, &owner)
    }

    fn owner_of(env: Env, token_id: u32) -> Address {
        require_initialized(&env);
        match get_token_owner(&env, token_id) {
            Some(owner) => owner,
            None => panic_with_error!(&env, ContractError::TokenNotFound),
        }
    }

    fn total_supply(env: Env) -> u32 {
        require_initialized(&env);
        get_total_supply(&env)
    }

    fn name(env: Env) -> String {
        require_initialized(&env);
        get_name(&env)
    }

    fn symbol(env: Env) -> String {
        require_initialized(&env);
        get_symbol(&env)
    }

    fn base_uri(env: Env) -> String {
        require_initialized(&env);
        get_base_uri(&env)
    }

    fn token_uri(env: Env, token_id: u32) -> String {
        require_initialized(&env);
        // Ensure token exists
        let _ = Self::owner_of(env.clone(), token_id);
        get_base_uri(&env)
    }

    fn has_role(env: Env, role: Symbol, account: Address) -> bool {
        require_initialized(&env);
        roles::has_role(&env, role, &account)
    }

    fn grant_role(env: Env, caller: Address, role: Symbol, account: Address) {
        require_initialized(&env);
        roles::grant_role(&env, role, &account, &caller);
    }

    fn revoke_role(env: Env, caller: Address, role: Symbol, account: Address) {
        require_initialized(&env);
        roles::revoke_role(&env, role, &account, &caller);
    }
}
