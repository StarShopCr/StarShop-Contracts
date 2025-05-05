//! Interface documentation for the Limited-Time Drop Contract
//!
//! This contract manages limited-time drops of products with access control and purchase tracking.

use crate::types::{Drop, DropStatus, Error, PurchaseRecord, UserLevel};
use soroban_sdk::{Address, String, Vec};

/// Contract Interface
pub trait LimitedTimeDropContract {
    /// Initialize the contract with an admin address
    ///
    /// # Arguments
    /// * `admin` - The address that will have administrative privileges
    ///
    /// # Errors
    /// * `AlreadyInitialized` - If the contract has already been initialized
    fn initialize(admin: Address) -> Result<(), Error>;

    /// Create a new limited-time drop
    ///
    /// # Arguments
    /// * `creator` - The address creating the drop
    /// * `title` - The title of the drop
    /// * `product_id` - The ID of the product being dropped
    /// * `max_supply` - Maximum number of items available
    /// * `start_time` - Unix timestamp when the drop starts
    /// * `end_time` - Unix timestamp when the drop ends
    /// * `price` - Price per item
    /// * `per_user_limit` - Maximum items a single user can purchase
    /// * `image_uri` - URI to the product image
    ///
    /// # Returns
    /// The ID of the created drop
    ///
    /// # Errors
    /// * `InvalidTime` - If the time window is invalid
    /// * `InvalidPrice` - If the price is invalid
    fn create_drop(
        creator: Address,
        title: String,
        product_id: u64,
        max_supply: u32,
        start_time: u64,
        end_time: u64,
        price: i128,
        per_user_limit: u32,
        image_uri: String,
    ) -> Result<u32, Error>;

    /// Purchase items from a drop
    ///
    /// # Arguments
    /// * `buyer` - The address making the purchase
    /// * `drop_id` - The ID of the drop
    /// * `quantity` - Number of items to purchase
    ///
    /// # Errors
    /// * `DropNotFound` - If the drop doesn't exist
    /// * `DropNotActive` - If the drop is not active
    /// * `InsufficientSupply` - If there aren't enough items left
    /// * `UserLimitExceeded` - If the user has reached their purchase limit
    /// * `NotWhitelisted` - If the user is not whitelisted
    /// * `InsufficientLevel` - If the user's level is too low
    fn purchase(buyer: Address, drop_id: u32, quantity: u32) -> Result<(), Error>;

    /// Get details of a drop
    ///
    /// # Arguments
    /// * `drop_id` - The ID of the drop
    ///
    /// # Returns
    /// The drop details
    ///
    /// # Errors
    /// * `DropNotFound` - If the drop doesn't exist
    fn get_drop(drop_id: u32) -> Result<Drop, Error>;

    /// Get purchase history for a user
    ///
    /// # Arguments
    /// * `user` - The user's address
    /// * `drop_id` - The ID of the drop
    ///
    /// # Returns
    /// Vector of purchase records
    ///
    /// # Errors
    /// * `DropNotFound` - If the drop doesn't exist
    fn get_purchase_history(user: Address, drop_id: u32) -> Result<Vec<PurchaseRecord>, Error>;

    /// Get total purchases for a drop
    ///
    /// # Arguments
    /// * `drop_id` - The ID of the drop
    ///
    /// # Returns
    /// Total number of purchases
    ///
    /// # Errors
    /// * `DropNotFound` - If the drop doesn't exist
    fn get_drop_purchases(drop_id: u32) -> Result<u32, Error>;

    /// Get list of buyers for a drop
    ///
    /// # Arguments
    /// * `drop_id` - The ID of the drop
    ///
    /// # Returns
    /// Vector of buyer addresses
    ///
    /// # Errors
    /// * `DropNotFound` - If the drop doesn't exist
    fn get_buyer_list(drop_id: u32) -> Result<Vec<Address>, Error>;

    /// Update the status of a drop (Admin only)
    ///
    /// # Arguments
    /// * `admin` - The admin address
    /// * `drop_id` - The ID of the drop
    /// * `status` - The new status
    ///
    /// # Errors
    /// * `Unauthorized` - If the caller is not the admin
    /// * `DropNotFound` - If the drop doesn't exist
    fn update_status(admin: Address, drop_id: u32, status: DropStatus) -> Result<(), Error>;

    /// Add a user to the whitelist (Admin only)
    ///
    /// # Arguments
    /// * `admin` - The admin address
    /// * `user` - The user to whitelist
    ///
    /// # Errors
    /// * `Unauthorized` - If the caller is not the admin
    fn add_to_whitelist(admin: Address, user: Address) -> Result<(), Error>;

    /// Set a user's access level (Admin only)
    ///
    /// # Arguments
    /// * `admin` - The admin address
    /// * `user` - The user's address
    /// * `level` - The new access level
    ///
    /// # Errors
    /// * `Unauthorized` - If the caller is not the admin
    /// * `InvalidUserLevel` - If the level is invalid
    fn set_user_level(admin: Address, user: Address, level: UserLevel) -> Result<(), Error>;
}
