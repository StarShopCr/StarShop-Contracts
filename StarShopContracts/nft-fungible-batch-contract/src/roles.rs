//! Role-based access control for Starshop NFT-Fungible Batch Distribution Contract

use soroban_sdk::{Env, Address, Symbol, symbol_short};
use crate::events;

// Role constants (32 bytes, as in OpenZeppelin)
pub const DEFAULT_ADMIN_ROLE: &[u8; 32] = b"DEFAULT_ADMIN_ROLE______________";
pub const MINTER_ROLE: &[u8; 32] = b"MINTER_ROLE_____________________";

/// Storage key prefix for roles
pub fn role_key(role: &[u8; 32], addr: &Address) -> (Symbol, Vec<u8>, Address) {
    (symbol_short!("role"), role.to_vec(), addr.clone())
}

/// Check if an address has a role
pub fn has_role(env: &Env, role: &[u8; 32], addr: &Address) -> bool {
    env.storage().instance().has(&role_key(role, addr))
}

/// Grant a role to an address (admin only)
pub fn grant_role(env: &Env, role: &[u8; 32], addr: &Address, sender: &Address) {
    // Only admin can grant roles
    if !has_role(env, DEFAULT_ADMIN_ROLE, sender) {
        panic!("AccessControl: sender must be admin to grant role");
    }
    env.storage().instance().set(&role_key(role, addr), &true);
    // Emit event
    events::emit_role_granted(env, role, addr, sender);
}

/// Revoke a role from an address (admin only)
pub fn revoke_role(env: &Env, role: &[u8; 32], addr: &Address, sender: &Address) {
    // Only admin can revoke roles
    if !has_role(env, DEFAULT_ADMIN_ROLE, sender) {
        panic!("AccessControl: sender must be admin to revoke role");
    }
    env.storage().instance().remove(&role_key(role, addr));
    // Emit event
    events::emit_role_revoked(env, role, addr, sender);
}
