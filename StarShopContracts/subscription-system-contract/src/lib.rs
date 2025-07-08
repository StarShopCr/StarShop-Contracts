#![no_std]

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, Env, String, Symbol};

mod plans;
mod minting;
mod expiration;
mod access;

#[cfg(test)]
mod test;

use crate::plans::{
    create_plan, get_plan, update_plan, disable_plan, Plan,
};
use crate::minting::{mint_subscription, renew_subscription, is_active, is_expired};
use crate::expiration::{
    is_in_grace_period, cleanup_expired, get_subscription_state, SubscriptionState,
};
use crate::access::{
    view_premium_content,
    access_gold_feature,
    admin_reset_subscription,
    get_usage_count,
    add_role,
};

#[contract]
pub struct SubscriptionContract;

#[contractimpl]
impl SubscriptionContract {
    // -----------------------
    // 📦 PLAN MANAGEMENT
    // -----------------------

    pub fn create_plan(
        env: Env,
        admin: Address,
        plan_id: Symbol,
        name: Bytes,
        duration: u64,
        price: u64,
        benefits: Bytes,
        version: u32,
        tier: Symbol,
    ) {
        let admin_address = admin.clone();
        let plan_id_clone = plan_id.clone();
        create_plan(
            &env,
            admin,
            plan_id,
            name,
            duration,
            price,
            benefits,
            version,
            tier,
        );

        env.events().publish(
            (symbol_short!("CREATE"), &admin_address, &plan_id_clone),
            (),
        );
    }

    pub fn update_plan(
        env: Env,
        admin: Address,
        plan_id: Symbol,
        name: Bytes,
        duration: u64,
        price: u64,
        benefits: Bytes,
        version: u32,
        tier: Symbol,
    ) {
        let admin_clone = admin.clone();
        let plan_id_clone = plan_id.clone();
        update_plan(
            &env,
            admin,
            plan_id,
            name,
            duration,
            price,
            benefits,
            version,
            tier,
        );
        env.events().publish(
            (symbol_short!("UPDATE"), &admin_clone, &plan_id_clone),
            (),
        );
    }

    pub fn disable_plan(env: Env, admin: Address, plan_id: Symbol) {
            let admin_clone = admin.clone();
            let plan_id_clone = plan_id.clone();
            disable_plan(&env, admin, plan_id);
            env.events().publish(
                (symbol_short!("DISABLE"), &admin_clone, &plan_id_clone),
                (),
            );
        }

    pub fn get_plan(env: Env, plan_id: Symbol) -> Option<Plan> {
        get_plan(&env, plan_id)
    }

    // -----------------------
    // 🪙 SUBSCRIPTION MINTING
    // -----------------------

    pub fn subscribe(env: Env, user: Address, plan_id: Symbol) {
        let user_clone = user.clone();
        let plan_id_clone = plan_id.clone();
        mint_subscription(&env, user, plan_id);
        env.events().publish(
            (symbol_short!("SUBSCRIBE"), &user_clone, &plan_id_clone),
            (),
        );
    }

    pub fn renew(env: Env, user: Address, plan_id: Symbol) {
        let user_clone = user.clone();
        let plan_id_clone = plan_id.clone();
        renew_subscription(&env, user, plan_id);
        env.events().publish(
            (symbol_short!("RENEW"), &user_clone, &plan_id_clone),
            (),
        );
    }

    // -----------------------
    // 🔐 ACCESS VERIFICATION
    // -----------------------

    pub fn is_active_sub(env: Env, user: Address, plan_id: Symbol) -> bool {
        is_active(&env, user, plan_id)
    }

    pub fn is_expired_sub(env: Env, user: Address, plan_id: Symbol) -> bool {
        is_expired(&env, user, plan_id)
    }

    pub fn is_in_grace(env: Env, user: Address, plan_id: Symbol) -> bool {
        is_in_grace_period(&env, user, plan_id)
    }

    pub fn get_state(env: Env, user: Address, plan_id: Symbol) -> SubscriptionState {
        get_subscription_state(&env, user, plan_id)
    }

    // -----------------------
    // 🎯 FEATURE ACCESS
    // -----------------------

    pub fn premium_content(env: Env, user: Address, plan_id: Symbol) -> String {
        view_premium_content(&env, user, plan_id)
    }

    pub fn gold_feature(env: Env, user: Address, plan_id: Symbol) -> Symbol {
        access_gold_feature(&env, user, plan_id)
    }

    // -----------------------
    // 🔧 ADMIN/ROLE MANAGEMENT
    // -----------------------

    pub fn add_user_role(env: Env, role: Symbol, user: Address) {
        add_role(&env, role.clone(), user.clone());
        env.events().publish(
            (symbol_short!("ADD_ROLE"), &role, &user),
            (),
        );
    }

    pub fn reset_subscription(env: Env, admin: Address, target_user: Address, plan_id: Symbol) {
        admin_reset_subscription(&env, admin.clone(), target_user.clone(), plan_id.clone());
        env.events().publish(
            (symbol_short!("RESET_SUB"), &admin, &target_user, &plan_id),
            (),
        );
    }

    pub fn get_feature_usage(env: Env, user: Address, feature: Symbol) -> u32 {
        get_usage_count(&env, user, feature)
    }

    // -----------------------
    // 🧹 EXPIRATION CLEANUP
    // -----------------------

    pub fn cleanup(env: Env, user: Address, plan_id: Symbol) -> bool {
        let res = cleanup_expired(&env, user.clone(), plan_id.clone());
        env.events().publish(
            (symbol_short!("CLEANUP"), &user, &plan_id),
            (),
        );
        res
    }
}
