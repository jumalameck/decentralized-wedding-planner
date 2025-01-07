# Wedding and Vendor Management Canister

This Rust-based Internet Computer (IC) canister provides core functionality for managing weddings, vendors, tasks, timelines, guests, and registries. The application uses IC-specific libraries like `ic_cdk` and `ic_stable_structures` for canister operations and stable memory management.

## Features

### Vendor Management

- Register a vendor with essential details (name, category, cost, etc.).
- Search vendors by category.
- Fetch all registered vendors.

### Wedding Management

- Create a new wedding record.
- Retrieve details of a wedding.
- Fetch a list of all weddings.

### Guest Management

- Submit guest RSVPs.
- Approve guest RSVPs and assign tables.
- Fetch guest lists and RSVP statuses.

### Task Management

- Add tasks to a wedding timeline.
- Update task statuses.
- Delete tasks.

### Registry Management

- Add registry items.
- Update item statuses (e.g., purchased).
- Delete registry items.

## Data Models

### Core Types

- **Vendor**: Details of service providers (e.g., category, cost, availability).
- **Wedding**: Comprehensive wedding details (e.g., couple names, date, location, tasks, guest lists).
- **Task**: Manage tasks associated with a wedding.
- **Guest**: Guest information and RSVP statuses.
- **Registry Item**: Items in a wedding registry.

### Enums

- **Category**: Predefined service categories (e.g., Catering, Music).
- **TableAssignment**: Guest table assignments (e.g., VIP Table).

### Payloads

Structured payloads for updating and querying data:

- Vendor registration, booking, and updates.
- Wedding creation and management.
- Guest RSVP submission and approvals.

## Requirements

- rustc 1.64 or higher

```bash
$ curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
$ source "$HOME/.cargo/env"
```

- rust wasm32-unknown-unknown targetz

```bash
$ rustup target add wasm32-unknown-unknown
```

- candid-extractor

```bash
$ cargo install candid-extractor
```

- install `dfx`

```bash
$ DFX_VERSION=0.15.0 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
$ echo 'export PATH="$PATH:$HOME/bin"' >> "$HOME/.bashrc"
$ source ~/.bashrc
$ dfx start --background
```

If you want to start working on your project right away, you might want to try the following commands:

```bash
$ cd icp_rust_boilerplate/
$ dfx help
$ dfx canister --help
```

## Update dependencies

update the `dependencies` block in `/src/{canister_name}/Cargo.toml`:

```
[dependencies]
candid = "0.9.9"
ic-cdk = "0.11.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1.0"
ic-stable-structures = { git = "https://github.com/lwshang/stable-structures.git", branch = "lwshang/update_cdk"}
```

## did autogenerate

Add this script to the root directory of the project:

```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh
```

Update line 16 with the name of your canister:

```
https://github.com/buildwithjuno/juno/blob/main/scripts/did.sh#L16
```

After this run this script to generate Candid.
Important note!

You should run this script each time you modify/add/remove exported functions of the canister.
Otherwise, you'll have to modify the candid file manually.

Also, you can add package json with this content:

```
{
    "scripts": {
        "generate": "./did.sh && dfx generate",
        "gen-deploy": "./did.sh && dfx generate && dfx deploy -y"
      }
}
```

and use commands `npm run generate` to generate candid or `npm run gen-deploy` to generate candid and to deploy a canister.

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
$ dfx start --background

# Deploys your canisters to the replica and generates your candid interface
$ dfx deploy
```
