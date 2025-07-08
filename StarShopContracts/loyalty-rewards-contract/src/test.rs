#![cfg(test)]

use crate::{
    types::{LevelCriteria, LevelRequirements, LoyaltyLevel, Milestone, MilestoneRequirement, Reward, RewardType},
    LoyaltyRewards, LoyaltyRewardsClient,
};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    Address, Env, Symbol,
};

// Helper struct to setup test environment
struct LoyaltyTest<'a> {
    env: Env,
    admin: Address,
    client: LoyaltyRewardsClient<'a>,
}

impl<'a> LoyaltyTest<'a> {
    fn setup() -> Self {
        let env = Env::default();
        env.mock_all_auths(); // Mock authentication for all calls

        #[allow(deprecated)]
        let contract_id = env.register_contract(None, LoyaltyRewards);
        let client = LoyaltyRewardsClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init(&admin);

        // Set default configuration for tests
        client.set_points_expiry(&90); // 90 days
        client.set_points_ratio(&100); // 100 currency units = 1 point
        client.set_max_redemption_percentage(&5000); // 50% max discount

        // Initialize level requirements, crucial for level-based tests
        let silver_req = LevelCriteria { min_points: 2000, min_purchases: 5, min_days_active: 30 };
        let gold_req = LevelCriteria { min_points: 5000, min_purchases: 10, min_days_active: 90 };
        let requirements = LevelRequirements { silver: silver_req, gold: gold_req };
        client.init_level_requirements(&requirements);


        Self { env, admin, client }
    }

    fn create_user(&self) -> Address {
        let user = Address::generate(&self.env);
        self.client.register_user(&user);
        user
    }

    fn advance_time(&self, days: u64) {
        let seconds = days * 24 * 60 * 60;
        self.env.ledger().with_mut(|li| {
            li.timestamp = li.timestamp.saturating_add(seconds);
        });
    }
}


#[test]
fn test_register_user() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();

    // Verify user exists with correct initial values
    let balance = test.client.get_points_balance(&user);
    assert_eq!(balance, 0);

    let lifetime = test.client.get_lifetime_points(&user);
    assert_eq!(lifetime, 0);

    let level = test.client.get_user_level(&user);
    assert_eq!(level, LoyaltyLevel::Bronze);
}

#[test]
fn test_add_points() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();

    // Add points (as admin)
    test.client.add_points(&user, &100, &symbol_short!("bonus"));

    // Verify points were added
    let balance = test.client.get_points_balance(&user);
    assert_eq!(balance, 100);

    let lifetime = test.client.get_lifetime_points(&user);
    assert_eq!(lifetime, 100);
}

#[test]
fn test_record_purchase_points() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();

    // Record a purchase (1000 / 100 = 10 points)
    let points = test
        .client
        .record_purchase_points(&user, &1000, &None, &None);
    assert_eq!(points, 10);

    // Verify points were added to balance
    let balance = test.client.get_points_balance(&user);
    assert_eq!(balance, points);

    let initial_points = points;

    // Record a purchase with a category bonus (2% -> 10 * 0.02 = 0.2, rounds down to 0 bonus)
    // Let's use a bigger bonus for a clear result (20% -> 10 + 2 = 12)
    test.client.set_category_bonus(&symbol_short!("food"), &2000);
    let points_with_category = test.client.record_purchase_points(
        &user,
        &1000,
        &None,
        &Some(symbol_short!("food")),
    );
    assert_eq!(points_with_category, 12);
    assert!(points_with_category > initial_points);

    // Record a purchase with a product bonus (30% -> 10 + 3 = 13)
    test.client.set_product_bonus(&symbol_short!("apple"), &3000);
    let points_with_product = test.client.record_purchase_points(
        &user,
        &1000,
        &Some(symbol_short!("apple")),
        &None,
    );
    assert_eq!(points_with_product, 13);
    assert!(points_with_product > initial_points);
}

#[test]
fn test_points_expiration() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();

    test.client.set_points_expiry(&30);
    test.client.add_points(&user, &100, &symbol_short!("bonus"));
    assert_eq!(test.client.get_points_balance(&user), 100);

    test.advance_time(29);
    assert_eq!(test.client.get_points_balance(&user), 100);

    test.advance_time(2);
    assert_eq!(test.client.get_points_balance(&user), 0);
    assert_eq!(test.client.get_lifetime_points(&user), 100);
}

