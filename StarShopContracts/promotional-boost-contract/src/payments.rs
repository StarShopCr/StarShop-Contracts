use soroban_sdk::{token, Address, Env};

pub struct PaymentProcessor;

impl PaymentProcessor {
    /// Accepts a payment in XLM from the seller and verifies it's at least the required amount
    pub fn collect_payment(
        env: &Env,
        from: &Address,
        amount: i128,
        price_required: i128,
    ) -> Result<(), &'static str> {
        if amount < price_required {
            return Err("Insufficient payment");
        }

        // Use Stellar's native asset (XLM) - in test this will be the mocked token
        let token_id = Self::get_xlm_token_address(env);
        let token = token::Client::new(env, &token_id);
        token.transfer(from, &env.current_contract_address(), &amount);

        Ok(())
    }

    /// Refund a payment in XLM to the given address
    pub fn refund_payment(env: &Env, to: &Address, amount: i128) -> Result<(), &'static str> {
        if amount <= 0 {
            return Err("Nothing to refund");
        }

        let token_id = Self::get_xlm_token_address(env);
        let token = token::Client::new(env, &token_id);
        token.transfer(&env.current_contract_address(), to, &amount);

        Ok(())
    }

    /// Calculates required slot price (fixed-tier example, but can be replaced with auction logic)
    pub fn calculate_price(duration_secs: u64) -> i128 {
        let base_price = 5_000_000i128; // 5 XLM in stroops (1 XLM = 1_000_000 stroops)
        let daily_rate = base_price; // per day rate

        // Calculate days using integer arithmetic (86400 seconds = 1 day)
        let days = duration_secs / 86400;
        (days as i128) * daily_rate
    }

    /// Gets the XLM token address - in production this would be the native XLM,
    /// in tests this will be the mocked stellar asset contract
    fn get_xlm_token_address(env: &Env) -> Address {
        // Try to get from storage first (for test environment)
        if let Some(addr) = env.storage().instance().get::<soroban_sdk::Symbol, Address>(&soroban_sdk::symbol_short!("xlm_addr")) {
            return addr;
        }
        
        // Fallback to hardcoded address for production (Stellar native asset representation)
        // This is a placeholder - in real Stellar, you'd use the native asset differently
        Address::from_str(
            env,
            "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC",
        )
    }

    /// Sets the XLM token address (for testing)
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn set_xlm_token_address(env: &Env, token_address: &Address) {
        env.storage().instance().set(&soroban_sdk::symbol_short!("xlm_addr"), token_address);
    }
}
