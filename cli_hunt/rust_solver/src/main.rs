use ashmaize::{hash, Rom, RomGenerationType};
use clap::Parser;
use hex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    address: String,
    #[arg(long)]
    challenge_id: String,
    #[arg(long)]
    difficulty: String, // This is the number of leading zero bits required
    #[arg(long)]
    no_pre_mine: String,
    #[arg(long)]
    latest_submission: String,
    #[arg(long)]
    no_pre_mine_hour: String,
}

fn hash_structure_good(hash: &[u8], zero_bits: usize) -> bool {
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

fn main() {
    const MB: usize = 1024 * 1024;
    const GB: usize = 1024 * MB;

    let args = Args::parse();

    // Initialize AshMaize ROM
    let key = hex::decode(&args.no_pre_mine).unwrap();
    let rom = Rom::new(
        &key,
        RomGenerationType::TwoStep {
            pre_size: 16 * MB, // 16777216
            mixing_numbers: 4,
        },
        1 * GB, // 1073741824
    );

    let mut nonce: u64 = 0; // Start with a random nonce or 0

    // Parse difficulty from hex string to number of zero bits
    let difficulty_bytes = hex::decode(&args.difficulty).unwrap();
    let mut leading_zeros_required = 0;
    for byte in difficulty_bytes {
        leading_zeros_required += byte.leading_zeros();
    }

    loop {
        let preimage = format!(
            "{0:016x}{1}{2}{3}{4}{5}{6}",
            nonce,
            args.address,
            args.challenge_id,
            args.difficulty, // This is the hex string, not the number of zero bits
            args.no_pre_mine,
            args.latest_submission,
            args.no_pre_mine_hour
        );

        let hash_result = hash(&preimage.as_bytes(), &rom, 8, 256);

        if hash_structure_good(&hash_result, leading_zeros_required as usize) {
            println!("{}", nonce);
            break;
        }

        nonce += 1;
    }
}