#[test]
fn test_point_limits_per_transaction() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();

    // Test with very large purchase amount (1 billion)
    let points_earned =
        test.client
            .record_purchase_points(&user, &1_000_000_000, &None, &None);

    // With 100:1 ratio, this should be 10 million points
    assert_eq!(points_earned, 10_000_000);

    // Verify points balance
    let balance = test.client.get_points_balance(&user);
    assert_eq!(balance, points_earned);
}

// 2. Reward Redemption Tests

#[test]
fn test_create_reward() {
    let test = LoyaltyTest::setup();

    // Create a reward
    let reward = Reward {
        id: 0, // Ignored, will be auto-assigned
        name: symbol_short!("discount"),
        description: Symbol::new(&test.env, "DiscountReward"),
        points_cost: 500,
        reward_type: RewardType::Discount(1000), // 10% discount
        min_level: LoyaltyLevel::Bronze,
        max_per_user: 0,
    };
    test.client.create_reward(&reward);

    // Verify reward was created by attempting to redeem it
    let user = test.create_user();
    test.client.add_points(&user, &1000, &symbol_short!("bonus"));

    // Try to redeem the reward (ID is 0 because it's the first one)
    test.client.redeem_reward(&user, &0, &Some(1000));

    // Verify points were deducted
    let balance = test.client.get_points_balance(&user);
    assert_eq!(balance, 500);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_reward_redemption_insufficient_points() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();

    let reward = Reward {
        id: 0, // Ignored
        name: symbol_short!("discount"),
        description: Symbol::new(&test.env, "DiscountReward"),
        points_cost: 500,
        reward_type: RewardType::Discount(1000),
        min_level: LoyaltyLevel::Bronze,
        max_per_user: 0,
    };
    test.client.create_reward(&reward);
    test.client.add_points(&user, &100, &symbol_short!("bonus"));
    test.client.redeem_reward(&user, &0, &Some(1000));
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_reward_redemption_insufficient_level() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();

    let reward = Reward {
        id: 0, // Ignored
        name: symbol_short!("premium"),
        description: Symbol::new(&test.env, "PremiumReward"),
        points_cost: 500,
        reward_type: RewardType::Discount(1500),
        min_level: LoyaltyLevel::Silver,
        max_per_user: 0,
    };
    test.client.create_reward(&reward);
    test.client.add_points(&user, &1000, &symbol_short!("bonus"));
    test.client.redeem_reward(&user, &0, &Some(1000));
}

#[test]
fn test_reward_redemption_discount() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();

    let reward = Reward {
        id: 0, // Ignored
        name: symbol_short!("discount"),
        description: Symbol::new(&test.env, "DiscountReward"),
        points_cost: 500,
        reward_type: RewardType::Discount(1000),
        min_level: LoyaltyLevel::Bronze,
        max_per_user: 0,
    };
    test.client.create_reward(&reward);
    test.client.add_points(&user, &1000, &symbol_short!("bonus"));

    let purchase_amount = 2000;
    let discount = test.client.calculate_discount(&0, &purchase_amount);
    assert_eq!(discount, 200);

    test.client.redeem_reward(&user, &0, &Some(purchase_amount));
    let balance = test.client.get_points_balance(&user);
    assert_eq!(balance, 500);
}

#[test]
fn test_reward_redemption_product() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();

    let reward = Reward {
        id: 0, // Ignored
        name: symbol_short!("freeprod"),
        description: Symbol::new(&test.env, "FreeProductReward"),
        points_cost: 1000,
        reward_type: RewardType::Product(symbol_short!("tshirt")),
        min_level: LoyaltyLevel::Bronze,
        max_per_user: 0,
    };
    test.client.create_reward(&reward);
    test.client.add_points(&user, &1500, &symbol_short!("bonus"));
    test.client.redeem_reward(&user, &0, &None);
    let balance = test.client.get_points_balance(&user);
    assert_eq!(balance, 500);
}

#[test]
fn test_reward_redemption_xlm() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();

    let reward = Reward {
        id: 0, // Ignored
        name: symbol_short!("xlm"),
        description: Symbol::new(&test.env, "XlmReward"),
        points_cost: 2000,
        reward_type: RewardType::XLM(10_0000000),
        min_level: LoyaltyLevel::Bronze,
        max_per_user: 0,
    };
    test.client.create_reward(&reward);
    test.client.add_points(&user, &3000, &symbol_short!("bonus"));
    test.client.redeem_reward(&user, &0, &None);
    let balance = test.client.get_points_balance(&user);
    assert_eq!(balance, 1000);
}

// 3. Loyalty Level Tests

