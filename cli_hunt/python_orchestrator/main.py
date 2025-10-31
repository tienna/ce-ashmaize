import requests
import json
import subprocess
import time
import os
from datetime import datetime, timezone

CONFIG_FILE = "config.json"
DB_FILE = "challenges.json"
RUST_SOLVER_PATH = (
    "../rust_solver/target/release/ashmaize-solver"  # Assuming it's built
)


def load_config():
    if not os.path.exists(CONFIG_FILE):
        print(
            "Config file not found. Please create config.json with your Cardano addresses."
        )
        return {"addresses": []}
    with open(CONFIG_FILE, "r") as f:
        return json.load(f)


def save_db(db):
    with open(DB_FILE, "w") as f:
        json.dump(db, f, indent=4)


def load_db():
    if not os.path.exists(DB_FILE):
        return {}
    with open(DB_FILE, "r") as f:
        return json.load(f)


def fetch_challenges(addresses):
    print("Fetching challenges...")
    db = load_db()
    for address in addresses:
        try:
            response = requests.get("https://sm.midnight.gd/api/challenge")
            response.raise_for_status()
            challenge_data = response.json()["challenge"]

            new_challenge = {
                "challengeId": challenge_data["challenge_id"],
                "challengeNumber": challenge_data["challenge_number"],
                "campaignDay": challenge_data["day"],
                "difficulty": challenge_data["difficulty"],
                "status": "available",
                "noPreMine": challenge_data["no_pre_mine"],
                "noPreMineHour": challenge_data["no_pre_mine_hour"],
                "latestSubmission": challenge_data["latest_submission"],
                "availableAt": challenge_data["issued_at"],
            }

            if address not in db:
                db[address] = []

            # Check if challenge already exists for this address
            if not any(
                c["challengeId"] == new_challenge["challengeId"] for c in db[address]
            ):
                db[address].append(new_challenge)
                db[address].sort(key=lambda c: c["challengeId"])
                print(
                    f"New challenge fetched for {address}: {new_challenge['challengeId']}"
                )
            else:
                print(
                    f"Challenge {new_challenge['challengeId']} already exists for {address}"
                )
        except requests.exceptions.RequestException as e:
            print(f"Error fetching challenge for {address}: {e}")
    save_db(db)


def solve_challenges():
    print("Solving challenges...")
    db = load_db()
    now = datetime.now(timezone.utc)
    for address, challenges in db.items():
        for challenge in challenges:
            if challenge["status"] == "available":
                latest_submission = datetime.fromisoformat(
                    challenge["latestSubmission"].replace("Z", "+00:00")
                )
                if now > latest_submission:
                    challenge["status"] = "expired"
                    print(
                        f"Challenge {challenge['challengeId']} for {address} has expired."
                    )
                    continue

                print(
                    f"Attempting to solve challenge {challenge['challengeId']} for {address}"
                )
                try:
                    command = [
                        RUST_SOLVER_PATH,
                        "--address",
                        address,
                        "--challenge-id",
                        challenge["challengeId"],
                        "--difficulty",
                        challenge["difficulty"],
                        "--no-pre-mine",
                        challenge["noPreMine"],
                        "--latest-submission",
                        challenge["latestSubmission"],
                        "--no-pre-mine-hour",
                        challenge["noPreMineHour"],
                    ]
                    result = subprocess.run(
                        command, capture_output=True, text=True, check=True
                    )
                    nonce = result.stdout.strip()
                    print(f"Found nonce: {nonce}")

                    # Submit solution
                    submit_url = f"https://sm.midnight.gd/api/solution/{address}/{challenge['challengeId']}/{nonce}"
                    submit_response = requests.post(submit_url, data={})
                    submit_response.raise_for_status()
                    print(
                        f"Solution submitted successfully for {challenge['challengeId']}"
                    )
                    challenge["status"] = "solved"
                    challenge["solvedAt"] = (
                        datetime.now(timezone.utc)
                        .isoformat(timespec="milliseconds")
                        .replace("+00:00", "Z")
                    )
                    challenge["salt"] = nonce
                    try:
                        submission_data = submit_response.json()
                        if "hash" in submission_data:
                            challenge["hash"] = submission_data["hash"]
                    except json.JSONDecodeError:
                        pass

                except subprocess.CalledProcessError as e:
                    print(f"Rust solver error: {e.stderr}")
                except requests.exceptions.RequestException as e:
                    print(f"Error submitting solution: {e}")
                except Exception as e:
                    print(f"An unexpected error occurred: {e}")
    save_db(db)


def main():
    config = load_config()
    addresses = config.get("addresses", [])

    if not addresses:
        print("No addresses found in config.json. Please add your Cardano addresses.")
        return

    # Simple CLI for now
    while True:
        print("\nChoose an action:")
        print("1. Fetch challenges")
        print("2. Solve challenges")
        print("3. Exit")
        choice = input("Enter your choice: ")

        if choice == "1":
            fetch_challenges(addresses)
        elif choice == "2":
            solve_challenges()
        elif choice == "3":
            break
        else:
            print("Invalid choice. Please try again.")


if __name__ == "__main__":
    main()
