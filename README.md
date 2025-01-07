# Wedding and Vendor Management Canister

This Rust-based Internet Computer (IC) canister manages weddings, vendors, tasks, timelines, guests, and registries. Built using `ic_cdk` and `ic_stable_structures` for canister operations and stable memory management.

## Features

### Vendor Management
- Register vendors with details (name, category, cost)
- Search vendors by category
- Fetch all registered vendors

### Wedding Management
- Create wedding records
- Retrieve wedding details
- List all weddings

### Guest Management
- Submit guest RSVPs
- Approve RSVPs and assign tables
- Fetch guest lists and RSVP statuses

### Task Management
- Add timeline tasks
- Update task statuses
- Delete tasks

### Registry Management
- Add registry items
- Update item purchase status
- Delete registry items

## Data Models

### Core Types
- **Vendor**: Service provider details (category, cost, availability)
- **Wedding**: Wedding details (couple names, date, location, tasks, guest lists)
- **Task**: Wedding-associated tasks
- **Guest**: Guest information and RSVP statuses
- **Registry Item**: Wedding registry items

### Enums
- **Category**: Service categories (Catering, Music)
- **TableAssignment**: Guest table assignments (VIP Table)

### Payloads
Structured payloads for data operations:
- Vendor registration, booking, updates
- Wedding creation, management
- Guest RSVP submission, approvals

## Requirements

Install required tools:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install candid-extractor
cargo install candid-extractor

# Install dfx
DFX_VERSION=0.15.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
echo 'export PATH="$PATH:$HOME/bin"' >> "$HOME/.bashrc"
source ~/.bashrc
dfx start --background
```

Quick start:
```bash
cd icp_rust_boilerplate/
dfx help
dfx canister --help
```

## Dependencies

Update `/src/{canister_name}/Cargo.toml`:

```toml
[dependencies]
candid = "0.9.9"
ic-cdk = "0.11.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
ic-stable-structures = { git = "https://github.com/lwshang/stable-structures.git", branch = "lwshang/update_cdk"}
```

## Candid Generation

1. Add the did.sh script from:
```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh
```

2. Update canister name on line 16 from:
```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh#L16
```

3. Optional: Add package.json:
```json
{
    "scripts": {
        "generate": "./did.sh && dfx generate",
        "gen-deploy": "./did.sh && dfx generate && dfx deploy -y"
    }
}
```

Run `npm run generate` for candid generation or `npm run gen-deploy` for generation and deployment.

## Local Development

```bash
# Start replica
dfx start --background

# Deploy canisters
dfx deploy
```
