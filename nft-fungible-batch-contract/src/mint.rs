use soroban_sdk::{Env, Address};
use crate::storage;
use crate::events;

pub fn mint_batch(env: &Env, recipient: &Address, quantity: u32) {
    if quantity == 0 {
        panic!("quantity must be > 0");
    }
    for _ in 0..quantity {
        let token_id = storage::increment_token_counter(env);
        storage::set_token_owner(env, token_id, recipient);
        events::emit_nft_minted(env, token_id, recipient);
    }
    events::emit_nft_batch_minted(env, quantity, recipient);
}
