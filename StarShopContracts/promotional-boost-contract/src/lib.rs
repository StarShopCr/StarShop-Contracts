#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec};

mod payments;
mod slots;
mod visibility;

use payments::PaymentProcessor;
use slots::SlotManager;
use visibility::VisibilityManager;

#[contract]
pub struct PromotionBoostContract;

#[contractimpl]
impl PromotionBoostContract {
    /// Seller calls this to boost a product
    pub fn boost_product(
        env: Env,
        seller_address: Address,
        category: Symbol,
        product_id: u64,
        duration_secs: u64,
        payment_amount: i128,
    ) {
        let now = env.ledger().timestamp();

        let seller_address_clone = seller_address.clone();

        // 1. Calculate required price
        let required_price = PaymentProcessor::calculate_price(duration_secs);
        if payment_amount < required_price {
            panic!("Insufficient payment for duration");
        }

        // 2. Collect XLM from seller
        PaymentProcessor::collect_payment(&env, &seller_address, payment_amount, required_price)
            .expect("XLM payment failed");

        // 3. Access or create slot manager
        let slot_id = env.ledger().timestamp(); // Use timestamp as unique ID
        let mut slot_manager = SlotManager::load_or_default(&env);

        let slot_result = slot_manager.add_slot(
            &env,
            slot_id,
            product_id,
            seller_address.clone(),
            category.clone(),
            duration_secs,
            payment_amount.try_into().expect("Amount conversion failed"),
            now,
        );

        // 4. Emit event for slot added
        env.events().publish(
            (
                Symbol::new(&env, "boost_slot_added"),
                seller_address.clone(),
            ),
            (
                slot_id,
                category.clone(),
                product_id,
                duration_secs,
                payment_amount,
            ),
        );

        // 5. Refund the replaced seller if a slot was evicted
        if let Some(replaced_slot_id) = slot_result {
            if let Some(replaced_slot) = slot_manager.get_slot(replaced_slot_id) {
                PaymentProcessor::refund_payment(
                    &env,
                    &replaced_slot.seller,
                    replaced_slot.price_paid.into(),
                )
                .expect("Refund failed");

                // Emit event for replaced slot
                env.events().publish(
                    (
                        Symbol::new(&env, "boost_slot_replaced"),
                        replaced_slot.seller.clone(),
                    ),
                    (replaced_slot.product_id, replaced_slot.price_paid),
                );
            }
        }

        // 6. Save updated slot state
        slot_manager.save(&env);

        // 7. Update visibility logic
        let mut visibility = VisibilityManager::load_or_default(&env);
        visibility.flag_product_as_boosted(
            product_id,
            seller_address,
            now,
            duration_secs,
            payment_amount.try_into().expect("Amount conversion failed"),
        );
        visibility.remove_expired(now);
        visibility.save(&env);

        // 8. Emit event for visibility change
        env.events().publish(
            (
                Symbol::new(&env, "visibility_boosted"),
                seller_address_clone,
            ),
            (product_id, category),
        );
    }

    /// View if a product is currently boosted
    pub fn is_boosted(env: Env, product_id: u64) -> bool {
        let now = env.ledger().timestamp();
        let visibility = VisibilityManager::load_or_default(&env);
        visibility.is_boosted(product_id, now)
    }

    /// Return all active boosted product IDs
    pub fn get_boosted_list(env: Env) -> Vec<u64> {
        let now = env.ledger().timestamp();
        let visibility = VisibilityManager::load_or_default(&env);
        visibility.get_active_boosts(now)
    }

    /// Admin clears expired slots (optional)
    pub fn cleanup_expired(env: Env, category: Symbol) {
        let now = env.ledger().timestamp();
        let mut slot_manager = SlotManager::load_or_default(&env);
        slot_manager.remove_expired_slots(&env, category, now);
        slot_manager.save(&env);

        let mut visibility = VisibilityManager::load_or_default(&env);
        visibility.remove_expired(now);
        visibility.save(&env);
    }
}
