# Fractal Sortition Bot

A prototype governance bot for OpenChat that implements a novel fractal sortition governance mechanism combining random selection with peer evaluation.

## 🔍 Overview

The Fractal Sortition Bot is an experimental implementation of the governance system described in "[Revolutionising DAO Governance with Fractal Sortition](https://medium.com/freedao/revolutionising-dao-governance-with-fractal-sortition-ff18fdda8692)". This innovative system is designed to fairly and efficiently select delegates in decentralized communities, addressing common issues in DAO governance such as token plutocracy, popularity contests, and passive voting by combining:

- **Random Selection (Sortition)** - Ensures equal opportunity for all members
- **Peer Evaluation** - Incorporates merit through small-group deliberation
- **Iterative Vetting** - Multiple rounds of peer selection refine candidates
- **Accountability** - Non-confidence voting mechanisms maintain delegate responsibility

## 🎯 How It Works

1. **Volunteer Phase**: Community members opt-in to participate
2. **Random Grouping**: Volunteers are randomly assigned to small groups (3-5 people)
3. **Deliberation**: Groups discuss governance topics and evaluate each other
4. **Peer Voting**: Each group selects their best candidate to advance
5. **Iterative Rounds**: Process repeats until a final shortlist emerges
6. **Final Sortition**: Random selection from the vetted shortlist
7. **Accountability**: Community can remove delegates via non-confidence votes

## 🛠️ Technical Architecture

- **Platform**: Internet Computer (IC) blockchain
- **Language**: Motoko for bot implementation
- **Integration**: OpenChat messaging platform
- **SDK**: OpenChat Bot SDK for seamless chat integration

## 🚀 Quick Start

### Prerequisites

- DFX 0.28.0-beta.1
- Rust toolchain
- Node.js and npm
- Motoko compiler

### Setup OpenChat Locally

```bash
# Start DFX
dfx start --clean

# Deploy OpenChat platform
cd open-chat
./scripts/deploy-local.sh

# Start frontend
npm --prefix frontend run dev
```

### Bot Commands

- `/volunteer` - Join the next governance round
- `/status` - Check current process stage
- `/vote @user` - Vote for a peer in your group
- `/petition` - Start a non-confidence petition
- `/sign` - Sign an active petition

## 📁 Repository Structure

```
fractal_sortition_bot/
├── README.md                           # This file
├── CLAUDE.md                           # Development guide for AI assistants
├── Fractal_Sortition_Bot_Overview.md   # Project overview
├── Fractal_Sortition_Spec.md          # Technical specification
├── open-chat/                          # OpenChat platform code
│   ├── backend/                        # Rust canisters
│   ├── frontend/                       # TypeScript/Svelte UI
│   └── scripts/                        # Deployment scripts
└── open-chat-bots/                     # Bot SDK and examples
    ├── motoko/                         # Motoko SDK (for this project)
    ├── rs/                             # Rust SDK
    └── ts/                             # TypeScript SDK
```

## 🔧 Development

### Bot Development

```bash
# Install Motoko dependencies
mops install

# Build bot canister
dfx build

# Deploy locally
dfx deploy

# Register bot in OpenChat
# Use /register_bot command in OpenChat UI
```

### Testing

```bash
# Run OpenChat integration tests
cd open-chat
./scripts/run-integration-tests.sh

# Run frontend tests
npm --prefix frontend run verify
```

## 📖 Documentation

- **[Project Overview](Fractal_Sortition_Bot_Overview.md)** - High-level concept and goals
- **[Technical Specification](Fractal_Sortition_Spec.md)** - Detailed implementation guide
- **[Development Guide](CLAUDE.md)** - Comprehensive development reference

## 🎨 Governance Process

### Phase 1: Volunteer Collection
- Users opt-in with `/volunteer` command
- Minimum 9 volunteers required to start
- Time-limited collection period

### Phase 2: Group Formation
- Random assignment to groups of 3-5 people
- Groups receive discussion questions
- 2-hour deliberation period with video calls

### Phase 3: Peer Selection
- Each group votes for their best candidate
- One vote per person, cannot vote for self
- Ties resolved randomly

### Phase 4: Shortlist Creation
- Top-voted candidates from each group advance
- Minimum 3 candidates required for final selection
- Process can repeat for multiple rounds

### Phase 5: Final Sortition
- Random selection from shortlist
- 24-hour delegate term
- Delegate gains special permissions

### Phase 6: Accountability
- Non-confidence petitions available
- 10% of members needed to trigger vote
- 51% approval required to remove delegate

## 🔐 Security Features

- **OpenChat Integration**: Secure command validation
- **IC Randomness**: Transparent on-chain random selection
- **Time Management**: Automated stage transitions
- **Access Control**: Role-based permissions
- **State Persistence**: Stable memory for upgrades

## 🤝 Contributing

This is an experimental governance prototype. Contributions welcome for:

- Bot implementation improvements
- UI/UX enhancements
- Additional governance mechanisms
- Security audits and testing
- Documentation improvements

## 📄 License

This project is an independent implementation using the OpenChat SDK. The included OpenChat platform and SDK code retain their original licenses:
- OpenChat platform code: [AGPLv3](open-chat/LICENSE)
- OpenChat Bot SDK: [MIT](open-chat-bots/LICENSE)

## 🔗 Links

- [Fractal Sortition Original Article](https://medium.com/freedao/revolutionising-dao-governance-with-fractal-sortition-ff18fdda8692) - The governance concept this project implements
- [OpenChat Platform](https://oc.app)
- [Internet Computer](https://internetcomputer.org)
- [Motoko Language](https://internetcomputer.org/docs/current/motoko/main/motoko)
- [OpenChat Bot Documentation](open-chat-bots/README.md)

## ⚠️ Disclaimer

This is experimental software implementing a novel governance mechanism. Use at your own risk. The fractal sortition concept is still being researched and tested in real-world scenarios.