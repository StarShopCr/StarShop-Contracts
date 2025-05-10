use crate::datatype::{
    DataKeys, EventLog, FollowCategory, FollowData, FollowError, NotificationPreferences,
    NotificationPriority,
};
use crate::interface::AlertOperations;
use soroban_sdk::{Address, Env, Vec};

pub struct AlertSystem;

impl AlertOperations for AlertSystem {
    fn check_price_change(env: Env, product_id: u128, _new_price: u64) -> Result<(), FollowError> {
        // Process notification for PriceChange events
        Self::process_alert(&env, product_id, FollowCategory::PriceChange)
    }

    fn check_restock(env: Env, product_id: u128) -> Result<(), FollowError> {
        // Process notification for Restock events
        Self::process_alert(&env, product_id, FollowCategory::Restock)
    }

    fn check_special_offer(env: Env, product_id: u128) -> Result<(), FollowError> {
        // Process notification for SpecialOffer events
        Self::process_alert(&env, product_id, FollowCategory::SpecialOffer)
    }
}

impl AlertSystem {
    // Helper function to get all users following a specific product
    fn get_users_following_product(
        env: &Env,
        product_id: u128,
    ) -> Result<Vec<Address>, FollowError> {
        let mut following_users = Vec::new(env);

        let all_users_key = DataKeys::AllUsers;
        let all_users: Vec<Address> = env
            .storage()
            .persistent()
            .get(&all_users_key)
            .unwrap_or_else(|| Vec::new(env));

        for user in all_users.iter() {
            let follow_list_key = DataKeys::FollowList(user.clone());
            if let Some(follows) = env
                .storage()
                .persistent()
                .get::<DataKeys, Vec<FollowData>>(&follow_list_key)
            {
                for follow in follows.iter() {
                    if u128::from(follow.product_id) == product_id {
                        following_users.push_back(user.clone());
                        break;
                    } else {
                    }
                }
            }
        }
        Ok(following_users)
    }

    // Rate limiting
    fn check_rate_limit(_env: &Env, _user: &Address) -> bool {
        #[cfg(test)]
        return true; // Disable rate limiting in tests

        #[cfg(not(test))]
        {
            let last_notification = _env
                .storage()
                .persistent()
                .get::<_, u64>(&DataKeys::LastNotification(_user.clone()))
                .unwrap_or(0);

            let current_time = _env.ledger().timestamp();
            current_time - last_notification > 3600
        }
    }

    fn log_event(env: &Env, user: Address, event: EventLog) -> Result<(), FollowError> {
        if !Self::check_rate_limit(env, &user) {
            return Ok(());
        }

        // Update the last notification timestamp
        env.storage().persistent().set(
            &DataKeys::LastNotification(user.clone()),
            &env.ledger().timestamp(),
        );

        let history_key = DataKeys::NotificationHistory(user.clone());
        let mut history: Vec<EventLog> = env
            .storage()
            .persistent()
            .get(&history_key)
            .unwrap_or_else(|| Vec::new(env));

        history.push_back(event);

        // Limit history size (optional)
        while history.len() > 100 {
            // Keep last 100 notifications
            history.remove(0);
        }

        env.storage().persistent().set(&history_key, &history);
        Ok(())
    }

    fn process_alert(
        env: &Env,
        product_id: u128,
        category: FollowCategory,
    ) -> Result<(), FollowError> {
        // Get all users who follow this product
        let users = Self::get_users_following_product(env, product_id)?;

        // Create an event for this alert
        let event = EventLog {
            product_id,
            event_type: category.clone(),
            triggered_at: env.ledger().timestamp(),
            priority: NotificationPriority::High,
        };

        // Log the event for each user who follows this product with the right category
        for user_address in users.iter() {
            // Check user's notification preferences
            let prefs_key = DataKeys::AlertSettings(user_address.clone());
            if let Some(prefs) = env
                .storage()
                .persistent()
                .get::<_, NotificationPreferences>(&prefs_key)
            {
                // Skip if notifications are muted
                if prefs.mute_notifications {
                    continue;
                }

                // Skip if user doesn't want notifications for this category
                if !prefs.categories.contains(&category) {
                    continue;
                }
            }

            let follow_key = DataKeys::FollowList(user_address.clone());
            let follows: Vec<FollowData> = env
                .storage()
                .persistent()
                .get(&follow_key)
                .unwrap_or_else(|| Vec::new(env));

            // Check if user is following for this category
            if follows
                .iter()
                .any(|f| u128::from(f.product_id) == product_id && f.categories.contains(&category))
            {
                Self::log_event(env, user_address.clone(), event.clone())?;
            }
        }

        Ok(())
    }
}
