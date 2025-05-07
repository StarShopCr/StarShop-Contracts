use soroban_sdk::{contracttype, Address, Env, Symbol};

use crate::minting::{MintKey, SubscriptionNFT};

/// Grace period duration (e.g., 3 days in seconds)
const GRACE_PERIOD_SECS: u64 = 3 * 24 * 60 * 60;

/// Check if user is in grace period after expiry
pub fn is_in_grace_period(env: &Env, user: Address, plan_id: Symbol) -> bool {
    let sub_key = MintKey::Subscription(user.clone(), plan_id.clone());

    if let Some(nft) = env.storage().instance().get::<_, SubscriptionNFT>(&sub_key) {
        let now = env.ledger().timestamp();
        now >= nft.expiry_time && now < (nft.expiry_time + GRACE_PERIOD_SECS)
    } else {
        false
    }
}

/// Cleanup expired subscriptions that have passed grace period
/// Can be run by admin or keeper bot
pub fn cleanup_expired(env: &Env, user: Address, plan_id: Symbol) -> bool {
    let sub_key = MintKey::Subscription(user.clone(), plan_id.clone());

    if let Some(nft) = env.storage().instance().get::<_, SubscriptionNFT>(&sub_key) {
        let now = env.ledger().timestamp();

        // If beyond grace period, remove subscription
        if now >= (nft.expiry_time + GRACE_PERIOD_SECS) {
            env.storage().instance().remove(&sub_key);
            return true;
        }
    }

    false
}

/// Optional: Check exact state of a subscription lifecycle
#[contracttype]
pub enum SubscriptionState {
    Active,
    Grace,
    Expired,
    NotFound,
}

/// Determine current lifecycle state of a user's subscription
pub fn get_subscription_state(env: &Env, user: Address, plan_id: Symbol) -> SubscriptionState {
    let sub_key = MintKey::Subscription(user.clone(), plan_id.clone());

    if let Some(nft) = env.storage().instance().get::<_, SubscriptionNFT>(&sub_key) {
        let now = env.ledger().timestamp();

        if now < nft.expiry_time {
            SubscriptionState::Active
        } else if now < nft.expiry_time + GRACE_PERIOD_SECS {
            SubscriptionState::Grace
        } else {
            SubscriptionState::Expired
        }
    } else {
        SubscriptionState::NotFound
    }
}
