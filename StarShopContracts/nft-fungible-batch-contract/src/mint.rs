//! Batch minting logic for Starshop NFT-Fungible Batch Distribution Contract

use soroban_sdk::{Env, Address};
use crate::roles::{MINTER_ROLE, has_role};
use crate::storage::{next_token_id, set_token_owner, get_owner_balance, set_owner_balance, increment_total_supply};
use crate::events::{emit_nft_minted, emit_nft_batch_minted};

/// Mint a batch of NFTs to a recipient. Only callable by MINTER_ROLE.
pub fn mint_batch(env: &Env, recipient: &Address, quantity: u32, sender: &Address) {
    // Validate inputs
    if !has_role(env, MINTER_ROLE, sender) {
        panic!("Only minters can mint");
    }
    if quantity == 0 {
        panic!("Quantity must be greater than zero");
    }
    // Mint unique NFTs
    for _ in 0..quantity {
        let token_id = next_token_id(env);
        set_token_owner(env, token_id, recipient);
        emit_nft_minted(env, token_id, recipient);
    }
    // Update recipient's balance
    let prev_balance = get_owner_balance(env, recipient);
    set_owner_balance(env, recipient, prev_balance + quantity);
    // Update total supply
    increment_total_supply(env, quantity);
    // Emit batch event
    emit_nft_batch_minted(env, quantity, recipient);
}