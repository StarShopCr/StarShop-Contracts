use crate::{metadata, redeem, roles, storage};
use soroban_sdk::{Address, Env, String};
use stellar_tokens::non_fungible::Base;

pub fn mint_promo(env: &Env, caller: &Address, recipient: &Address, uri: Option<String>) -> u32 {
    roles::require_role(&env, caller, storage::MINTER_ROLE);

    // Mint the NFT using OpenZeppelin's sequential mint
    let token_id = Base::sequential_mint(&env, &recipient);

    // Set custom metadata if provided
    if let Some(uri) = uri {
        metadata::set_token_metadata(&env, token_id, &uri);
    }

    // Initialize redemption state
    redeem::initialize_token_redemption(&env, token_id);

    // Emit promo minted event
    env.events().publish(
        (storage::PROMO_MINTED_EVENT,),
        (token_id, recipient.clone()),
    );

    token_id
}
