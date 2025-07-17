# 🧠 Fractal Sortition Bot for OpenChat (Prototype)


## 📖 Overview

**Fractal Sortition Bot** is a prototype governance experiment designed to run within **OpenChat**, a decentralized messaging platform on the Internet Computer. The goal is to explore new mechanisms for fair, scalable, and peer-driven leadership selection using small-group deliberation and random selection — inspired by the process known as **Fractal Sortition**.

### 🔍 What is Fractal Sortition?

Fractal Sortition is a novel approach to decentralized governance that combines:

- **Randomized grouping** of community members into small clusters (e.g., 3–5 people),
- **Deliberative discussion** within each group on predefined governance topics,
- **Peer voting** to select the most representative or capable individual from each group,
- **Iteration** across multiple rounds, narrowing down candidates,
- And finally, **random sortition** — selecting a leader from the final pool of peer-nominated individuals.

This method is inspired by principles of deliberative democracy and aims to reduce popularity contests, token plutocracy, and passive voting behavior often seen in typical DAO systems.

### 💬 Why OpenChat?

OpenChat provides the perfect testbed for this concept due to:

- Its decentralized nature,
- Built-in group chat and video chat functionality,
- Support for custom bots via a powerful on-chain SDK,
- And its upcoming **DecideID** integration — a proof-of-personhood system for verifying unique identities.

By building directly on OpenChat using its **Motoko SDK**, we can prototype a fully on-chain governance layer that encourages grassroots leadership and scalable democratic engagement — all within a live chat platform.

### 🛠️ What This Bot Will Do

The Fractal Sortition bot will:

- Allow users to volunteer as candidates through a simple chat command.
- Randomly assign volunteers into small discussion groups (e.g. 5 per group).
- Provide each group with questions and instructions for timed discussion (e.g. 2 hours).
- Facilitate peer voting within groups to identify promising leaders.
- Build a **shortlist** of top-voted individuals.
- Randomly select one from this shortlist to serve as **delegate** for a 24-hour term.
- Allow users to initiate and vote on **non-confidence petitions** to remove ineffective delegates.
- Rinse and repeat — enabling continuous, emergent governance.

This is a proof-of-concept experiment to evaluate how such a process performs in a real-world social setting, using real-time tools and a decentralized identity system.


## 🎯 Purpose

Prototype a decentralized and iterative leadership selection process within an OpenChat channel using **Fractal Sortition** — involving group discussions, peer selection, shortlist creation, and final random leader selection with accountability mechanisms (non-confidence votes).



---

## 📌 Core Concepts

| Concept             | Description                                                                 |
|---------------------|-----------------------------------------------------------------------------|
| **Volunteer**        | A user opts in via the bot to participate in the next Fractal PlayUp stage. |
| **Fractal PlayUp**   | Small groups discuss/debate; peer-vote selects 1 potential leader per group.|
| **Shortlist**        | The pool of candidates selected in PlayUp stages.                            |
| **Sortition**        | Random selection of 1 delegate from the shortlist for a fixed term.          |
| **Petition/Vote**    | Mechanism to challenge/remove delegate.                                     |

---

## 🛠️ Functional Specification

### 1. User Interaction Flow

| Command               | Description                                                                 |
|------------------------|-----------------------------------------------------------------------------|
| `/volunteer`           | Opt-in for next PlayUp round. Requires DecideID.                            |
| `/status`              | Shows current stage: open volunteers, active PlayUp, voting, shortlist, etc.|
| `/vote @user`          | Cast vote for 1 peer in your group (cannot vote for self).                  |
| `/petition`            | Starts a non-confidence petition.                                          |
| `/sign`                | Adds signature to an active petition.                                       |
| `/delegate-action`     | Dummy action only the current leader can perform.                           |

---

### 2. Fractal PlayUp Stage

- **Trigger**: When `≥9` users have volunteered.
- **Group Size**:
  - 5 per group ideally.
  - 3 per group if only 9–11 users.
