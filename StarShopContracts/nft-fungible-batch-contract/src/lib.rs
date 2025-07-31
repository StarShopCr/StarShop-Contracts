//! Starshop NFT-Fungible Batch Distribution Contract
//! Implements batch minting, transfer, and role-based access control for NFTs on Stellar using Soroban SDK and OpenZeppelin patterns.

mod mint;
mod transfer;
mod roles;
mod events;
mod storage;

use soroban_sdk::{contract, contractimpl, Env, Address};
use crate::roles::{DEFAULT_ADMIN_ROLE, grant_role};
use crate::storage::{is_initialized, set_initialized};
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
        // Grant admin role
        grant_role(&env, DEFAULT_ADMIN_ROLE, &admin, &admin);
        // Mark as initialized
        set_initialized(&env);
        // Emit event
        emit_contract_initialized(&env, &admin);
    }
    // TODO: Add mint, transfer, and role management entrypoints
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
}