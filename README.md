# Squvd

Squvd is an on-chain governance system designed for decentralized organizations, featuring secure multisig management, proposal creation, and voting mechanisms. Built using Rust and the Pinocchio framework, Squvd enables trusted group decision-making with robust state management.

## Features

- **Multisig Management:** Initialize a multisig configuration with up to 10 members, enabling secure shared ownership and treasury management.
- **Proposal System:** Members can create proposals, specifying voters and expiration times. Each proposal is tracked with a unique ID and status.
- **Voting Mechanism:** Eligible members can cast votes on active proposals. Votes are recorded and tracked, ensuring each member votes once per proposal.
- **Stateful Governance:** All actions and states (multisig, proposal, vote) are stored and validated on-chain for transparency and auditability.
- **Treasury Account:** Automatically creates and manages a treasury system account as part of multisig setup.

## How It Works

1. **Initialize Multisig:**
   - A creator sets up a multisig wallet and treasury, specifying up to 10 member public keys.
   - The multisig account is securely created on-chain.

2. **Create Proposal:**
   - A multisig member creates a proposal, which is initialized with voters, status (`Active`), and expiration time.
   - Proposals are tracked using Program Derived Addresses (PDAs) for security.

3. **Vote on Proposal:**
   - Eligible voters (multisig members) can vote `Yes (1)` or `No (0)` on active proposals before expiration.
   - Each voter’s participation is tracked in a global `VoteState` PDA, ensuring single-vote enforcement.

4. **Proposal Lifecycle:**
   - Proposals transition from `Active` to `Failed` (if expired without quorum) or to other statuses based on voting results.

## Code Structure

- `src/state/`: Contains core state definitions for multisig, proposal, and vote.
- `src/instructions/`: Logic for initializing multisig, creating proposals, and voting.
- `src/lib.rs`: Program entrypoint and instruction routing.

## Getting Started

> **Prerequisites:** Rust, Pinocchio framework, and access to a Solana-compatible environment.

1. Clone the repository:
   ```sh
   git clone https://github.com/raunit-dev/Squvd.git
   cd Squvd
   ```

2. Build the program:
   ```sh
   cargo build-bpf
   ```

3. Deploy to your preferred Solana cluster.

## Usage

- **Initialize Multisig:** Call the `process_initalize_multisig_instructions` with required accounts and member keys.
- **Create Proposal:** Use `process_initialize_proposal_instruction` as a valid multisig member.
- **Vote:** Call `process_vote_instruction` with your signature and vote value.

## Security

- Only authorized multisig members can create proposals and vote.
- All account creations and state transitions are validated with strict checks and program-derived addresses.

## Contributing

Contributions are welcome! Please open issues or pull requests for bug fixes, feature requests, or improvements.

## License

MIT License

---

**Author:** [raunit-dev](https://github.com/raunit-dev)
