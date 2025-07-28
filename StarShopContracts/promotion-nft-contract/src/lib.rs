#![no_std]

pub mod metadata;
pub mod mint;
pub mod redeem;
pub mod roles;
pub mod storage;
pub mod test;

use soroban_sdk::{Address, Env, String, Symbol, contract, contractimpl, panic_with_error};
use stellar_macros::default_impl;
use stellar_tokens::non_fungible::{Base, NonFungibleToken};

#[contract]
pub struct StarshopPromoNFT;

#[contractimpl]
impl StarshopPromoNFT {
    /// Initialize the contract with admin role and metadata
    pub fn initialize(env: Env, admin: Address, name: String, symbol: String, base_uri: String) {
        storage::initialize_contract(&env, &admin, name, symbol, base_uri);
    }

    /// Mint a new promotional NFT with optional metadata
    pub fn mint_promo(env: Env, recipient: Address, metadata_uri: Option<String>) -> u32 {
        mint::mint_promo(&env, &recipient, metadata_uri)
    }

    /// Redeem a promotional NFT
    pub fn redeem(env: Env, token_id: u32) {
        redeem::redeem_token(&env, token_id);
    }

    /// Check if an NFT has been redeemed
    pub fn is_redeemed(env: Env, token_id: u32) -> bool {
        redeem::is_token_redeemed(&env, token_id)
    }

    /// Get redemption timestamp for an NFT
    pub fn redemption_timestamp(env: Env, token_id: u32) -> Option<u64> {
        redeem::get_redemption_timestamp(&env, token_id)
    }

    /// Grant role to an address
    pub fn grant_role(env: Env, role: Symbol, account: Address) {
        roles::grant_role(&env, role, &account);
    }

    /// Revoke role from an address
    pub fn revoke_role(env: Env, role: Symbol, account: Address) {
        roles::revoke_role(&env, role, &account);
    }

    /// Check if address has role
    pub fn has_role(env: Env, role: Symbol, account: Address) -> bool {
        roles::has_role(&env, role, &account)
    }

    /// Get custom NFT metadata URI (overrides base implementation if set)
    pub fn get_token_metadata(env: Env, token_id: u32) -> Option<String> {
        metadata::get_token_metadata(&env, token_id)
    }
}

// Implement the standard NonFungibleToken trait
#[default_impl]
#[contractimpl]
impl NonFungibleToken for StarshopPromoNFT {
    type ContractType = Base;

    // Override transfer methods to include redemption restrictions
    fn transfer(e: &Env, from: Address, to: Address, token_id: u32) {
        if redeem::is_token_redeemed(e, token_id) {
            panic_with_error!(e, storage::ContractError::TransferRestricted);
        }
        Base::transfer(e, &from, &to, token_id);
    }

    fn transfer_from(e: &Env, spender: Address, from: Address, to: Address, token_id: u32) {
        if redeem::is_token_redeemed(e, token_id) {
            panic_with_error!(e, storage::ContractError::TransferRestricted);
        }
        Base::transfer_from(e, &spender, &from, &to, token_id);
    }

    // Override token_uri to support custom metadata
    fn token_uri(e: &Env, token_id: u32) -> String {
        if let Some(custom_uri) = metadata::get_token_metadata(e, token_id) {
            return custom_uri;
        }
        Base::token_uri(e, token_id)
    }
}
