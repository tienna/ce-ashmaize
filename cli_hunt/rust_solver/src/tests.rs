#[cfg(test)]
mod tests {
    use super::*;
    use ashmaize::{Rom, RomGenerationType, hash};
    use hex;

    // Copy of hash_structure_good from main.rs for testing purposes
    fn hash_structure_good_for_test(hash: &[u8], zero_bits: usize) -> bool {
        let full_bytes = zero_bits / 8; // Number of full zero bytes
        let remaining_bits = zero_bits % 8; // Bits to check in the next byte

        // Check full zero bytes
        if hash.len() < full_bytes || hash[..full_bytes].iter().any(|&b| b != 0) {
            return false;
        }

        if remaining_bits == 0 {
            return true;
        }
        if hash.len() > full_bytes {
            // Mask for the most significant bits
            let mask = 0xFF << (8 - remaining_bits);
            hash[full_bytes] & mask == 0
        } else {
            false
        }
    }

    #[test]
    fn validate_example_solution() {
        const MB: usize = 1024 * 1024;
        const GB: usize = 1024 * MB;

        let address = "addr1q84h0q756f6fslk9y3v48kztxug9nk2es3wvw3dyumfy2qwvpuzhn97jay38vh4sspz45ukzavalsm0tf6q4gx39rl8sc7f5rf";
        let challenge_id = "**D01C17";
        let difficulty_str = "00007FFF";
        let no_pre_mine = "e8a195800bae57517c85955a784faa6162051f41ef86bcb93be0c3e01a9b63c8";
        let latest_submission = "2025-10-31T15:59:59.000Z";
        let no_pre_mine_hour = "967125414";
        let nonce_hex = "001af01e65703909";
        let nonce = u64::from_str_radix(nonce_hex, 16).unwrap();

        // Initialize AshMaize ROM
        let key = hex::decode(no_pre_mine).unwrap();
        let rom = Rom::new(
            &key,
            RomGenerationType::TwoStep {
                pre_size: 16 * MB,
                mixing_numbers: 4,
            },
            1 * GB,
        );

        // Construct the preimage
        let preimage = format!(
            "{0:016x}{1}{2}{3}{4}{5}{6}",
            nonce,
            address,
            challenge_id,
            difficulty_str,
            no_pre_mine,
            latest_submission,
            no_pre_mine_hour
        );

        // Hash the preimage
        let hash_result = hash(&preimage.as_bytes(), &rom, 8, 256);
        println!("DEBUG: Hash result: {:?}", hash_result);

        // Calculate required leading zeros from difficulty string
        let difficulty_bytes = hex::decode(difficulty_str).unwrap();
        let mut leading_zeros_required = 0;
        for byte in difficulty_bytes {
            leading_zeros_required += byte.leading_zeros();
        }
        println!("DEBUG: Leading zeros required: {}", leading_zeros_required);

        // Validate the hash against the difficulty
        assert!(
            hash_structure_good_for_test(&hash_result, leading_zeros_required as usize),
            "Hash does not meet difficulty requirements"
        );
    }
}
