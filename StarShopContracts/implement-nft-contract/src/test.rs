#[cfg(test)]
mod tests {
    use crate::{NFTContract, NFTContractClient, NFTDetail, COUNTER_KEY};
    use soroban_sdk::{
        testutils::Address as _,
        vec,
        Address,
        Env, // Changed import
        String,
        Vec,
    };

    fn setup() -> (Env, Address, NFTContractClient<'static>) {
        let env = Env::default();
        let contract_id = env.register(NFTContract {}, ()); // Added empty tuple for constructor args
        let client = NFTContractClient::new(&env, &contract_id);

        env.as_contract(&contract_id, || {
            env.storage().instance().set(&COUNTER_KEY, &0u32);
        });

        (env, contract_id, client)
    }

    #[test]
    fn test_mint_with_metadata() {
        let (env, contract_id, client) = setup();
        let user = Address::generate(&env);

        env.mock_all_auths();

        let name = String::from_str(&env, "Test_NFT");
        let description = String::from_str(&env, "A_test_NFT_with_metadata");
        let mut attributes = Vec::new(&env);
        attributes.push_back(String::from_str(&env, "attribute1"));
        attributes.push_back(String::from_str(&env, "attribute2"));

        let token_id = client.mint_nft(&user, &name, &description, &attributes);

        env.as_contract(&contract_id, || {
            let nft: NFTDetail = env.storage().persistent().get(&token_id).unwrap();
            assert_eq!(nft.metadata.name, name, "Name mismatch");
            assert_eq!(
                nft.metadata.description, description,
                "Description mismatch"
            );
            assert_eq!(nft.metadata.attributes, attributes, "Attributes mismatch");
        });
    }

    #[test]
    fn test_metadata_persistence_after_transfer() {
        let (env, contract_id, client) = setup();
        let original_owner = Address::generate(&env);
        let new_owner = Address::generate(&env);

        env.mock_all_auths();

        let name = String::from_str(&env, "Transferable_NFT");
        let description = String::from_str(&env, "NFT_with_persistent_metadata");
        let mut attributes = Vec::new(&env);
        attributes.push_back(String::from_str(&env, "transferable"));

        let token_id = client.mint_nft(&original_owner, &name, &description, &attributes);

        client.transfer_nft(&original_owner, &new_owner, &token_id);

        env.as_contract(&contract_id, || {
            let nft: NFTDetail = env.storage().persistent().get(&token_id).unwrap();
            assert_eq!(nft.metadata.name, name, "Name changed after transfer");
            assert_eq!(
                nft.metadata.description, description,
                "Description changed after transfer"
            );
            assert_eq!(
                nft.metadata.attributes, attributes,
                "Attributes changed after transfer"
            );
        });
    }

    #[test]
    fn test_metadata_removal_after_burn() {
        let (env, contract_id, client) = setup();
        let owner = Address::generate(&env);

        env.mock_all_auths();

        let token_id = client.mint_nft(
            &owner,
            &String::from_str(&env, "Burnable_NFT"),
            &String::from_str(&env, "NFT_to_be_burned"),
            &Vec::new(&env),
        );

        client.burn_nft(&owner, &token_id);

        // Verify NFT doesn't exist after burning
        env.as_contract(&contract_id, || {
            let result = env.storage().persistent().get::<u32, NFTDetail>(&token_id);
            assert!(result.is_none(), "NFT should not exist after burning");
        });
    }

    #[test]
    fn test_mint_reward_nft() {
        let (env, contract_id, client) = setup();
        let recipient = Address::generate(&env);

        env.mock_all_auths();

        let reward_metadata = vec![
            &env,
            String::from_str(&env, "Achievement_Level_Gold"),
            String::from_str(&env, "Points_1000"),
        ];

        let token_id = client.mint_nft(
            &recipient,
            &String::from_str(&env, "Performance_Reward"),
            &String::from_str(&env, "Outstanding_performance_achievement_reward"),
            &reward_metadata,
        );

        env.as_contract(&contract_id, || {
            let nft: NFTDetail = env.storage().persistent().get(&token_id).unwrap();
            assert_eq!(nft.owner, recipient);
            assert_eq!(
                nft.metadata.name,
                String::from_str(&env, "Performance_Reward")
            );
            assert_eq!(reward_metadata.len(), nft.metadata.attributes.len());
        });
    }

    #[test]
    fn test_mint_invoice_nft() {
        let (env, contract_id, client) = setup();
        let recipient = Address::generate(&env);

        env.mock_all_auths();

        let invoice_metadata = vec![
            &env,
            String::from_str(&env, "Invoice_Number_INV-2024-001"),
            String::from_str(&env, "Amount_500_USDC"),
            String::from_str(&env, "Due_Date_2025-01-31"),
        ];

        let token_id = client.mint_nft(
            &recipient,
            &String::from_str(&env, "Service_Invoice"),
            &String::from_str(&env, "Professional_services_rendered_January_2024"),
            &invoice_metadata,
        );

        env.as_contract(&contract_id, || {
            let nft: NFTDetail = env.storage().persistent().get(&token_id).unwrap();
            assert_eq!(nft.owner, recipient);
            assert_eq!(nft.metadata.name, String::from_str(&env, "Service_Invoice"));
            assert_eq!(invoice_metadata.len(), nft.metadata.attributes.len());
        });
    }