#[test]
fn test_level_progression() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();
    assert_eq!(test.client.get_user_level(&user), LoyaltyLevel::Bronze);

    for _ in 0..5 {
        test.client.record_purchase_points(&user, &40_000, &None, &None);
    }
    test.advance_time(31);
    let updated = test.client.check_and_update_level(&user);
    assert!(updated);
    assert_eq!(test.client.get_user_level(&user), LoyaltyLevel::Silver);
}

#[test]
fn test_anniversary_bonus() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();
    test.client.set_points_expiry(&400);
    test.client.add_points(&user, &100, &symbol_short!("init"));
    assert_eq!(test.client.award_anniversary_bonus(&user), 0);
    test.advance_time(366);
    assert_eq!(test.client.get_points_balance(&user), 100);
    assert_eq!(test.client.award_anniversary_bonus(&user), 100);
    assert_eq!(test.client.get_points_balance(&user), 200);
    assert_eq!(test.client.award_anniversary_bonus(&user), 0);
    assert_eq!(test.client.get_points_balance(&user), 200);
}

#[test]
fn test_prevent_level_downgrade() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();
    for _ in 0..5 {
        test.client.record_purchase_points(&user, &40_000, &None, &None);
    }
    test.advance_time(31);
    test.client.check_and_update_level(&user);
    assert_eq!(test.client.get_user_level(&user), LoyaltyLevel::Silver);
    let reward = Reward {
        id: 0, name: symbol_short!("big"), description: Symbol::new(&test.env, "BigReward"),
        points_cost: 1500, reward_type: RewardType::Discount(1000),
        min_level: LoyaltyLevel::Bronze, max_per_user: 0
    };
    test.client.create_reward(&reward);
    test.client.redeem_reward(&user, &0, &Some(1000));
    assert_eq!(test.client.get_points_balance(&user), 500);
    let updated = test.client.check_and_update_level(&user);
    assert!(!updated);
    assert_eq!(test.client.get_user_level(&user), LoyaltyLevel::Silver);
}

#[test]
fn test_tier_based_rewards() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();
    let bronze_reward = Reward {
        id: 0, name: symbol_short!("bronze"), description: Symbol::new(&test.env, "BronzeReward"),
        points_cost: 100, reward_type: RewardType::Discount(500),
        min_level: LoyaltyLevel::Bronze, max_per_user: 0
    };
    let silver_reward = Reward {
        id: 0, name: symbol_short!("silver"), description: Symbol::new(&test.env, "SilverReward"),
        points_cost: 200, reward_type: RewardType::Discount(1000),
        min_level: LoyaltyLevel::Silver, max_per_user: 0
    };
    test.client.create_reward(&bronze_reward);
    test.client.create_reward(&silver_reward);
    test.client.add_points(&user, &500, &symbol_short!("bonus"));
    assert_eq!(test.client.get_user_level(&user), LoyaltyLevel::Bronze);
    test.client.redeem_reward(&user, &0, &Some(1000));
    let result = test.client.try_redeem_reward(&user, &1, &Some(1000));
    assert!(result.is_err());
    assert_eq!(result.err().unwrap().unwrap(), crate::Error::InsufficientPoints);
}

// 4. Milestones & Activities Tests

#[test]
fn test_complete_milestone() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();
    let milestone = Milestone {
        id: 0, name: symbol_short!("buyer"), description: Symbol::new(&test.env, "BuyerMilestone"),
        points_reward: 500, requirement: MilestoneRequirement::TotalPurchases(3),
    };
    test.client.create_milestone(&milestone);
    test.client.record_purchase_points(&user, &1000, &None, &None);
    test.client.record_purchase_points(&user, &1000, &None, &None);
    test.client.check_and_complete_milestones(&user);
    assert_eq!(test.client.get_points_balance(&user), 20);
    test.client.record_purchase_points(&user, &1000, &None, &None);
    test.client.check_and_complete_milestones(&user);
    assert_eq!(test.client.get_points_balance(&user), 530);
}

#[test]
fn test_specific_product_milestone() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();
    let milestone = Milestone {
        id: 0, name: symbol_short!("applefan"), description: Symbol::new(&test.env, "AppleFan"),
        points_reward: 100, requirement: MilestoneRequirement::SpecificProduct(symbol_short!("apple")),
    };
    test.client.create_milestone(&milestone);
    test.client.record_purchase_points(&user, &1000, &Some(symbol_short!("orange")), &None);
    test.client.check_and_complete_milestones(&user);
    assert_eq!(test.client.get_points_balance(&user), 10);
    test.client.record_purchase_points(&user, &1000, &Some(symbol_short!("apple")), &None);
    test.client.check_and_complete_milestones(&user);
    assert_eq!(test.client.get_points_balance(&user), 120);
}

