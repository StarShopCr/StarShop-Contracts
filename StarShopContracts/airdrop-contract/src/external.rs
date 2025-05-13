use soroban_sdk::{contractclient, contracterror, Address, Symbol};

/// Error codes for provider contracts.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProviderError {
    InvalidUser = 1,
    MetricNotSupported = 2,
    InternalError = 3,
}

/// Interface for metric provider contracts (e.g., Referral, Subscription, Loyalty).
///
#[allow(dead_code)]
#[contractclient(name = "MetricProviderClient")]
pub trait MetricProvider {
    /// Returns the user's metric value for the given metric key.
    /// - `user`: The user's address.
    /// - `metric`: The metric key (e.g., "referrals", "is_subscribed").
    /// - Returns: `u64` value (numeric or 1/0 for booleans) or a `ProviderError`.
    fn get_user_metric(user: Address, metric: Symbol) -> Result<u64, ProviderError>;
}
