# E-Ticketer-ICP

## Overview

This is a comprehensive ticketing system smart contract, providing a variety of functions for managing events, users, and tickets. The use of stable data structures and Candid serialization for compatibility with the Internet Computer framework. The design is modular, with distinct functions for different aspects of the ticketing system, supporting CRUD operations and relationship management.

## Prerequisites

- Rust
- Internet Computer SDK
- IC CDK

## Installation

1. **Clone the repository:**

    ```bash
    git clone https://github.com/foryouflowerai/e-ticketer-ICP.git
    cd e-ticketer-ICP
    ```

## Data Structure

### Type Aliases

- `Memory`: Alias for `VirtualMemory<DefaultMemoryImpl>`.
- `IdCell`: Alias for `Cell<u64, Memory>`.

### Struct Definitions

- `Event`, `User`, `Ticket`: Structs representing Event, User, and Ticket entities.
  - Implement `CandidType`, `Clone`, `Serialize`, `Deserialize`, and provide default values.

### Trait Implementations

- `Storable` and `BoundedStorable` implemented for `Event`, `User`, and `Ticket`.
  - `Storable`: Conversion to and from bytes.
  - `BoundedStorable`: Defines maximum size and whether the size is fixed.

### Thread-Local Static Variables

- `MEMORY_MANAGER`: Manages virtual memory.
- `ID_COUNTER`: Keeps track of global IDs.
- `EVENT_STORAGE`, `USER_STORAGE`, `TICKET_STORAGE`: Stable BTreeMaps for storing events, users, and tickets.

### Payload Structs

- `EventPayload`, `UserPayload`, `TicketPayload`: Payload data structures for update calls.

### Candid Interface Definitions

- Functions annotated with `ic_cdk::query` are read-only queries.
- Functions annotated with `ic_cdk::update` are updates, which can modify the state.

## Memory Management

Memory is allocated using a `MemoryManager` from the `ic-stable-structures` crate:

```rust
static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = // initialized
```

This manages allocating `VirtualMemory` for storages.

## ID Generation

Unique IDs are generated using a thread-local `IdCell`:

```rust
static ID_COUNTER: RefCell<IdCell> = // initialized
```

The counter is incremented when adding new records.

## Record Storage

Records are stored in thread-local `StableBTreeMap`s:

```rust
static EVENT_STORAGE: RefCell<StableBTreeMap<u64, Event>> = // initialized

```

This provides fast random access to records.

## Main Functions

### Event Functions

- `get_all_events()`: Retrieves all events.
- `get_event(id: u64)`: Retrieves a specific event by ID.
- `create_event(payload: EventPayload)`: Creates a new event.
- `update_event(id: u64, payload: EventPayload)`: Updates an existing event.
- `delete_event(id: u64)`: Deletes an event.

### User Functions

- `get_user(id: u64)`: Retrieves a user by ID.
- `create_user(payload: UserPayload)`: Creates a new user.
- `update_user(id: u64, payload: UserPayload)`: Updates an existing user.
- `delete_user(id: u64)`: Deletes a user.

### Ticket Functions

- `get_ticket(id: u64)`: Retrieves a ticket by ID.
- `create_ticket(payload: TicketPayload)`: Creates a new ticket.
- `update_ticket(id: u64, payload: TicketPayload)`: Updates an existing ticket.
- `delete_ticket(id: u64)`: Deletes a ticket.

### Relationship Functions

- `get_event_attendees(id: u64)`: Retrieves attendees for a specific event.
- `get_user_tickets(id: u64)`: Retrieves tickets owned by a specific user.
- `get_event_tickets(id: u64)`: Retrieves tickets associated with a specific event.
- `remove_user_ticket(payload: TicketPayload)`: Removes a ticket from a user's collection.

## Error Handling

- `Error` enum: Represents errors, particularly the `NotFound` variant used for signaling that a resource with a given ID doesn't exist.

## Candid Interface Export

- `ic_cdk::export_candid!()`: Generates the Candid interface for this canister.

## Note

- The code references external libraries and may be part of a larger project or canister.

To learn more before you start working with e_ticketer, see the following documentation available online:

- [Quick Start](https://internetcomputer.org/docs/quickstart/quickstart-intro)
- [SDK Developer Tools](https://internetcomputer.org/docs/developers-guide/sdk-guide)
- [Rust Canister Devlopment Guide](https://internetcomputer.org/docs/rust-guide/rust-intro)
- [ic-cdk](https://docs.rs/ic-cdk)
- [ic-cdk-macros](https://docs.rs/ic-cdk-macros)
- [Candid Introduction](https://internetcomputer.org/docs/candid-guide/candid-intro)
- [JavaScript API Reference](https://erxue-5aaaa-aaaab-qaagq-cai.raw.icp0.io)

If you want to start working on your project right away, you might want to try the following commands:

```bash
cd e_ticketer/
dfx help
dfx canister --help
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

If you are making frontend changes, you can start a development server with

```bash
npm start
```

Which will start a server at `http://localhost:8080`, proxying API requests to the replica at port 4943.

### Note on frontend environment variables

If you are hosting frontend code somewhere without using DFX, you may need to make one of the following adjustments to ensure your project does not fetch the root key in production:

- set`DFX_NETWORK` to `production` if you are using Webpack
- use your own preferred method to replace `process.env.DFX_NETWORK` in the autogenerated declarations
  - Setting `canisters -> {asset_canister_id} -> declarations -> env_override to a string` in `dfx.json` will replace `process.env.DFX_NETWORK` with the string in the autogenerated declarations
- Write your own `createActor` constructor