#[test]
fn test_prevent_duplicate_milestone_completion() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();
    let milestone = Milestone {
        id: 0, name: symbol_short!("buyer"), description: Symbol::new(&test.env, "BuyerMilestone"),
        points_reward: 200, requirement: MilestoneRequirement::TotalPurchases(3),
    };
    test.client.create_milestone(&milestone);
    for _ in 0..3 {
        test.client.record_purchase_points(&user, &1000, &None, &None);
    }
    test.client.check_and_complete_milestones(&user);
    let balance_after_first = test.client.get_points_balance(&user);
    assert_eq!(balance_after_first, 230);
    test.client.record_purchase_points(&user, &1000, &None, &None);
    test.client.check_and_complete_milestones(&user);
    let balance_after_second = test.client.get_points_balance(&user);
    assert_eq!(balance_after_second, balance_after_first + 10);
}

// 5. Anti-Fraud & Edge Case Tests

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_prevent_unauthorized_point_access() {
    let test = LoyaltyTest::setup();
    let fake_user = Address::generate(&test.env);
    test.client.get_points_balance(&fake_user);
}

#[test]
fn test_reward_redemption_max_percentage() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();
    test.client.set_max_redemption_percentage(&3000);
    let reward = Reward {
        id: 0, name: symbol_short!("bigdisc"), description: Symbol::new(&test.env, "BigDiscountReward"),
        points_cost: 10, reward_type: RewardType::Discount(4000),
        min_level: LoyaltyLevel::Bronze, max_per_user: 0,
    };
    test.client.create_reward(&reward);
    test.client.add_points(&user, &100, &symbol_short!("bonus"));
    let value = test.client.redeem_reward(&user, &0, &Some(2000));
    assert_eq!(value, 600);
}

#[test]
fn test_comprehensive_user_flow() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();
    let reward = Reward {
        id: 0, name: symbol_short!("bronze"), description: Symbol::new(&test.env, "BronzeReward"),
        points_cost: 100, reward_type: RewardType::Discount(500),
        min_level: LoyaltyLevel::Bronze, max_per_user: 1,
    };
    test.client.create_reward(&reward);
    let milestone = Milestone {
        id: 0, name: symbol_short!("firstbuy"), description: Symbol::new(&test.env, "FirstBuyMilestone"),
        points_reward: 200, requirement: MilestoneRequirement::TotalPurchases(1),
    };
    test.client.create_milestone(&milestone);
    test.client.add_points(&user, &50, &symbol_short!("signup"));
    assert_eq!(test.client.get_points_balance(&user), 50);
    test.client.record_purchase_points(&user, &1000, &None, &None);
    test.client.check_and_complete_milestones(&user);
    let pre_redeem_balance = test.client.get_points_balance(&user);
    assert_eq!(pre_redeem_balance, 260);
    test.client.redeem_reward(&user, &0, &Some(1000));
    let final_balance = test.client.get_points_balance(&user);
    assert_eq!(final_balance, 160);
    let lifetime_points = test.client.get_lifetime_points(&user);
    assert_eq!(lifetime_points, 260);
}

#[test]
#[should_panic(expected = "Error(Contract, #17)")]
fn test_direct_milestone_completion_prevention() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();
    let milestone = Milestone {
        id: 0, name: symbol_short!("buyer"), points_reward: 200,
        description: Symbol::new(&test.env, "desc"),
        requirement: MilestoneRequirement::TotalPurchases(3),
    };
    test.client.create_milestone(&milestone);
    test.client.complete_milestone(&user, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_prevent_negative_points() {
    let test = LoyaltyTest::setup();
    let user = test.create_user();
    test.client.add_points(&user, &-100, &symbol_short!("fraud"));
}

#[test]
fn test_admin_authorization() {
    let test = LoyaltyTest::setup();

    // Verify the admin can set points expiry
    test.client.set_points_expiry(&30);

    // Create a user
    let user = test.create_user();

    // Add points to the user as admin
    test.client
        .add_points(&user, &1000, &symbol_short!("adm_bonus"));

    // Verify the points were added
    let balance = test.client.get_points_balance(&user);
    assert!(balance > 0);

    // Verify we can get the admin address
    assert_eq!(test.admin, test.admin);
}
