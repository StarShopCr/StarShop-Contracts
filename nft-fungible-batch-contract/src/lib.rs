use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

mod storage;
mod events;
mod roles;
mod mint;
mod transfer;

#[contract]
pub struct NftFungibleBatchContract;

#[contractimpl]
impl NftFungibleBatchContract {
    pub fn initialize(env: Env, admin: Address) {
        storage::ensure_not_initialized(&env);
        storage::set_initialized(&env, true);
        roles::grant_role(&env, &roles::default_admin_role(&env), &admin);
        events::emit_contract_initialized(&env);
    }

    pub fn mint_batch(env: Env, caller: Address, recipient: Address, quantity: u32) {
        caller.require_auth();
        roles::require_role(&env, &caller, &roles::minter_role(&env));
        mint::mint_batch(&env, &recipient, quantity);
    }

    pub fn transfer(env: Env, caller: Address, from: Address, to: Address, token_id: u64) {
        caller.require_auth();
        let owner = storage::get_token_owner(&env, token_id).expect("token not found");
        if owner != from {
            panic!("not owner");
        }
        if caller != from {
            panic!("caller not owner");
        }
        transfer::transfer(&env, &from, &to, token_id);
    }

    pub fn grant_role(env: Env, caller: Address, role: Symbol, account: Address) {
        caller.require_auth();
        roles::require_role(&env, &caller, &roles::default_admin_role(&env));
        roles::grant_role(&env, &role, &account);
    }

    pub fn revoke_role(env: Env, caller: Address, role: Symbol, account: Address) {
        caller.require_auth();
        roles::require_role(&env, &caller, &roles::default_admin_role(&env));
        roles::revoke_role(&env, &role, &account);
    }
}
