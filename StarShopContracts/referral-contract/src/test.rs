use super::*;
use crate::types::{MilestoneRequirement, UserLevel};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[cfg(test)]
mod test_setup {
    use super::*;

    pub fn setup_contract(e: &Env) -> (ReferralContractClient<'_>, Address, Address) {
        let admin = Address::generate(e);
        let token = e.register_stellar_asset_contract_v2(admin.clone());
        let contract_id = e.register(ReferralContract, ());
        let client = ReferralContractClient::new(e, &contract_id);

        e.mock_all_auths();

        // Initialize first
        let _ = client.initialize(&admin, &token.address());

        // Set default reward rates after initializationclea
        let rates = RewardRates {
            level1: 500, // 5%
            level2: 200, // 2%
            level3: 100, // 1%
            max_reward_per_referral: 1000000,
        };

        client.set_reward_rates(&rates);

        (client, admin, token.address())
    }
}

mod test_admin {
    use super::*;

    #[test]
    #[should_panic(expected = "Error(Contract, #2)")]
    fn test_initialization() {
        let env = Env::default();
        let (contract, admin, token_address) = test_setup::setup_contract(&env);

        // Verify contract is initialized correctly
        assert!(!contract.get_paused_state());

        env.mock_all_auths();
        // Try to initialize again (should fail)
        let _ = contract.initialize(&admin, &token_address);
    }

    #[test]
    fn test_pause_resume() {
        let env = Env::default();
        let (contract, _, _) = test_setup::setup_contract(&env);

        env.mock_all_auths();
        // Test pause
        contract.pause_contract();
        assert!(contract.get_paused_state());

        // Test resume
        contract.resume_contract();
        assert!(!contract.get_paused_state());
    }
}

mod test_verification {
    use super::*;

    #[test]
    fn test_verification_flow() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);
        let user = Address::generate(&env);

        env.mock_all_auths();

        // Register user with admin as referrer first
        contract.register_with_referral(&user, &admin, &String::from_str(&env, "proof123"));

        // Check pending status
        let status = contract.get_verification_status(&user);
        assert!(matches!(status, VerificationStatus::Pending));

        // Approve verification
        contract.approve_verification(&user);
        let status = contract.get_verification_status(&user);
        assert!(matches!(status, VerificationStatus::Verified));
    }
}

mod test_referral {
    use super::*;

    #[test]
    fn test_referral_registration() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);

        env.mock_all_auths();
        // First user referred by admin
        let user1 = Address::generate(&env);
        contract.register_with_referral(&user1, &admin, &String::from_str(&env, "proof1"));
        contract.approve_verification(&user1);

        env.mock_all_auths();
        // Second user referred by first user
        let user2 = Address::generate(&env);
        contract.register_with_referral(&user2, &user1, &String::from_str(&env, "proof2"));

        // Verify referral relationship
        let user_info = contract.get_user_info(&user2);
        assert_eq!(user_info.referrer, Some(user1));
    }

    #[test]
    fn test_team_size_update() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);

        env.mock_all_auths();
        // First user referred by admin
        let user1 = Address::generate(&env);
        contract.register_with_referral(&user1, &admin, &String::from_str(&env, "proof1"));
        contract.approve_verification(&user1);

        // Register multiple referrals under first user
        for _ in 0..3 {
            env.mock_all_auths();
            let user = Address::generate(&env);
            contract.register_with_referral(&user, &user1, &String::from_str(&env, "proof123"));
        }

        // Check team size
        let team_size = contract.get_team_size(&user1);
        assert_eq!(team_size, 3);
    }
}

mod test_rewards {
    use super::*;

    #[test]
    fn test_reward_distribution() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);

        env.mock_all_auths();
        // First user referred by admin
        let user1 = Address::generate(&env);
        contract.register_with_referral(&user1, &admin, &String::from_str(&env, "proof1"));
        contract.approve_verification(&user1);

        env.mock_all_auths();
        // Second user referred by first user
        let user2 = Address::generate(&env);
        contract.register_with_referral(&user2, &user1, &String::from_str(&env, "proof2"));
        contract.approve_verification(&user2);

        env.mock_all_auths();
        // Distribute rewards
        contract.distribute_rewards(&user2, &1000);

        // Check rewards
        let user1_rewards = contract.get_pending_rewards(&user1);
        assert_eq!(user1_rewards, 50); // 5% of 1000
    }

    #[test]
    fn test_milestone_achievement() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);

        env.mock_all_auths();
        // Add milestone
        let milestone = Milestone {
            required_level: UserLevel::Basic,
            requirement: MilestoneRequirement::DirectReferrals(2),
            reward_amount: 1000,
            description: String::from_str(&env, "First milestone"),
        };
        contract.add_milestone(&milestone);

        // Setup user with required referrals
        let user = Address::generate(&env);
        contract.register_with_referral(&user, &admin, &String::from_str(&env, "proof1"));
        contract.approve_verification(&user);

        // Add referrals
        for _ in 0..2 {
            let referral = Address::generate(&env);
            contract.register_with_referral(&referral, &user, &String::from_str(&env, "proof"))
        }

        // Check milestone
        contract.check_and_reward_milestone(&user);
        let user_rewards = contract.get_pending_rewards(&user);
        assert_eq!(user_rewards, 1000);
    }
}

mod test_levels {
    use super::*;

    #[test]
    fn test_level_progression() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);

        env.mock_all_auths();

        let new_requirements = LevelRequirements {
            silver: LevelCriteria {
                required_direct_referrals: 5,
                required_team_size: 5,
                required_total_rewards: 0,
            },
            gold: LevelCriteria {
                required_direct_referrals: 10,
                required_team_size: 50,
                required_total_rewards: 5000,
            },
            platinum: LevelCriteria {
                required_direct_referrals: 20,
                required_team_size: 100,
                required_total_rewards: 20000,
            },
        };
        contract.set_level_requirements(&new_requirements);

        // First user referred by admin
        let user = Address::generate(&env);
        contract.register_with_referral(&user, &admin, &String::from_str(&env, "proof1"));
        contract.approve_verification(&user);

        // Add enough referrals for Silver
        for _ in 0..6 {
            let referral = Address::generate(&env);
            contract.register_with_referral(&referral, &user, &String::from_str(&env, "proof1243"));
            contract.approve_verification(&referral);
        }

        // Check level upgrade
        let user_level = contract.get_user_level(&user);
        assert_eq!(user_level, UserLevel::Silver);
    }
}

mod test_metrics {
    use super::*;

    #[test]
    fn test_system_metrics() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);

        env.mock_all_auths();

        for _ in 0..3 {
            let user = Address::generate(&env);
            contract.register_with_referral(&user, &admin, &String::from_str(&env, "proof"));
            contract.approve_verification(&user);
        }

        // Check metrics
        let total_users = contract.get_total_users();
        assert_eq!(total_users, 4); // referrer + 3 referrals

        let conversion_rate = contract.get_referral_conversion_rate(&admin);
        assert_eq!(conversion_rate, 100); // All referrals are verified
    }
}
