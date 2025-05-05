use crate::types::{DataKey, Error, PurchaseRecord};
use soroban_sdk::{Address, Env, Map, Vec};

pub struct TrackingManager;

impl TrackingManager {
    /// Initialize the tracking manager
    pub fn init(env: &Env) {
        // No initialization needed
    }

    /// Record a purchase
    pub fn record_purchase(
        env: &Env,
        buyer: &Address,
        drop_id: u32,
        quantity: u32,
        price_paid: i128,
    ) -> Result<(), Error> {
        let timestamp = env.ledger().timestamp();

        // Create purchase record
        let purchase = PurchaseRecord {
            drop_id,
            quantity,
            timestamp,
            price_paid,
        };

        // Get user's purchase history
        let mut user_purchases: Map<u32, PurchaseRecord> = env
            .storage()
            .instance()
            .get(&DataKey::UserPurchases(buyer.clone()))
            .unwrap_or_else(|| Map::new(env));

        // Add purchase to history
        user_purchases.set(drop_id, purchase);
        env.storage()
            .instance()
            .set(&DataKey::UserPurchases(buyer.clone()), &user_purchases);

        // Update drop's total purchases
        let mut total_purchases: u32 = env
            .storage()
            .instance()
            .get(&DataKey::DropPurchases(drop_id))
            .unwrap_or(0);
        total_purchases += quantity;
        env.storage()
            .instance()
            .set(&DataKey::DropPurchases(drop_id), &total_purchases);

        // Add buyer to drop's buyer list if not already there
        let mut buyers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::DropBuyers(drop_id))
            .unwrap_or_else(|| Vec::new(env));

        if !buyers.contains(buyer) {
            buyers.push_back(buyer.clone());
            env.storage()
                .instance()
                .set(&DataKey::DropBuyers(drop_id), &buyers);
        }

        Ok(())
    }

    /// Get purchase history for a user
    pub fn get_purchase_history(
        env: &Env,
        user: Address,
        drop_id: u32,
    ) -> Result<Vec<PurchaseRecord>, Error> {
        let user_purchases: Map<u32, PurchaseRecord> = env
            .storage()
            .instance()
            .get(&DataKey::UserPurchases(user))
            .unwrap_or_else(|| Map::new(env));

        let mut history = Vec::new(env);
        if let Some(purchase) = user_purchases.get(drop_id) {
            history.push_back(purchase);
        }

        Ok(history)
    }

    /// Get buyer list for a drop
    pub fn get_buyer_list(env: &Env, drop_id: u32) -> Result<Vec<Address>, Error> {
        let buyers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::DropBuyers(drop_id))
            .unwrap_or_else(|| Vec::new(env));

        Ok(buyers)
    }

    /// Get total purchases for a user in a drop
    pub fn get_user_purchases(env: &Env, user: &Address, drop_id: u32) -> u32 {
        let user_purchases: Map<u32, PurchaseRecord> = env
            .storage()
            .instance()
            .get(&DataKey::UserPurchases(user.clone()))
            .unwrap_or_else(|| Map::new(env));

        if let Some(purchase) = user_purchases.get(drop_id) {
            purchase.quantity
        } else {
            0
        }
    }
}
