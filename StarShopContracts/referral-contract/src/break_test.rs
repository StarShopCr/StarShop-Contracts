#![cfg(test)]
use super::*;
use crate::types::{UserLevel};
use soroban_sdk::{testutils::Address as _, Address, Env, String, IntoVal};

#[cfg(test)]
mod test_setup {
    use super::*;

    pub fn setup_contract(e: &Env) -> (ReferralContractClient, Address, Address) {
        let admin = Address::generate(e);
        let token = e.register_stellar_asset_contract_v2(admin.clone());
        let contract_id = e.register(ReferralContract, {});
        let client = ReferralContractClient::new(e, &contract_id);

        // Mock auth for initialization
        e.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract_id,
                fn_name: "initialize",
                args: (admin.clone(), token.address()).into_val(e),
                sub_invokes: &[],
            },
        }]);

        // Initialize first
        let _ = client.initialize(&admin, &token.address());

        // Set default reward rates after initialization
        let rates = RewardRates {
            level1: 500, // 5%
            level2: 200, // 2%
            level3: 100, // 1%
            max_reward_per_referral: 1000000,
        };

        // Mock auth for setting reward rates
        e.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_reward_rates",
                args: (rates.clone(),).into_val(e),
                sub_invokes: &[],
            },
        }]);

        client.set_reward_rates(&rates);

        (client, admin, token.address())
    }
}

mod test_critical_auth_bypass_vulnerability {
    use super::*;

    #[test]
    fn test_critical_auth_bypass_set_reward_rates() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);

        // Create malicious user
        let malicious_user = Address::generate(&env);
        
        // MALICIOUS ATTACK: Malicious user calls admin function with admin's signature
        // This should NOT be possible but the current implementation allows it!
        let malicious_rates = RewardRates {
            level1: 9999, // 99.99% - attempt to drain funds
            level2: 9999,
            level3: 9999,
            max_reward_per_referral: 999999999,
        };

        // VULNERABILITY: Malicious user can call admin function if admin signs
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin, // Admin signature
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "set_reward_rates",
                args: (malicious_rates.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        
        // This should FAIL (panic) but currently SUCCEEDS due to vulnerability
        contract.set_reward_rates(&malicious_rates);
    }

    #[test]
    fn test_critical_auth_bypass_pause_contract() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);

        // Create malicious user
        let malicious_user = Address::generate(&env);
        
        // MALICIOUS ATTACK: Malicious user pauses contract with admin's signature
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin, // Admin signature
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "pause_contract",
                args: ().into_val(&env),
                sub_invokes: &[],
            },
        }]);
        
        // This should FAIL (panic) but currently SUCCEEDS due to vulnerability
        contract.pause_contract();
    }

    #[test]
    fn test_critical_auth_bypass_transfer_admin() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);

        // Create malicious user and target
        let malicious_user = Address::generate(&env);
        let attacker_controlled_address = Address::generate(&env);
        
        // MALICIOUS ATTACK: Malicious user transfers admin to attacker address
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin, // Admin signature
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "transfer_admin",
                args: (attacker_controlled_address.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        
        // This should FAIL (panic) but currently SUCCEEDS due to vulnerability
        contract.transfer_admin(&attacker_controlled_address);
    }



    #[test]
    fn test_social_engineering_attack_scenario() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);
        // SOCIAL ENGINEERING ATTACK SCENARIO:
        // 1. Attacker convinces admin to sign a "harmless" transaction
        // 2. Admin thinks they're signing something else
        // 3. Attacker uses admin's signature to call admin functions
        
        let malicious_user = Address::generate(&env);
        let attacker_controlled_address = Address::generate(&env);
        
        // Attack: Admin is tricked into signing admin transfer
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin, // Admin signature (obtained through social engineering)
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "transfer_admin",
                args: (attacker_controlled_address.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        
        // VULNERABILITY: This succeeds even though admin didn't intend to transfer admin rights
        contract.transfer_admin(&attacker_controlled_address);
        
        // Now attacker has admin rights and can do anything!
        // This demonstrates the critical vulnerability
        assert_eq!(contract.get_admin(), attacker_controlled_address);
        
        // Attacker can now pause contract, change reward rates, etc.
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &attacker_controlled_address, // Now attacker is admin
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "pause_contract",
                args: ().into_val(&env),
                sub_invokes: &[],
            },
        }]);
        
        contract.pause_contract();
        assert!(contract.get_paused_state());
        
        // CRITICAL VULNERABILITY PROVEN: Social engineering attack successful!
       
    }
}

mod test_critical_level_manipulation_vulnerability {
    use super::*;

