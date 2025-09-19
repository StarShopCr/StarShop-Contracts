//! Starshop NFT-Fungible Batch Distribution Contract
//! Implements batch minting, transfer, and role-based access control for NFTs on Stellar using Soroban SDK and OpenZeppelin patterns.

mod mint;
mod transfer;
mod roles;
mod events;
mod storage;

use soroban_sdk::{contract, contractimpl, Env, Address};
use crate::roles::DEFAULT_ADMIN_ROLE;
use crate::storage::{is_initialized, set_initialized, get_token_owner, get_owner_balance, get_total_supply};
use crate::events::emit_contract_initialized;

#[contract]
pub struct NFTFungibleBatchContract;

#[contractimpl]
impl NFTFungibleBatchContract {
    /// Initialize the contract, set the first admin, and mark as initialized.
    pub fn initialize(env: Env, admin: Address) {
        if is_initialized(&env) {
            panic!("Contract already initialized");
        }
        // Grant initial admin role (bypass normal access control during init)
        env.storage().instance().set(&crate::roles::role_key(DEFAULT_ADMIN_ROLE, &admin), &true);
        crate::events::emit_role_granted(&env, DEFAULT_ADMIN_ROLE, &admin, &admin);
        // Mark as initialized
        set_initialized(&env);
        // Emit event
        emit_contract_initialized(&env, &admin);
    }

    /// Mint a batch of NFTs to a recipient. Only callable by MINTER_ROLE.
    pub fn mint_batch(env: Env, recipient: Address, quantity: u32, sender: Address) {
        mint::mint_batch(&env, &recipient, quantity, &sender);
    }

    /// Transfer an NFT from one address to another.
    pub fn transfer(env: Env, from: Address, to: Address, token_id: u128, sender: Address) {
        transfer::transfer(&env, &from, &to, token_id, &sender);
    }

    /// Grant a role to an account. Only callable by admin.
    pub fn grant_role(env: Env, role: Vec<u8>, account: Address, sender: Address) {
        roles::grant_role(&env, &role.try_into().expect("role must be 32 bytes"), &account, &sender);
    }

    /// Revoke a role from an account. Only callable by admin.
    pub fn revoke_role(env: Env, role: Vec<u8>, account: Address, sender: Address) {
        roles::revoke_role(&env, &role.try_into().expect("role must be 32 bytes"), &account, &sender);
    }

    // Query functions
    /// Get the owner of a token
    pub fn owner_of(env: Env, token_id: u128) -> Option<Address> {
        get_token_owner(&env, token_id)
    }

    /// Get the balance (number of NFTs) of an address
    pub fn balance_of(env: Env, owner: Address) -> u32 {
        get_owner_balance(&env, &owner)
    }

    /// Get the total supply of NFTs
    pub fn total_supply(env: Env) -> u32 {
        get_total_supply(&env)
    }

    /// Check if an address has a role
    pub fn has_role(env: Env, role: Vec<u8>, account: Address) -> bool {
        roles::has_role(&env, &role.try_into().expect("role must be 32 bytes"), &account)
    }
}