    #[test]
    fn test_unauthorized_metadata_update() {
        let (env, contract_id, client) = setup(); // Changed to use contract_id
        let owner = Address::generate(&env);
        let unauthorized_user = Address::generate(&env);

        env.mock_all_auths();

        let token_id = client.mint_nft(
            &owner,
            &String::from_str(&env, "Protected_NFT"),
            &String::from_str(&env, "NFT_with_protected_metadata"),
            &Vec::new(&env),
        );

        let result = client.try_update_metadata(
            &unauthorized_user,
            &token_id,
            &String::from_str(&env, "Unauthorized_Update"),
            &String::from_str(&env, "This_should_fail"),
            &Vec::new(&env),
        );

        assert!(result.is_err(), "Unauthorized update should fail");

        // Add contract context check if needed
        env.as_contract(&contract_id, || {
            let nft: NFTDetail = env.storage().persistent().get(&token_id).unwrap();
            assert_eq!(nft.owner, owner, "Owner should not change");
        });
    }

    // Test Nft Distribution
    #[test]
    fn test_nft_distribution() {
        let (env, contract_id, client) = setup();

        let recipients = [
            Address::generate(&env),
            Address::generate(&env),
            Address::generate(&env),
        ];

        env.mock_all_auths();

        let mut token_ids = Vec::new(&env);

        for (index, recipient) in recipients.iter().enumerate() {
            let name = if index == 0 {
                String::from_str(&env, "NFT_1")
            } else if index == 1 {
                String::from_str(&env, "NFT_2")
            } else {
                String::from_str(&env, "NFT_3")
            };

            let description = if index == 0 {
                String::from_str(&env, "Description_1")
            } else if index == 1 {
                String::from_str(&env, "Description_2")
            } else {
                String::from_str(&env, "Description_3")
            };

            let mut attributes = Vec::new(&env);
            attributes.push_back(if index == 0 {
                String::from_str(&env, "attribute_1")
            } else if index == 1 {
                String::from_str(&env, "attribute_2")
            } else {
                String::from_str(&env, "attribute_3")
            });

            let token_id = client.mint_nft(recipient, &name, &description, &attributes);
            token_ids.push_back(token_id);
        }

        // Verify each NFT ownership and metadata
        env.as_contract(&contract_id, || {
            for (index, (token_id, recipient)) in
                token_ids.iter().zip(recipients.iter()).enumerate()
            {
                let nft: NFTDetail = env.storage().persistent().get(&token_id).unwrap();

                assert_eq!(&nft.owner, recipient, "NFT owner mismatch");

                // Verify metadata
                let expected_name = if index == 0 {
                    String::from_str(&env, "NFT_1")
                } else if index == 1 {
                    String::from_str(&env, "NFT_2")
                } else {
                    String::from_str(&env, "NFT_3")
                };

                let expected_description = if index == 0 {
                    String::from_str(&env, "Description_1")
                } else if index == 1 {
                    String::from_str(&env, "Description_2")
                } else {
                    String::from_str(&env, "Description_3")
                };

                let expected_attribute = if index == 0 {
                    String::from_str(&env, "attribute_1")
                } else if index == 1 {
                    String::from_str(&env, "attribute_2")
                } else {
                    String::from_str(&env, "attribute_3")
                };

                assert_eq!(nft.metadata.name, expected_name, "NFT name mismatch");
                assert_eq!(
                    nft.metadata.description, expected_description,
                    "NFT description mismatch"
                );
                assert_eq!(
                    nft.metadata.attributes.get(0).unwrap(),
                    expected_attribute,
                    "NFT attribute mismatch"
                );
            }
        });
    }

    #[test]
    fn test_nft_distribution_edge_cases() {
        let (env, contract_id, client) = setup();
        let recipient = Address::generate(&env);

        let result = client.try_mint_nft(
            &recipient,
            &String::from_str(&env, "Unauthorized_NFT"),
            &String::from_str(&env, "Should_fail_without_auth"),
            &Vec::new(&env),
        );
        assert!(result.is_err(), "Minting should fail without authorization");

        env.mock_all_auths();

        let token_id = client.mint_nft(
            &recipient,
            &String::from_str(&env, "Authorized_NFT"),
            &String::from_str(&env, "Should_succeed_with_auth"),
            &Vec::new(&env),
        );

        env.as_contract(&contract_id, || {
            let nft: NFTDetail = env.storage().persistent().get(&token_id).unwrap();
            assert_eq!(nft.owner, recipient, "NFT owner mismatch");
        });

        assert_eq!(token_id, 1, "First token ID should be 1");
    }
}