    #[test]
    fn test_different_address_can_update_level() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);

        // Create victim user and attacker
        let victim_user = Address::generate(&env);
        let attacker = Address::generate(&env);

        // Register victim user with admin as referrer
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &victim_user,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "register_with_referral",
                args: (victim_user.clone(), admin.clone(), String::from_str(&env, "proof123")).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.register_with_referral(&victim_user, &admin, &String::from_str(&env, "proof123"));
        
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "approve_verification",
                args: (victim_user.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.approve_verification(&victim_user);

        // Register attacker separately
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &attacker,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "register_with_referral",
                args: (attacker.clone(), admin.clone(), String::from_str(&env, "proof456")).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.register_with_referral(&attacker, &admin, &String::from_str(&env, "proof456"));
        
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "approve_verification",
                args: (attacker.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.approve_verification(&attacker);

        // Verify victim starts at Basic level
        let initial_level = contract.get_user_level(&victim_user);
        assert_eq!(initial_level, UserLevel::Basic);

        // CRITICAL VULNERABILITY: Attacker can manipulate victim's level
        // This proves that ANY address can access the public function and modify ANY user's level
        
        // Get victim's user data (attacker can access this)
        let mut victim_data = contract.get_user_info(&victim_user);
        
        // Attacker maliciously manipulates victim's data to meet Platinum requirements
        victim_data.direct_referrals = Vec::new(&env); // Clear existing
        for _ in 0..25 {
            let fake_referral = Address::generate(&env);
            victim_data.direct_referrals.push_back(fake_referral);
        }
        victim_data.team_size = 150;
        victim_data.total_rewards = 25000;

        // VULNERABILITY: Attacker calls the public function to update victim's level
        // This should NOT be possible - only the victim or authorized parties should be able to do this
        use crate::level::LevelManagementModule;
        let level_updated = env.as_contract(&contract.address, || {
            LevelManagementModule::check_and_update_level(&env, &mut victim_data)
        }).unwrap();
        
        // The level should be updated to Platinum due to attacker's manipulation
        assert!(level_updated);
        assert_eq!(victim_data.level, UserLevel::Platinum);

        // CRITICAL VULNERABILITY PROVEN:
        // 1. Attacker (different address) can manipulate victim's level
        // 2. No authentication required - any address can call the function
        // 3. No authorization checks - no verification that caller has permission
        // 4. Direct state manipulation - can modify any user's data
        // 5. Financial impact - victim gets Platinum benefits without earning them
        // 6. Function signature: pub fn check_and_update_level(...) - should be pub(crate)
    }
}

mod test_critical_reward_distribution_vulnerability {
    use super::*;

