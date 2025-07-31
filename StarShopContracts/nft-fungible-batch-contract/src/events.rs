//! Event definitions and emission for Starshop NFT-Fungible Batch Distribution Contract

use soroban_sdk::{Env, Address, Symbol};

// Event: RoleGranted
pub fn emit_role_granted(env: &Env, role: &[u8; 32], account: &Address, sender: &Address) {
    env.events().publish(
        (Symbol::short("RoleGranted"), role.to_vec(), account.clone()),
        sender.clone(),
    );
}

// Event: RoleRevoked
pub fn emit_role_revoked(env: &Env, role: &[u8; 32], account: &Address, sender: &Address) {
    env.events().publish(
        (Symbol::short("RoleRevoked"), role.to_vec(), account.clone()),
        sender.clone(),
    );
}

// Event: ContractInitialized
pub fn emit_contract_initialized(env: &Env, admin: &Address) {
    env.events().publish(
        Symbol::short("ContractInitialized"),
        admin.clone(),
    );
}

// Event: NFTMinted
pub fn emit_nft_minted(env: &Env, token_id: u128, recipient: &Address) {
    env.events().publish(
        (Symbol::short("NFTMinted"), token_id),
        recipient.clone(),
    );
}

// Event: NFTBatchMinted
pub fn emit_nft_batch_minted(env: &Env, quantity: u32, recipient: &Address) {
    env.events().publish(
        (Symbol::short("NFTBatchMinted"), quantity),
        recipient.clone(),
    );
}

// Event: Transfer
pub fn emit_transfer(env: &Env, from: &Address, to: &Address, token_id: u128) {
    env.events().publish(
        (Symbol::short("Transfer"), from.clone(), to.clone()),
        token_id,
    );
}