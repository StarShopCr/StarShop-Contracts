use crate::types::{
    AdminConfig, DataKey, Error, Product, ProductCreatedEvent, Vote, VoteAction, 
    VoteCastEvent, VoteHistoryEntry, VoteType
};
use soroban_sdk::{Address, Env, Map, Symbol, Vec};

pub struct VoteManager;

impl VoteManager {
    pub fn init(env: &Env) {
        let products: Map<Symbol, Product> = Map::new(env);
        env.storage()
            .instance()
            .set(&DataKey::Products, &products);
        
        // Initialize product count
        env.storage()
            .instance()
            .set(&DataKey::ProductCount, &0u32);
    }

    // AUTHORIZATION: Initialize admin
    pub fn init_admin(
        env: &Env,
        admin: Address,
        max_products_per_user: u32,
        voting_period_days: u32,
        reversal_window_hours: u32
        )-> Result<(), Error> {
        admin.require_auth();
        
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        let admin_config = AdminConfig {
            admin,
            initialized: true,
            max_products_per_user,
            voting_period_days,
            reversal_window_hours,
        };

        env.storage().instance().set(&DataKey::Admin, &admin_config);
        Ok(())
    }

    // AUTHORIZATION: Verify admin access
    fn verify_admin(env: &Env) -> Result<AdminConfig, Error> {
        let admin_config: AdminConfig = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;
        
        admin_config.admin.require_auth();
        Ok(admin_config)
    }

    // INPUT VALIDATION & AUTHORIZATION: Enhanced product creation
    pub fn create_product(env: &Env, id: Symbol, name: Symbol, creator: Address) -> Result<(), Error> {
        creator.require_auth();

        // INPUT VALIDATION: Check for empty/invalid inputs
        // FIXED: Use proper Symbol validation for no_std environment
        if id == Symbol::new(env, "") || name == Symbol::new(env, "") {
            return Err(Error::InvalidInput);
        }

        let mut products: Map<Symbol, Product> = env
            .storage()
            .instance()
            .get(&DataKey::Products)
            .unwrap_or_else(|| Map::new(env));

        if products.contains_key(id.clone()) {
            return Err(Error::ProductExists);
        }

        // AUTHORIZATION: Check admin config and per-user limits
        let admin_config: AdminConfig = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        // Check per-user product creation limits
        let mut user_product_counts: Map<Address, u32> = env
            .storage()
            .instance()
            .get(&DataKey::UserProductCounts)
            .unwrap_or_else(|| Map::new(env));

        let current_count = user_product_counts.get(creator.clone()).unwrap_or(0);
        if current_count >= admin_config.max_products_per_user {
            return Err(Error::DailyLimitReached); // Reusing for product limit
        }

        // Create product with audit trail support
        let product = Product {
            id: id.clone(),
            name: name.clone(),
            created_at: env.ledger().timestamp(),
            creator: creator.clone(),
            votes: Map::new(env),
            vote_history: Vec::new(env),
            is_active: true,
        };

        products.set(id.clone(), product);
        env.storage().instance().set(&DataKey::Products, &products);

        // Update user product count
        user_product_counts.set(creator.clone(), current_count + 1);
        env.storage().instance().set(&DataKey::UserProductCounts, &user_product_counts);

        // EVENTS: Emit product creation event
        let event = ProductCreatedEvent {
            product_id: id,
            name,
            creator,
            timestamp: env.ledger().timestamp(),
        };
        env.events().publish(("product_created",), event);

        Ok(())
    }

    pub fn cast_vote(
        env: &Env,
        product_id: Symbol,
        vote_type: VoteType,
        voter: Address,
    ) -> Result<(), Error> {
        voter.require_auth();

        let admin_config: AdminConfig = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)?;

        let mut products: Map<Symbol, Product> = env
            .storage()
            .instance()
            .get(&DataKey::Products)
            .ok_or(Error::ProductNotFound)?;

        let mut product = products.get(product_id.clone()).ok_or(Error::ProductNotFound)?;

        if !product.is_active {
            return Err(Error::VotingPeriodEnded);
        }

        let now = env.ledger().timestamp();

        // Check voting period using admin config
        let voting_period = admin_config.voting_period_days as u64 * 24 * 60 * 60;
        if now > product.created_at + voting_period {
            return Err(Error::VotingPeriodEnded);
        }

        let previous_vote = product.votes.get(voter.clone());
        let mut vote_action = VoteAction::NewVote;
        let mut previous_vote_type: Option<VoteType> = None;

        // SECURITY FIX: Properly scoped vote reversal logic
        if let Some(existing_vote) = &previous_vote {
            let reversal_window = admin_config.reversal_window_hours as u64 * 60 * 60;
            
            // Check reversal window (24 hours)
            if now > existing_vote.timestamp + reversal_window {
                return Err(Error::ReversalWindowExpired);
            }
            
            // SECURITY FIX: Properly check if user is trying to cast same vote type
            if existing_vote.vote_type == vote_type {
                return Err(Error::AlreadyVoted);
            }
            
            vote_action = VoteAction::ChangeVote;
            previous_vote_type = Some(existing_vote.vote_type);
        }

        // Record vote with enhanced tracking
        let vote = Vote {
            vote_type,
            timestamp: now,
            voter: voter.clone(),
            last_modified: now,
        };

        // AUDIT TRAIL: Add to immutable vote history
        let history_entry = VoteHistoryEntry {
            voter: voter.clone(),
            vote_type,
            timestamp: now,
            action: vote_action,
            previous_vote: previous_vote_type.unwrap_or(VoteType::None),
        };

        product.votes.set(voter.clone(), vote);
        product.vote_history.push_back(history_entry);

        products.set(product_id.clone(), product);
        env.storage().instance().set(&DataKey::Products, &products);

        // EVENTS: Emit vote cast event
        let event = VoteCastEvent {
            product_id,
            voter,
            vote_type,
            timestamp: now,
            previous_vote: previous_vote_type.unwrap_or(VoteType::None),
        };
        env.events().publish(("vote_cast",), event);

        Ok(())
    }

    pub fn get_product(env: &Env, product_id: Symbol) -> Option<Product> {
        let products: Map<Symbol, Product> = env
            .storage()
            .instance()
            .get(&DataKey::Products)?;
        products.get(product_id)
    }

    // ADMIN FUNCTION: Deactivate product
    pub fn deactivate_product(env: &Env, product_id: Symbol) -> Result<(), Error> {
        Self::verify_admin(env)?;

        let mut products: Map<Symbol, Product> = env
            .storage()
            .instance()
            .get(&DataKey::Products)
            .ok_or(Error::ProductNotFound)?;

        let mut product = products.get(product_id.clone()).ok_or(Error::ProductNotFound)?;
        product.is_active = false;

        products.set(product_id, product);
        env.storage().instance().set(&DataKey::Products, &products);
        Ok(())
    }

    // ADMIN FUNCTION: Get admin config
    pub fn get_admin_config(env: &Env) -> Result<AdminConfig, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(Error::NotInitialized)
    }
}
