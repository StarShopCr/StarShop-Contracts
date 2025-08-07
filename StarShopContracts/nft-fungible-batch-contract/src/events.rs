//! Event definitions and emission for Starshop NFT-Fungible Batch Distribution Contract

use soroban_sdk::{Env, Address, symbol_short};

// Event: RoleGranted
pub fn emit_role_granted(env: &Env, role: &[u8; 32], account: &Address, sender: &Address) {
    env.events().publish(
        (symbol_short!("RoleGrant"), role.to_vec(), account.clone()),
        sender.clone(),
    );
}

// Event: RoleRevoked
pub fn emit_role_revoked(env: &Env, role: &[u8; 32], account: &Address, sender: &Address) {
    env.events().publish(
        (symbol_short!("RoleRevok"), role.to_vec(), account.clone()),
        sender.clone(),
    );
}

// Event: ContractInitialized
pub fn emit_contract_initialized(env: &Env, admin: &Address) {
    env.events().publish(
        (symbol_short!("Init"),),
        admin.clone(),
    );
}

// Event: NFTMinted
pub fn emit_nft_minted(env: &Env, token_id: u128, recipient: &Address) {
    env.events().publish(
        (symbol_short!("NFTMint"), token_id),
        recipient.clone(),
    );
}

// Event: NFTBatchMinted
pub fn emit_nft_batch_minted(env: &Env, quantity: u32, recipient: &Address) {
    env.events().publish(
        (symbol_short!("BatchMint"), quantity),
        recipient.clone(),
    );
}

// Event: Transfer
pub fn emit_transfer(env: &Env, from: &Address, to: &Address, token_id: u128) {
    env.events().publish(
        (symbol_short!("Transfer"), from.clone(), to.clone()),
        token_id,
    );
}