- **Bot Actions**:
  - Randomly assign members to groups.
  - Create group chat with instructions to initiate a video call.
  - Post discussion questions (static list).
  - Set a 2-hour countdown.
  - Open voting window at the end of discussion time.

---

### 3. Voting in Groups

- **Voting rules**:
  - One vote per participant.
  - Cannot vote for self.
  - Tie → bot randomly picks among top voted.
- **Bot command**: `/vote @user`
- **After voting ends**:
  - Top-voted from each group added to the **shortlist**.

---

### 4. Sortition Stage

- **Trigger**: When at least 3 people are in the shortlist.
- **Action**:
  - Randomly select 1 delegate from shortlist.
  - Announce with timestamp (term = 24 hours).
  - Track term expiry with block timestamp or wall time.

---

### 5. Petition & Non-Confidence Vote

- **Start Petition**: `/petition`
- **Sign Petition**: `/sign`
- **Thresholds**:
  - Needs 10% of total channel members to trigger a vote.
  - Vote passes with ≥51% approval → leader removed.
- **After Vote Passes**:
  - Randomly select new leader from existing shortlist.

---

## 🧱 Technical Specification

### Bot Implementation

- **Language**: Motoko  
- **Platform**: Internet Computer  
- **OpenChat SDK**: Use `motoko` bot examples as base

### Persistence (Canister State)

```motoko
type User = Principal;
type GroupId = Nat;

type Volunteer = { user: User; timestamp: Time };
type Group = { id: GroupId; members: [User]; votes: Map<User, User> };
type Petition = { initiator: User; signers: Set<User> };

var volunteers: [Volunteer];
var groups: Map<GroupId, Group];
var shortlist: [User];
var currentDelegate: ?(User, Time); // (delegate, timestamp)
var activePetition: ?Petition;
```

### Time Handling

* Use `Time.now()` (Motoko’s `Int`) for scheduling.
* Voting and petition expiry handled via timestamps and periodic checks.

---

## 🧑‍💻 Admin & Debug Tools

| Command             | Purpose                                 |
| ------------------- | --------------------------------------- |
| `/debug volunteers` | View current volunteers                 |
| `/debug groups`     | Inspect active group assignments        |
| `/debug shortlist`  | View current shortlist                  |
| `/debug delegate`   | View active delegate and remaining time |

---

## 🎨 UX & Messaging Templates

| Event                         | Bot Message Example                                                                   |
| ----------------------------- | ------------------------------------------------------------------------------------- |
| PlayUp round triggered        | 🌀 *Fractal PlayUp Round 1 started!* You've been placed in Group A. Start discussion. |
| Voting opens                  | 🗳️ Voting is now open! Use `/vote @user` to cast your vote.                          |
| Shortlist formed              | ✅ 5 shortlisted candidates have been selected. Preparing for Sortition...             |
| Delegate selected             | 🎉 New Delegate: @alice! Their term lasts until July 8, 18:00 UTC.                    |
| Petition launched             | ⚠️ Petition of Non-Confidence launched. 3/10 signatures needed.                       |
| Vote of Non-Confidence passed | ❌ Delegate @alice removed. New delegate selected: @bob.                               |

---

## ⚠️ Edge Case Handling

| Scenario                      | Solution                                                                |
| ----------------------------- | ----------------------------------------------------------------------- |
| <9 volunteers                 | Notify: "Waiting for more volunteers to start PlayUp stage."            |
| Tie in votes                  | Randomly choose among top-voted candidates.                             |
| All shortlist members decline | No leader is selected. Re-run Fractal PlayUp when enough volunteers.    |
| Petition spam                 | Limit 1 active petition per term. Optional cooldown time per initiator. |

---

## 🔮 Stretch Goals (Future Enhancements)

* Anonymous voting using zk-proofs or DecideID extensions.
* Integration with OpenChat calendar or reminders for session times.
* Dynamic group sizes beyond 3 or 5, based on active participation.
* Voting interface via inline components (if OpenChat supports it).
