//! Transfer logic for Starshop NFT-Fungible Batch Distribution Contract

use soroban_sdk::{Env, Address};
use crate::storage::{get_token_owner, set_token_owner, get_owner_balance, set_owner_balance};
use crate::events::emit_transfer;

/// Transfer an NFT from one address to another.
pub fn transfer(env: &Env, from: &Address, to: &Address, token_id: u128, sender: &Address) {
    // Check ownership
    let owner = get_token_owner(env, token_id).expect("Token does not exist");
    if &owner != from {
        panic!("From address is not the owner");
    }
    if sender != from {
        panic!("Sender is not authorized to transfer this token");
    }
    // Update token owner
    set_token_owner(env, token_id, to);
    // Update balances
    let from_balance = get_owner_balance(env, from);
    let to_balance = get_owner_balance(env, to);
    set_owner_balance(env, from, from_balance.saturating_sub(1));
    set_owner_balance(env, to, to_balance + 1);
    // Emit event
    emit_transfer(env, from, to, token_id);
}