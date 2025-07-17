# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repository contains the **Fractal Sortition Bot** project - a prototype governance bot for OpenChat that implements a novel fractal sortition governance mechanism. The project includes:

1. **OpenChat Platform** - Full-featured decentralized chat application running on Internet Computer
2. **OpenChat Bot SDK** - Multi-language SDKs for building bots that integrate with OpenChat
3. **Fractal Sortition Bot** - A governance experiment combining random selection with peer evaluation

## Fractal Sortition Bot Architecture

The Fractal Sortition Bot implements a governance process that:
- Randomly groups community members into small discussion clusters (3-5 people)
- Facilitates peer voting within groups to select representatives
- Uses iterative rounds to narrow down candidates ("PlayUp" stages)
- Randomly selects final delegate from shortlisted candidates
- Provides accountability through non-confidence voting mechanisms

**Key Components:**
- **Motoko Canister**: Core bot logic running on Internet Computer
- **OpenChat Integration**: Uses OpenChat Motoko SDK for chat interactions
- **Governance State**: Manages volunteers, groups, votes, and delegate terms
- **Command System**: Chat-based commands for user interaction

## Development Commands

### OpenChat Platform Setup

**Prerequisites:**
- DFX 0.28.0-beta.1: `DFX_VERSION=0.28.0-beta.1 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"`
- Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Node.js with npm
- Motoko compiler (via DFX)

**Local Development:**
```bash
# Start DFX
dfx start --clean

# Deploy OpenChat locally
cd open-chat
./scripts/deploy-local.sh

# Start frontend development server
npm --prefix frontend run dev

# Access at http://localhost:5001/
```

### Bot Development

**Bot Implementation:**
- Language: Motoko
- Platform: Internet Computer
- SDK: OpenChat Motoko SDK (`openchat-bot-sdk` via MOPS)

**Bot Commands:**
- `/volunteer` - Opt-in for next PlayUp round
- `/status` - Shows current governance stage
- `/vote @user` - Cast peer vote in group
- `/petition` - Start non-confidence petition
- `/sign` - Sign active petition
- `/delegate-action` - Action only current delegate can perform

**Development Workflow:**
```bash
# Install dependencies
mops install

# Build bot canister
dfx build

# Deploy bot locally
dfx deploy

# Test bot in local OpenChat
# Register bot using /register_bot command in OpenChat
```

## Code Architecture

### OpenChat Platform

**Backend (Rust):**
- **Canisters**: Microservices architecture with individual canisters for users, groups, storage, etc.
- **Libraries**: Shared utilities (chat_events, stable_memory, types, etc.)
- **Integration Tests**: End-to-end testing using PocketIC
- **External Integrations**: IC ledgers, SNS, external services

**Frontend (TypeScript/Svelte):**
- **Workspaces**: Modular packages (app, agent, client, shared, worker)
- **Build System**: Turbo monorepo with Vite/Rollup
- **UI Framework**: Svelte 5 with TypeScript

### Fractal Sortition Bot

**Core Data Models:**
```motoko
type User = Principal;
type GroupId = Nat;
type Volunteer = { user: User; timestamp: Time };
type Group = { id: GroupId; members: [User]; votes: Map<User, User> };
type Petition = { initiator: User; signers: Set<User> };

// State variables
var volunteers: [Volunteer];
var groups: Map<GroupId, Group>;
var shortlist: [User];
var currentDelegate: ?(User, Time);
var activePetition: ?Petition;
```

**Process Flow:**
1. **Volunteer Stage**: Users opt-in via `/volunteer` command
2. **PlayUp Stage**: Random grouping when ≥9 volunteers (5 per group ideal, 3 minimum)
3. **Discussion**: Groups discuss for 2 hours with video calls
4. **Voting**: Peer voting within groups (`/vote @user`)
5. **Shortlist**: Top-voted from each group advance
6. **Sortition**: Random selection from shortlist (minimum 3 candidates)
7. **Delegate Term**: 24-hour governance term
8. **Accountability**: Non-confidence petitions via `/petition` and `/sign`

## Testing Strategy

**OpenChat Tests:**
```bash
# Run integration tests
cd open-chat
./scripts/run-integration-tests.sh

# Run frontend tests
npm --prefix frontend run verify
```

**Bot Testing:**
- Unit tests for governance logic
- Integration tests with OpenChat platform
- End-to-end governance flow testing

## Key Files and Directories

**OpenChat Core:**
- `open-chat/dfx.json` - IC canister configuration
- `open-chat/Cargo.toml` - Rust workspace configuration
- `open-chat/frontend/package.json` - Frontend build configuration
- `open-chat/scripts/` - Deployment and management scripts

**Bot SDK:**
- `open-chat-bots/motoko/` - Motoko SDK and examples
- `open-chat-bots/schema/` - Bot definition schema
- `open-chat-bots/motoko/examples/` - Example bot implementations

**Project Documentation:**
- `Fractal_Sortition_Bot_Overview.md` - High-level project overview
- `Fractal_Sortition_Spec.md` - Detailed technical specification

## Code Style

**Motoko:**
- Follow IC Motoko style guidelines
- Use stable variables for persistent state
- Implement proper error handling
- Use system functions for time management (heartbeat)

**Rust (OpenChat):**
- Use `cargo fmt` (configured in `backend/rustfmt.toml`)
- Max line width: 128 characters
- Workspace-based dependency management

**TypeScript (Frontend):**
- Turbo-based monorepo with workspaces
- Svelte 5 component patterns
- Consistent import/export conventions

## Development Workflow

1. **Setup**: Install DFX, Rust, Node.js, and start local OpenChat
2. **Bot Development**: Implement bot logic in Motoko using OpenChat SDK
3. **Local Testing**: Deploy bot to local IC and test in OpenChat
4. **Integration**: Register bot in OpenChat using `/register_bot`
5. **User Testing**: Test governance flow with multiple users
6. **Deployment**: Deploy to IC mainnet when ready

## Governance Process Details

**Time Management:**
- Uses `Time.now()` for scheduling
- Heartbeat function for automated stage transitions
- 2-hour discussion windows
- 24-hour delegate terms

**Randomness:**
- IC system randomness via `Random.rand()`
- Transparent on-chain randomness for group formation
- Final delegate selection from shortlist

**Security:**
- Validate OpenChat command origins
- Enforce user permissions and voting rules
- Prevent state manipulation and command replay
- Resource management and cleanup after cycles