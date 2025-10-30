Here is a detailed summary of how the Scavenger Hunt technically works and how you could implement a command-line program to participate.

### Scavenger Hunt: Technical Workflow

The Scavenger Hunt is a proof-of-work competition where participants solve cryptographic challenges to earn rewards. The process is managed via a web API and utilizes a custom ASIC-resistant hashing algorithm called AshMaize.

#### 1. Registration

Before participating, each Cardano address you want to use must be registered. This is a one-time process.

*   **Fetch Terms & Conditions (T&C):** Make a `GET` request to the `/TandC` endpoint. The response contains a message that must be signed.
*   **Sign the T&C Message:** Using the private key associated with your Cardano address, create a CIP-8 or CIP-30 signature of the exact T&C message.
*   **Register:** Make a `POST` request to `/register/{address}/{signature}/{pubkey}` with your Cardano address, the signature, and the corresponding public key.

#### 2. The Mining Loop

Once an address is registered, it can participate in the mining loop.

*   **Fetch the Challenge:** A new challenge is available every hour. Fetch the current challenge details by making a `GET` request to the `/challenge` endpoint. The key fields in the response are:
    *   `challenge_id`: A unique identifier for the current challenge.
    *   `difficulty`: A 4-byte hex string representing a bitmask. Your solution's hash must have at least as many leading zero bits as this mask.
    *   `no_pre_mine`: A daily seed used to initialize the AshMaize hashing environment.
    *   `no_pre_mine_hour`: An hourly seed used as part of the data to be hashed.
    *   `latest_submission`: The timestamp by which the solution must be submitted.

*   **Solve the Challenge (Proof-of-Work):** This is the most computationally intensive part. The goal is to find a `nonce` that, when combined with other challenge data and hashed, produces a result that meets the difficulty requirement.

    1.  **Initialize AshMaize ROM:** The AshMaize algorithm requires a large (~1GB) read-only memory (ROM) to be initialized.
        *   This ROM is generated using the `no_pre_mine` value from the challenge.
        *   The ROM only needs to be regenerated when the `no_pre_mine` value changes (i.e., once a day).
        *   The specific configuration parameters for AshMaize are detailed in the whitepaper and `SPECS.md`:
            *   `nbLoops`: 8
            *   `nblnstrs`: 256
            *   `pre_size`: 16777216
            *   `mixing_numbers`: 4
            *   `rom_size`: 1073741824

    2.  **Construct the Preimage:** The data to be hashed (the "preimage") is a specific concatenation of the following string values:
        *   A 64-bit `nonce`, hex-encoded (16 characters).
        *   Your registered Cardano `address`.
        *   The `challenge_id`.
        *   The `difficulty` string.
        *   The `no_pre_mine` string.
        *   The `latest_submission` timestamp.
        *   The `no_pre_mine_hour` string.

    3.  **Find a Valid Nonce:**
        *   Start with a random 64-bit integer for your `nonce`.
        *   Construct the preimage string as described above.
        *   Hash the preimage using the initialized AshMaize algorithm.
        *   Check if the resulting hash satisfies the `difficulty` (i.e., has enough leading zero bits).
        *   If it does, you have found a valid solution.
        *   If not, increment your `nonce` and repeat the process.

*   **Submit the Solution:** Once a valid `nonce` is found, submit it immediately by making a `POST` request to `/solution/{address}/{challenge_id}/{nonce}`. The server will validate your solution.

#### 3. Managing Multiple Addresses

The process is identical for each address. To participate with multiple addresses, you would run the mining loop for each one. A command-line tool can do this in parallel to maximize efficiency.

#### 4. Consolidating Rewards (Optional)

If you are participating with multiple addresses, you can consolidate the rewards into a single address.

*   **Create Signature:** The owner of the `original_address` (the one donating the rewards) must sign the message: `"Assign accumulated Scavenger rights to: <destination_address>"`.
*   **Donate:** Make a `POST` request to `/donate_to/{destination_address}/{original_address}/{signature}`.

### Implementing a Command-Line Tool

A command-line tool to automate this process would need to perform the following actions:

1.  **Configuration:**
    *   Accept a list of Cardano addresses and a secure way to access their corresponding private/signing keys.
    *   Store the API base URL.

2.  **Registration Command:**
    *   Implement a command (`register`) that takes an address, fetches the T&C, prompts for signing (or does it automatically if keys are provided), and submits the registration to the server.

3.  **Mining Command (`mine`):**
    *   This is the main component. It should be a long-running process.
    *   **Parallel Execution:** The tool should spawn a separate worker/thread for each registered address provided in the configuration.
    *   **Worker Logic:** Each worker would execute the following loop:
        1.  **Get Challenge:** Fetch the current challenge from the API.
        2.  **Check ROM:** Determine if the `no_pre_mine` value has changed since the last run. If so, regenerate the AshMaize ROM. This is a significant one-time cost per day.
        3.  **Start Mining:** Begin the nonce-finding loop (construct preimage, hash, check difficulty). This loop should be optimized for performance. The core hashing logic should be implemented by leveraging the provided `ashmaize` Rust crate.
        4.  **Submit Solution:** Upon finding a valid nonce, immediately send it to the solution endpoint.
        5.  **Repeat:** After submitting (or if the challenge expires), the worker should wait for the next challenge to become available and repeat the process.

4.  **Donation Command:**
    *   Implement a command (`donate`) that takes an original address, a destination address, and facilitates the signing and submission process for consolidating rewards.

5.  **Core Hashing Module:**
    *   The most critical part of the implementation is the AshMaize hashing. The provided Rust repository contains the `ashmaize` crate which implements this algorithm. Your tool should be built in Rust to directly and efficiently use this crate. The `ashmaize-web` crate provides a good example of how to interface with the core `ashmaize` library.

By following this structure, you can create a powerful command-line tool to efficiently participate in the Scavenger Hunt with multiple addresses without relying on the web interface.