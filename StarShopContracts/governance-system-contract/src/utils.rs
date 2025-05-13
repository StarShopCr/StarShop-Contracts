use soroban_sdk::{xdr::ToXdr, Address, Bytes, Env, IntoVal, Symbol, Val};

/// Generate a storage key for a user
///
/// # Arguments
/// * `env` - The environment object
/// * `key_bytes` - The base key bytes
/// * `key_num` - The numeric key to append
///
/// # Returns
/// * `Symbol` - A unique storage key for this user
pub fn get_key_str(env: &Env, mut key_bytes: Bytes, key_num: u32) -> Symbol {
    // Convert the numeric key to a string and append it to the base key
    if key_num == 0 {
        key_bytes.extend_from_array(b"0");
    } else {
        // Convert the number to its string representation
        let mut digits = [0u8; 10];
        let mut num = key_num;
        let mut digit_count = 0;

        while num > 0 {
            digits[digit_count] = (num % 10) as u8 + b'0';
            num /= 10;
            digit_count += 1;
        }

        // Reverse the digits to get the correct order
        for i in (0..digit_count).rev() {
            key_bytes.push_back(digits[i]);
        }
    }

    // Convert the key bytes to a buffer
    let len = key_bytes.len() as usize;
    let mut buffer = [0u8; 32];
    // Copy the key bytes into the buffer
    for i in 0..len {
        buffer[i] = key_bytes.get(i as u32).unwrap();
    }

    // Convert the buffer to a string and create a Symbol
    let key = unsafe { core::str::from_utf8_unchecked(&buffer[..len]) };
    Symbol::new(env, &key)
}

/// Generate a unique key for the vote specification (proposal, status, vote, delegation)
///
/// * Arguments
/// * `env` - The environment object
/// * `key_bytes` - The base key bytes
/// * `proposal_id` - The proposal ID
/// * `addr` - The address of the user
///
pub fn get_governance_op_key(
    env: &Env,
    mut key_bytes: Bytes,
    proposal_id: u32,
    addr: &Address,
) -> Symbol {
    // Convert the address to bytes
    let val: Val = addr.clone().into_val(env);
    let xdr_bytes = val.to_xdr(env);

    // Convert each byte to hex
    let mut hex_chars = Bytes::new(env);
    for byte in xdr_bytes.iter() {
        let high = (byte >> 4) & 0xF;
        let low = byte & 0xF;
        let high_char = if high < 10 { b'0' + high } else { b'a' + (high - 10) };
        let low_char = if low < 10 { b'0' + low } else { b'a' + (low - 10) };
        hex_chars.push_back(high_char);
        hex_chars.push_back(low_char);
    }

    // Append the proposal ID and hex characters to the key
    if proposal_id == 0 {
        key_bytes.extend_from_array(b"0");
    } else {
        // Convert the proposal ID to a string
        let mut digits = [0u8; 60];
        let mut num = proposal_id;
        let mut digit_count = 0;

        while num > 0 {
            digits[digit_count] = (num % 10) as u8 + b'0';
            num /= 10;
            digit_count += 1;
        }

        // Reverse the digits to get the correct order
        for i in (0..digit_count).rev() {
            key_bytes.push_back(digits[i]);
        }
    }

    // Append the hex characters to the key
    key_bytes.extend_from_array(b"_");
    key_bytes.insert_from_bytes(key_bytes.len() as u32, hex_chars);

    // Ensure the final key doesn't exceed 32 bytes (Soroban Symbol limit)
    let max_key_length = 32;
    let mut truncated = Bytes::new(env);
    if key_bytes.len() > max_key_length {
        // Take first 16 bytes
        for i in 0..16 {
            if i < key_bytes.len() {
                truncated.push_back(key_bytes.get(i).unwrap());
            }
        }
        
        // Add last 16 bytes (or remaining bytes if less)
        if key_bytes.len() > 32 {
            let start_idx = key_bytes.len() - 16;
            for i in start_idx..key_bytes.len() {
                truncated.push_back(key_bytes.get(i).unwrap());
            }
        } else {
            // If less than 32 bytes, add remaining
            for i in 16..key_bytes.len() {
                truncated.push_back(key_bytes.get(i).unwrap());
            }
        }
    }
    
    // Convert the key bytes to a string
    let len = truncated.len() as usize;
    let mut buffer = [0u8; 89];
    for i in 0..len {
        // Copy the key bytes into the buffer
        buffer[i] = truncated.get(i as u32).unwrap();
    }

    let key = unsafe { core::str::from_utf8_unchecked(&buffer[..32]) };
    Symbol::new(env, &key)
}