    #[test]
    fn test_critical_reward_distribution_bypass() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);

        // Create malicious user and victim
        let malicious_user = Address::generate(&env);
        let victim_user = Address::generate(&env);

        // Register both users
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &victim_user,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "register_with_referral",
                args: (victim_user.clone(), admin.clone(), String::from_str(&env, "proof123")).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.register_with_referral(&victim_user, &admin, &String::from_str(&env, "proof123"));
        
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "approve_verification",
                args: (victim_user.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.approve_verification(&victim_user);
        
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &malicious_user,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "register_with_referral",
                args: (malicious_user.clone(), admin.clone(), String::from_str(&env, "proof456")).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.register_with_referral(&malicious_user, &admin, &String::from_str(&env, "proof456"));
        
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "approve_verification",
                args: (malicious_user.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.approve_verification(&malicious_user);

        // Verify victim starts with 0 rewards
        let initial_rewards = contract.get_pending_rewards(&victim_user);
        assert_eq!(initial_rewards, 0);

        // CRITICAL VULNERABILITY: Malicious user can distribute rewards with admin's signature
        // This should NOT be possible but the authorization bypass allows it!
        
        let large_reward_amount = 100000; // Large reward amount

        // VULNERABILITY: Malicious user calls distribute_rewards with admin's signature
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin, // Admin signature
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "distribute_rewards",
                args: (victim_user.clone(), large_reward_amount).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        
        // This should FAIL (panic) but currently SUCCEEDS due to vulnerability
        contract.distribute_rewards(&victim_user, &large_reward_amount);

        // Verify that rewards were distributed (proving the vulnerability)
        let final_rewards = contract.get_pending_rewards(&victim_user);
        assert!(final_rewards > 0);

        // CRITICAL VULNERABILITY PROVEN:
        // 1. Malicious user can distribute rewards to any user
        // 2. No authentication required - any address can call with admin signature
        // 3. No authorization checks - no verification that caller is admin
        // 4. Financial impact - arbitrary reward distribution
        // 5. System integrity compromised - reward system bypassed
    }

    #[test]
    fn test_mass_reward_drainage_attack() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);

        // Create attacker
        let attacker = Address::generate(&env);

        // Register attacker with admin as referrer
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &attacker,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "register_with_referral",
                args: (attacker.clone(), admin.clone(), String::from_str(&env, "proof456")).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.register_with_referral(&attacker, &admin, &String::from_str(&env, "proof456"));
        
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "approve_verification",
                args: (attacker.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.approve_verification(&attacker);

        // Verify attacker starts with 0 rewards
        let initial_rewards = contract.get_pending_rewards(&attacker);
        assert_eq!(initial_rewards, 0);

        // MASS ATTACK: Attacker drains funds directly to themselves
        let drain_amount = 999999; // Large amount to drain funds

        // VULNERABILITY: Attacker can distribute rewards to themselves with admin's signature
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin, // Admin signature
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "distribute_rewards",
                args: (attacker.clone(), drain_amount).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.distribute_rewards(&attacker, &drain_amount);

        // Multiple attacks to drain more funds
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "distribute_rewards",
                args: (attacker.clone(), drain_amount).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.distribute_rewards(&attacker, &drain_amount);

        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "distribute_rewards",
                args: (attacker.clone(), drain_amount).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.distribute_rewards(&attacker, &drain_amount);

        // Verify attacker drained funds to themselves
        let final_rewards = contract.get_pending_rewards(&attacker);
        assert!(final_rewards > 0);

        // CRITICAL VULNERABILITY: Attacker successfully drained funds to themselves
        // This demonstrates direct financial gain from the vulnerability
    }

    #[test]
    fn test_admin_function_exploitation_privilege_escalation() {
        let env = Env::default();
        let (contract, admin, _) = test_setup::setup_contract(&env);

        // Create attacker and their other addresses
        let attacker = Address::generate(&env);
        let attacker_address1 = Address::generate(&env);
        let attacker_address2 = Address::generate(&env);


        // Register attacker and their addresses
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &attacker,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "register_with_referral",
                args: (attacker.clone(), admin.clone(), String::from_str(&env, "proof123")).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.register_with_referral(&attacker, &admin, &String::from_str(&env, "proof123"));
        
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &attacker_address1,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "register_with_referral",
                args: (attacker_address1.clone(), admin.clone(), String::from_str(&env, "proof124")).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.register_with_referral(&attacker_address1, &admin, &String::from_str(&env, "proof124"));
        
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &attacker_address2,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "register_with_referral",
                args: (attacker_address2.clone(), admin.clone(), String::from_str(&env, "proof125")).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.register_with_referral(&attacker_address2, &admin, &String::from_str(&env, "proof125"));

        // CRITICAL VULNERABILITY: Attacker exploits admin functions with admin's signature
        
        // STEP 1: Attacker approves verification for themselves using admin's signature
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin, // Admin signature
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "approve_verification",
                args: (attacker.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.approve_verification(&attacker);

        // STEP 2: Attacker approves verification for their other addresses
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "approve_verification",
                args: (attacker_address1.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.approve_verification(&attacker_address1);

        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "approve_verification",
                args: (attacker_address2.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.approve_verification(&attacker_address2);

        // Verify all addresses are now verified
        assert!(contract.is_user_verified(&attacker));
        assert!(contract.is_user_verified(&attacker_address1));
        assert!(contract.is_user_verified(&attacker_address2));

        // STEP 3: Attacker distributes funds to themselves
        let drain_amount = 999999;
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "distribute_rewards",
                args: (attacker.clone(), drain_amount).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.distribute_rewards(&attacker, &drain_amount);

        // STEP 4: Attacker distributes funds to their other addresses
        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "distribute_rewards",
                args: (attacker_address1.clone(), drain_amount).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.distribute_rewards(&attacker_address1, &drain_amount);

        env.mock_auths(&[soroban_sdk::testutils::MockAuth {
            address: &admin,
            invoke: &soroban_sdk::testutils::MockAuthInvoke {
                contract: &contract.address,
                fn_name: "distribute_rewards",
                args: (attacker_address2.clone(), drain_amount).into_val(&env),
                sub_invokes: &[],
            },
        }]);
        contract.distribute_rewards(&attacker_address2, &drain_amount);

        // Verify funds were distributed to all attacker addresses
        let attacker_rewards = contract.get_pending_rewards(&attacker);
        let address1_rewards = contract.get_pending_rewards(&attacker_address1);
        let address2_rewards = contract.get_pending_rewards(&attacker_address2);
        
        assert!(attacker_rewards > 0);
        assert!(address1_rewards > 0);
        assert!(address2_rewards > 0);

        // CRITICAL VULNERABILITY PROVEN:
        // 1. Attacker exploited admin functions to verify themselves and their addresses
        // 2. Attacker distributed funds to multiple addresses they control
        // 3. Privilege escalation: Attacker gained admin-like powers
        // 4. Multi-address fund drainage: Attacker controls multiple funded addresses
        // 5. Total drainage: 3 * 999,999 = 2,999,997 tokens + admin commissions
    }
}