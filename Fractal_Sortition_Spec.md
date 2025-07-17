Perfect. I’ll create a comprehensive Markdown Technical Specification document for the Fractal Sortition Bot prototype. It will include implementation details for the Motoko canister, integration with OpenChat, full UX flows, command references, data models, API logic (where applicable), as well as user stories with clear acceptance criteria.

I’ll let you know as soon as the full document is ready for review.


# Fractal Sortition Bot – Technical Specification

## Overview and Project Purpose

The Fractal Sortition Bot is a prototype governance bot designed to facilitate a novel **fractal sortition** process within an OpenChat group on the Internet Computer. The primary purpose of this project is to empower decentralized communities (DAOs) to fairly and efficiently select governance delegates by combining **random selection (sortition)** with **peer evaluation (fractal democracy)**. Traditional token-voting governance can lead to plutocracy and bias, whereas sortition ensures fairness by giving every member an equal chance to participate in decision-making. By leveraging fractal sortition, this bot aims to provide a **meritocratic yet random** leadership selection that mitigates concentrated power, encourages diverse participation, and holds leaders accountable to the community.

Key objectives of the Fractal Sortition Bot include:

* **Democratic Fairness:** Use random selection to ensure all members have an equal opportunity to become leaders, bringing in diverse perspectives and preventing wealth-based influence.
* **Meritocratic Vetting:** Incorporate a multi-round “fractal” process where candidates are evaluated in small groups by peers, so that those advancing have demonstrated competence and trustworthiness.
* **Accountability:** Introduce a **non-confidence vote** mechanism that allows the community to remove a selected delegate if they lose support, embodying the principle “*sortition selects and democracy deselects*”.
* **Seamless Integration:** Operate entirely within **OpenChat** – a decentralized chat application on the Internet Computer – using the OpenChat Motoko SDK. Community members interact with the bot through chat commands, making the governance process accessible and user-friendly without leaving the chat environment.
* **Transparency:** Provide clear status updates and records of each stage (participants, group assignments, votes, outcomes) so that the process is auditable and trusted by the community.

Overall, this project serves as a template for decentralized governance automation. It is intended to be open-source and collaborative – developers on GitHub can contribute to the code and technical design. This **technical specification** acts as a comprehensive guide for contributors, detailing how the bot works, its architecture, data models, and the logic driving the fractal sortition process.

## Fractal Sortition Governance – Concept and Goals

**Fractal sortition** is an innovative governance model that builds upon classical sortition (random selection) by adding layered peer evaluation, inspired by the concept of *fractal democracy*. The governance goals of fractal sortition are to **select leaders who are both randomly chosen and highly competent**, thereby blending fairness with meritocracy.

In a fractal sortition process, the following occurs:

* **Random Group Formation:** A pool of candidates (typically volunteers from the community) is randomly divided into small groups. This randomness ensures no pre-selected blocs or biases – each group is a microcosm of the larger community.
* **Peer Discussion & Evaluation:** Within each small group, members engage in discussion or Q\&A. They assess each other’s insight, expertise, and leadership ability on relevant topics. The group then **votes to choose one member** who they believe is best suited to advance as a delegate from that group. This *fractal* step ensures that those who move up have earned the respect of their peers through merit and collaboration.
* **Iterative “PlayUp” Stages:** The selection process is **iterative**, referred to as the **PlayUp process**. In each round, the winners from the previous round are again randomly assigned into new small groups (i.e., they “play up” to the next level). They repeat the discussion and peer-vote process. Round by round, the pool of candidates is **refined to a smaller, highly vetted group**. This multi-level vetting means each finalist has passed through several peer evaluations, proving their competence and trust within different groups.
* **Final Sortition:** Once the pool has been narrowed to a final group of qualified candidates (for example, the last 5 candidates after several rounds), the **final selection is made randomly** from this shortlist. By selecting at random from a group of proven capable individuals, the process maintains fairness and unpredictability while **ensuring the selected leader meets a minimum standard of competence**. This is the essence of “meritocratic randomness.”
* **Governance Crew and Roles:** The outcome of fractal sortition can be a single delegate (e.g., a community representative) or a small **governance committee** if multiple winners are chosen. In this prototype, we focus on selecting one delegate at a time (for a defined term), but the process could be extended to choose multiple leaders or fill multiple roles by repeating it or adjusting the final selection to pick several names.
* **Accountability via Non-Confidence:** A crucial aspect of fractal sortition governance is that it doesn’t end at selection. The chosen delegate(s) remain **accountable** to the community through a **vote of non-confidence** mechanism. Members can petition to remove a delegate if they are underperforming or acting against the community’s wishes. Initially, right after election, removing a delegate is intentionally difficult (requiring a high threshold of support to trigger), preventing knee-jerk ousters. However, as the delegate’s term progresses, the threshold to trigger a non-confidence vote becomes lower (making removal easier later in the term). This ensures leaders can govern with stability initially but must maintain community support over time. In short, “**sortition selects and democracy deselects**” – random selection puts leaders in power, and democratic consensus can remove them.
* **Petition and Replacement:** If a delegate is removed by a successful non-confidence vote (or when their term expires), the system can **seamlessly replace** them from the previously vetted candidates without needing an entire new election. Specifically, the **previous final shortlist** (those who made it to the last round but were not picked) can be drawn upon for a replacement. Since these individuals were finalists, they are already trusted as capable, ensuring minimal disruption in leadership continuity. This replacement could be done by randomly selecting from the remaining finalists or by using the runner-up if the final round had a ranking.

**Governance Goals Summary:** Fractal sortition aims to achieve **fairness, competence, and accountability** in decentralized leadership selection. By involving random chance at the start and end, it prevents centralization of power and **encourages diverse participation**. By involving peer evaluation in the middle, it ensures **merit and community trust** factor into who gets to lead. And by allowing the community to remove leaders via established procedures, it avoids unchecked power accumulation. These principles are directly aligned with DAO values of transparency, inclusivity, and collective empowerment.

The Fractal Sortition Bot encapsulates this governance model in an automated workflow. The bot will manage the process from start to finish – from opening nominations, to random grouping, facilitating votes, announcing results, and tracking the current delegate’s status. The ultimate goal is to **provide DAOs using OpenChat with a ready-to-use tool for better governance**, one that can be adjusted and improved as the model is tested in practice (acknowledging that fractal sortition is a cutting-edge concept, currently proposed and experimental).

## Technical Architecture

### Overall Architecture and Components

The Fractal Sortition Bot is implemented as a **smart contract canister** on the Internet Computer, written in Motoko. It utilizes the **OpenChat Bot SDK for Motoko** to interface with the OpenChat application. This means the bot runs on-chain (as a canister) and interacts with OpenChat through defined APIs and message-passing mechanisms, rather than being an off-chain service. Contributors should familiarize themselves with Internet Computer canister development and the OpenChat bot framework. Key architectural components include:

* **Motoko Canister (Bot Logic):** The core of the system is a Motoko actor (canister) that contains all the logic for fractal sortition. This canister will maintain state (participants, groups, votes, etc.) and expose methods that OpenChat calls when bot commands are invoked in chat. The canister logic is modularized into sections for handling different commands and stages of the process (nomination stage, grouping, voting, etc.). State persistence is handled via stable variables (see Data Models section).
* **OpenChat Integration (Bot API):** The bot canister interfaces with OpenChat through the OpenChat bot API. When users type commands in the chat (prefixed with `/`, e.g. `/join`), OpenChat routes those commands to the bot. OpenChat does not host the bot itself; instead, **we register the bot’s canister ID and an endpoint with OpenChat**, so it knows how to reach our canister. For on-chain bots, OpenChat will perform an **inter-canister call** to our canister’s public interface when a relevant command is detected. The Motoko SDK simplifies this by providing traits or base classes to implement.
* **Command Handler Modules:** The bot’s code is organized by commands/feature areas. For example, there might be a module or section for the **Nomination Phase** (handling `/startsortition` and `/join` commands), one for **Group Formation and Voting** (handling internal grouping and `/vote` commands), one for **Petition** (handling `/petition` commands), etc. Each of these implements the logic and updates the shared state accordingly.
* **OpenChat Bot SDK Usage:** Using the `openchat-bot-sdk` (available via MOPS package manager as `openchat-bot-sdk`), the canister implements the required interface for an OpenChat bot. Under the hood, the SDK likely provides message data types and perhaps a dispatcher that maps incoming commands to handler functions. For example, the SDK may allow registering command keywords like `"join"` or `"vote"` and will parse incoming chat messages that start with `/join` or `/vote` and call the corresponding Motoko function. Contributors will implement these handlers following the SDK’s patterns (similar to how an Echo bot example simply returns a message it received).

**Interaction Flow:** When a user in OpenChat sends a message in a group that the bot is installed in, the following happens:

1. **Command Detection:** If the message text matches one of the bot’s registered commands (for instance, exactly `/join` or `/vote <user>`), the OpenChat system recognizes it as a bot command rather than a normal chat message. (The bot must be registered with those command keywords – see Deployment section for how commands are declared.)
2. **OpenChat → Bot Call:** OpenChat invokes the bot canister’s API, passing along details of the command. This likely includes:

   * The **chat ID** (identifying which chat or group the command came from).
   * The **user ID or principal** of the sender (to know who is invoking the command).
   * The **content** of the message after the command keyword (e.g., the username voted for, or any arguments).
     These are packaged in a payload that the bot canister receives via an update call (since it alters state). For canister bots, OpenChat presumably calls a method on the bot canister (the exact method name is abstracted by SDK).
3. **Bot Processing:** The bot’s Motoko code executes the corresponding command handler. It updates the internal state as needed and prepares a response. For example, if `/join` is received, the bot will add that user to the participant list and perhaps respond with a confirmation message. If `/vote @Alice` is received, the bot will record the vote if valid.
4. **Bot → OpenChat Response:** After processing, the bot returns a result. The OpenChat SDK may allow the bot to either:

   * **Reply immediately** via the call’s return value (e.g., returning a text string that OpenChat will post to the chat as the bot’s message).
   * **Or send messages via a `send()` function:** The bot can call an OpenChat message-sending API to post multiple messages or more complex replies. In the OpenChat architecture, bots are allowed to send messages into chats they are installed in, given appropriate permissions (like “send text messages” permission granted during installation in the group). The forum example shows an off-chain actor using a `send(chatId, message)` call. In our on-chain scenario, the SDK likely provides a similar capability, abstracting the inter-canister call needed to post a message.
     In this prototype, we primarily rely on simple text responses to confirm commands and to announce results.
5. **OpenChat Displays Message:** Any message the bot sends (either returned or via send API) will appear in the OpenChat group as posted by the bot (with the bot’s name). For instance, when the bot announces group assignments or final results, all members will see the message from the bot user.

**Canister Structure:** The bot is a single Motoko canister, but we can conceptually break down its internal structure:

* **Public Interface (Candid):** Exposes the methods that OpenChat will call. This includes at least one method that handles incoming chat commands. Depending on the SDK design, this might be a generic `handleMessage(command, args, context)` or distinct methods for each command. Additionally, we may expose some management methods (e.g., for configuration or retrieving state for debugging) – though in production those would be restricted or not used. The candid interface will align with OpenChat’s expected format for bot calls.
* **Internal Logic:** Private functions for each stage of the sortition process: e.g. `beginSortitionCycle()`, `formGroups()`, `tallyVotes()`, `selectFinalWinner()`, etc. The command handlers will delegate to these as needed. This separation keeps the command parsing minimal and the process logic clear.
* **State Storage:** Variables storing the state of an ongoing sortition cycle (see Data Models below). These are likely kept in a **stable variable** (so they persist across canister upgrades). When a cycle is active, state includes the list of participants, their group assignments, votes cast, current stage, etc. When no cycle is active, state holds perhaps the last outcome (current delegate) and is ready to initialize a new cycle.
* **Time Management:** Because the process has timed stages (for nominations, voting windows, etc.), the bot canister needs a way to track time. The Internet Computer provides a system API to get the current time (`Time.now()`) which can be used to enforce deadlines. To automate stage transitions at certain times, the bot can use a *heartbeat* function. **Heartbeat** is a canister method that runs periodically (every few seconds) automatically. We can implement `system func heartbeat()` to wake up and check if any stage timeout has been reached (and then perform the next step, like closing nominations or ending a voting round). This avoids requiring manual intervention to advance stages. The heartbeat will be carefully coded to be very lightweight (just simple time checks) to conserve cycles. If for any reason heartbeat is not desirable, an alternative is requiring an admin to issue a command (like `/advance`) to move to the next stage, but automated timing is preferable for fairness and ease of use.
* **Randomness Source:** The fractal sortition relies on randomness for grouping participants and for the final selection among finalists. The canister will use the Internet Computer’s pseudo-randomness function `Random.rand()` (Motoko: `Experimental.rand()` or similar) which produces a system-provided random value. This ensures group assignments and final picks are unpredictably random (within the constraints of the IC’s consensus randomness). Each time random grouping is needed, the bot will generate a random seed and shuffle or assign participants accordingly. This is done on-chain for transparency (we could log or output the random seed for verifiability if needed).
* **Integration with OpenChat identities:** Each OpenChat user is represented by a principal (or user ID). The bot will receive user information in the command context and use that to identify users in its internal records. OpenChat likely provides the user’s principal and perhaps their username. The bot may store the principal as the unique key for a participant, and possibly keep a display name for friendly output. We assume that if a user changes their username mid-process, it’s rare and we can still refer to their original name or mention their principal short ID to avoid confusion.

Finally, **security** considerations in the architecture: The bot must validate inputs and context for each command:

* Only accept commands from OpenChat (to avoid arbitrary calls to the canister interface). Since OpenChat calls will come from a known canister (the OpenChat backend’s principal) or via an authenticated route, we will ensure that either the SDK filters unauthorized calls or we implement checks (e.g., verifying `msg.caller` is OpenChat if possible). This prevents someone from directly calling the bot canister’s methods outside of the chat context.
* Within the OpenChat context, verify that a user is allowed to perform the action: e.g., only the designated admin can start a sortition cycle; only those who joined can vote; only group members vote within their group; one vote per user; etc. These rules are enforced in the logic using the stored state.
* Ensure that state transitions happen in the correct order (no skipping stages or reopening a closed phase unless via a proper command).
* The bot will also be careful with resource usage: messages and state sizes are relatively small (a few hundred participants at most, basic records), but we ensure to clean up data after a cycle to avoid memory bloat. All critical operations (group creation, random selection, vote counting) happen within a single message call context or a defined sequence, keeping complexity manageable.

### Motoko Canister Structure Diagram (Textual)

To summarize the architecture, below is a logical outline of the canister’s structure and interaction:

* **Actor** `FractalSortitionBot` (Motoko actor using OpenChat SDK)

  * **State:**

    * `currentCycle: SortitionCycle?` – holds an active cycle data (or null if none active).
    * `currentDelegate: DelegateInfo?` – stores the current delegate (e.g., principal, name, term info) if someone is elected and serving.
    * `config: Config` – configuration parameters (e.g., default group size, time durations, thresholds).
    * (All above are stable var to persist across upgrades.)
  * **Public Methods:**

    * `command(entry: CommandRequest) -> async Response` – primary entry called by OpenChat with a command. It parses `entry.command` (e.g. "join", "vote") and routes to the appropriate handler function. It returns either a text response or an empty response if the bot will send multiple messages separately.
    * Possibly individual command methods if the SDK requires (e.g., `join(args)`, `vote(args)` etc.), but likely centralized.
    * Administrative or utility calls (could be behind access control):

      * e.g. `getState() -> StateDump` (for admin debugging),
      * `reset()` to clear state (if needed for dev/testing).
  * **Private Functions:**

    * `startCycle(adminUser)` – initialize a new SortitionCycle (if none active), setting stage = Nomination, deadline = now + signupDuration, etc.
    * `endNominationPhase()` – called when signup deadline passes; freezes participant list and calls group formation.
    * `formGroups()` – groups current participants randomly into groups for this round; advances stage to “Round 1 voting”.
    * `endVotingRound()` – tally votes for current round, determine winners (one per group), store them as next round participants. If more than one participant remains, form new groups (increment round number) and continue. If this was the final round (winners count <= groupSize or reached target finalist count), proceed to final selection.
    * `finalSelection()` – randomly pick the final delegate from finalists; set as currentDelegate; store others as reserve pool; announce result.
    * `handleVote(user, targetUser)` – record a vote for the given target by the given user (with validation such as user is in the correct group and hasn’t voted yet).
    * `handlePetition(user)` – process a petition signature from a user to remove current delegate; add user to petition supporters list if not already; if threshold reached, initiate removal process.
    * `removeDelegate()` – finalize the removal of current delegate (if petition succeeds), pick replacement delegate from reserve or mark position vacant, then announce and possibly trigger end of term processes.
    * Helper functions for time and randomness, e.g.:

      * `now() -> Timestamp` (wrap system time call),
      * `randomShuffle(list) -> list` or `randomPick(list) -> element`.
    * `heartbeat()` – (system function) periodically checks if any timed stage has expired:

      * If nomination phase active and now >= nominationDeadline, call `endNominationPhase()`.
      * If a voting round is active and now >= votingDeadline, call `endVotingRound()`.
      * If a petition is open and now >= petitionExpiration, possibly close it (if threshold not met).
      * etc.

* **External Interaction:**

  * OpenChat calls into `command(...)` for each user message that matches the bot’s commands.
  * The bot calls out via the OpenChat SDK to send messages. E.g., the SDK might provide a `Bot.sendMessage(chatId, text)` which internally calls OpenChat’s API to post the message. Our code will use such calls especially to broadcast announcements (like when forming groups or announcing winners).
  * The bot does *not* call any other external canisters except OpenChat’s messaging. (No calls to NNS or other services in this prototype.)
  * Users and admins only interact through OpenChat messages (no direct calls to canister outside the OpenChat framework, in normal operation).

This architecture ensures that all governance logic runs atomically on-chain (ensuring integrity of the process), while OpenChat provides the user interface in the familiar form of group chat commands and messages.

## OpenChat Bot Commands and User Flow

The Fractal Sortition Bot provides a set of chat commands to interact with the community. These commands are typed by users (prefixed with `/`) in the OpenChat group where the bot is installed. The bot will respond with confirmations or results, guiding users through the governance process. Below is the **command list** with descriptions, followed by a narrative **user flow** illustrating how a typical sortition cycle unfolds via these commands.

### Command List

* **`/startsortition`** – **Start a new sortition cycle.** This command is used to initiate the fractal sortition process. Only authorized persons (e.g., a community admin or a predefined governance facilitator) should use this command. When invoked, the bot checks that no cycle is currently running. If all is clear, it transitions the bot into a **Nomination Stage** (also called PlayUp signup stage). It then announces the start of a new cycle to the group, including instructions on how members can join as candidates. The announcement will typically include the window of time for sign-ups (e.g., “Nominations are now open for the next 24 hours. If you wish to participate, type `/join`.”). The bot sets an internal timer for the nomination deadline.
  *Example:* An admin types `/startsortition` in the group. The bot replies, “**A new Fractal Sortition cycle has begun!** 🎉 Anyone who wants to be considered for selection, please type `/join`. Nominations will close on 2025-08-01 12:00 UTC.”

* **`/join`** – **Nominate oneself as candidate (participation).** Community members use this command during the Nomination Stage to enter themselves into the pool of candidates. Upon receiving `/join`, the bot verifies that a cycle is active and currently accepting nominations. It records the user as a participant (preventing duplicates – if the same user tries to join twice, it will ignore and perhaps inform them they’re already registered). The bot responds with a confirmation, possibly tagging the user, e.g., “You have been added as a participant.” If a user who is not allowed (e.g., after the nomination period ended or not in the community whitelist, if any) tries to join, the bot will respond with an error message.
  *Example:* A user types `/join`. Bot replies, “@Alice has joined the candidate pool (Total participants so far: 5).”

* **`/status`** – **Check the current status of the cycle.** Any user can use this command at any time to get an update on what’s happening. The bot will reply with information such as: which stage is active (e.g., “Nominations open until X time” or “Round 1 voting in progress”), how many participants or groups there are, time remaining in the current stage, etc. This is useful for transparency. If no cycle is active, the bot can respond with “No active governance cycle at the moment. Waiting for an admin to start a new sortition.”
  *Example:* User types `/status`. Bot replies, “**Status:** Nominations open. 12 participants have joined. 3h 20m remaining until grouping.”

* **`/groups`** – **List current group assignments.** This command is primarily relevant during the voting rounds. When groups have been formed for a round, a user (especially participants) can query `/groups` to see the composition of all groups. The bot will list each group and the members in it. The bot already typically announces group assignments when the round begins, but this command allows retrieving the information again on demand. If no groups are currently formed (e.g., during nominations or final selection already done), the bot might respond accordingly (“Groups will be available once the voting round begins” or “No active groups right now”).
  *Example:* During Round 1, user types `/groups`. Bot replies with a structured message: “**Round 1 Groups:** Group 1 – @Alice, @Bob, @Charlie. Group 2 – @Dave, @Eve, @Frank. (Each group will choose 1 representative by voting.)”

* **`/vote <username>`** – **Vote for a representative in your group.** This command is used by participants during a voting round to cast their vote for one member of their group to advance. For example, if Alice, Bob, Charlie are in Group 1, and Alice thinks Charlie is the best candidate, Alice would type `/vote @Charlie`. The bot validates that:

  1. There is an active voting round.
  2. The voter is indeed a member of one of the current groups.
  3. The target of the vote is a member of *the same group* as the voter.
  4. The voter has not already voted in this round.
     If all checks pass, the vote is recorded. The bot usually does **not** reveal interim vote counts publicly (to avoid influencing others), but it might ACK in private or quietly. We may opt to have the bot send a direct confirmation to the voter (e.g., via direct chat if supported) or just respond in-group with a generic confirmation like “Vote recorded from @Alice.” If any check fails, it responds with an error (e.g., “You are not part of any group currently,” or “Voting has closed”). Votes are tallied at the end of the round (when the voting deadline hits or when all group members have voted, whichever comes first).
     *Example:* Alice types `/vote @Charlie`. Bot responds (in group): “@Alice’s vote has been recorded.” If Bob then tries to vote for someone not in his group or after voting ended, bot might respond: “Sorry, you cannot vote at this time or invalid vote target.”

* **`/petition`** – **Sign a petition for a vote of no confidence.** This command comes into play after a delegate has been selected and is serving. Any community member can initiate or sign onto a petition to remove the current delegate. The first time someone uses `/petition` during a delegate’s term, the bot treats it as **initiating a new petition** (if none active). It will record that user’s support and broadcast a message like: “@Frank has started a petition of no-confidence against the current delegate. Type `/petition` to support this petition. Requires at least 10 signatures within 7 days to trigger a removal vote.” (Numbers configurable.) Subsequent `/petition` commands from other users within the allowed petition period will add their signature in support. The bot keeps track of unique supporters (each user counted once). It might periodically update the group: “Petition support: 3/10 signatures.” When the required threshold is reached, the bot confirms that the threshold is met and initiates the removal process (which could either immediately remove the delegate or move to a dedicated confirmation vote stage, see Petition Mechanics section). If a petition is already ongoing and a user who already signed sends `/petition` again, the bot will likely ignore or gently remind “You have already signed the petition.” If the petition period expires without reaching threshold, the petition is closed and labeled as failed (and perhaps a cool-down is enforced before a new one can start).
  *Example:* User types `/petition`. Bot: “@Grace has started a petition to remove Delegate X. (Needs 5 more supporters in the next 14 days.)” Later another user `/petition`, bot: “Petition update: 2 out of 6 required signatures gathered.”

* **`/help`** – **Display help information.** The bot will list all available commands and a brief description of each, similar to the list above. This helps users discover how to interact with the governance process.
  *Example:* User types `/help`. Bot responds with a message enumerating commands: “Commands: `/startsortition` (admins) – start a new election; `/join` – enter yourself as candidate; `/vote <user>` – vote your group’s rep; `/petition` – call for delegate removal; `/status` – show current phase info; `/help` – show this help.”

* **(Optional) `/cancel`** – **Cancel the ongoing cycle.** This is an admin-only command to abort a sortition process in exceptional cases (e.g., if something goes wrong or needs to restart). It would reset the state to no active cycle. The bot would announce that the cycle was canceled. This should be used cautiously; for the spec we note its existence for completeness and error recovery. If invoked, any ongoing votes or nominations are discarded.
  *Example:* Admin types `/cancel`. Bot: “The current sortition cycle has been **canceled**. No delegate was selected. (Admins may start a new cycle with /startsortition.)”

**Note:** In the OpenChat registration, we will list the main command triggers (e.g., `/startsortition`, `/join`, `/vote`, `/petition`, `/status`, `/help`). OpenChat uses this to know which messages to route to the bot. All commands should be entered without arguments except where noted (`/vote` expects a username argument). The bot will ignore messages that are not commands or not relevant (so normal discussion in the chat won’t trigger the bot). This means community members can still chat normally during the process – e.g., groups can discuss in the chat, only when they actually type the command `/vote ...` does the bot intervene.

### User Experience Flow

The following describes the typical user flow through a full fractal sortition cycle, from initiation to final outcome, including the PlayUp rounds and post-selection petition. It’s divided into stages for clarity:

**1. Initiation (Nomination Stage Begins):** A governance admin or moderator triggers the process by typing **`/startsortition`** in the OpenChat group. The bot verifies no other cycle is running and that the user has permission (the bot can maintain a list of authorized principals for this command). Upon success, the bot transitions to *Nomination Stage*. It posts a clear announcement to the group chat, for example:

> *Bot:* “**🚀 Fractal Sortition cycle started!** Community members who wish to stand as candidates for the next delegate position, please type `/join` in the next **48 hours**. After that, we will begin the selection rounds. For more info, type `/help`. (Cycle ID: 7)”

The message may include the nomination deadline (calculated as current time + configured nomination window) so everyone knows the timeframe. At this point, the bot’s internal state has `cycle.active = true` and is accumulating participants. The admin who started the cycle might also receive a confirmation (either via the group message or a DM if configured) that the process has started successfully.

**2. Nomination/Sign-Up Stage (PlayUp initial):** Once the cycle is announced, any user in the community group can self-nominate by sending **`/join`**.

* When a user sends `/join`, the bot checks that nominations are indeed open (current stage = Nomination and now < nominationDeadline). If valid, the bot records this user as a participant in the cycle. It then typically acknowledges in the chat. The acknowledgment could be minimal to avoid spamming the chat if many join; e.g., “`@UserXYZ joined the candidates.`” possibly in a single line or by updating a running list. We could have the bot occasionally post an updated count of participants (e.g., every 5th join or on request via `/status`). For example:

  * Alice: `/join`
  * Bot: “Alice has joined as a candidate. (Total candidates: 1)”
  * Bob: `/join`
  * Bot: “Bob has joined as a candidate. (Total candidates: 2)”
  * ... and so on.
* If someone who already joined sends `/join` again, bot might respond, “You are already in the candidate pool.”
* If someone tries to join after the deadline or when not in Nomination stage, bot responds with something like, “Nominations are closed; you cannot join at this time.”

During this stage, participants can continue normal discussion in the group. They might lobby or share their interest in being a delegate, but formal evaluation hasn’t started yet. Everyone can see who has joined by either reading bot’s join confirmations or using `/status` which might list current candidates.

The **`/status`** command here would show: e.g., “Stage: Nomination (open). 18 candidates so far. Closes at 18:00 UTC tomorrow.”

The bot’s internal timer (via heartbeat) monitors the deadline.

**3. Group Formation (Round 1 begins):** When the nomination period ends (deadline reached), the bot automatically moves to the next phase. It stops accepting `/join` commands (any further attempts get a “too late” message). Then it calculates how to form groups for Round 1 of the PlayUp process.

* **Group sizing:** Suppose `N` people joined. The bot uses a preset group size (e.g., 5 per group is ideal). It will randomly shuffle the list of participants and then split them into groups of up to 5. If N is not a multiple of 5, one group may have fewer members (e.g., 4 or 3). The bot should ensure no group has less than 2 if possible (a group of 1 would automatically win with no discussion; if that happens due to odd numbers, it’s an edge case the bot will handle by possibly merging small remainder groups or just allowing it with note).
* The bot now announces the **Round 1 groups** in the chat. This announcement is critical and should be well-formatted for clarity. For example:

> *Bot:* “**Round 1:** We have 4 groups. Each group will deliberate and **choose 1 representative** among themselves. The groups are:
> **Group 1:** Alice, Bob, Charlie, Dee, (Emily).
> **Group 2:** Felix, Georgia, Henry, Iris, Jack.
> **Group 3:** Kelly, Liam, Maya, Noah, Olivia.
> **Group 4:** Peggy, Quentin, Raj.
> *(Group 4 has 3 members due to numbers)*.
> **Round 1 Discussion & Voting Window:** 2 hours (until 20:00 UTC). Group members, discuss and then cast your vote using `/vote @name` for who should represent your group.”

The above message shows how the bot lists group membership. It might tag each user if tagging is supported, or just names. The bot also clearly states the duration for this round (which is configured, e.g., 2 hours or 1 day depending on how real-time we expect it).

* **Group discussions:** Now group members are supposed to talk among themselves to decide whom to vote for. Since all are in the same OpenChat group by default, one challenge is that all participants are still in one chat room. Ideally, each group should have a separate discussion space to avoid confusion. In the current OpenChat framework, **the bot cannot automatically create new group chats** for each group (that feature isn’t available to bots yet). As a workaround, we encourage each group to perhaps create a thread or just coordinate times. For the prototype, we assume groups will coordinate in the main chat by perhaps taking turns or using threading (if OpenChat supports threading per message). This is a known limitation and is mentioned in Edge Cases. Regardless, they have a set time to discuss. The bot is not actively moderating content; it just waits for votes.
* Group members, after discussing, each submit their vote via **`/vote <user>`**. For example, in Group 1, maybe Alice votes for Charlie (`/vote @Charlie`), Bob votes Charlie, Charlie votes Alice, Dee votes Charlie, Emily (if present) votes Bob, etc. They can do this at any time during the round before the deadline. The bot records each vote, and typically responds with a quick confirmation in the chat as they come in. It does not reveal the tally yet.
* If a group finishes early (all members have voted) and perhaps one candidate already has majority, the bot might immediately finalize that group’s result to move things faster. However, for fairness and simplicity, the prototype will likely wait until the round’s time is up to collect all votes (or until all votes are in). If all votes are in early, the bot could optionally allow an early end to the round. We may implement a check: if every participant in every group has cast a vote, then end the round early and process results. If even one is missing, let the timer run out (or an admin can manually end with a command if needed).
* **Voting validation:** The bot ensures that users can only vote for someone in their own group. If Bob tries to vote for someone not in his group, bot will reply “@Bob, you can only vote for members of your group (Group 1).” Each user only gets one vote; if Alice tries to vote again, bot will say “You have already voted. You cannot vote twice.”
* **Non-participants**: If some user who isn’t in any group (didn’t join the cycle) tries to `/vote`, the bot will ignore and maybe reply “This command is only for active participants in a group.”

During the voting window, participants and even spectators might use **`/status`** to see progress. The status might show something like: “Stage: Round 1 voting. Groups: 4. Votes received: 12/18 total. Time left: 45 minutes.” It won’t show interim results per group to avoid biasing remaining voters.

**4. Round Conclusion and Advancement:** When the Round 1 deadline arrives (or all votes are in), the bot proceeds to determine the outcome for each group:

* It counts votes group by group. For each group, the person who received the **highest number of votes** is the winner (the group’s representative). If one person has a clear majority or plurality, they win.
* **Tiebreaks:** In case of a tie within a group (common if group has even numbers or scattered votes), the bot resolves it by a predefined rule:

  * If two or more candidates are tied for top votes, the bot can either choose one **randomly among the tied** (to maintain fairness) or, if time permits, prompt the group for a quick tiebreak discussion. The simpler approach implemented is a random tiebreak: the bot will pick one of the tied names at random and declare them the winner. (The randomness here can be seen as an extension of sortition—when the group can’t decide, let chance decide among the top). This should be communicated: “Tie between Alice and Charlie in Group 1 – randomly selected Alice as winner.”
* The bot then creates a list of all group winners. These winners advance to the next round (or to final selection if this was the last round).
* The bot announces the results of Round 1 to the group chat. For example:

> *Bot:* “**Round 1 Results:** Group 1 -> **Alice** wins (3 votes) over Charlie (2 votes). Group 2 -> **Henry** wins (4 votes) over others. Group 3 -> **Maya** wins (5 votes unanimous). Group 4 -> **Raj** wins (2 votes) \[tie broken randomly]. Congratulations to the winners! 🎉 They will advance to the next round.”

It tags or bolds the winners for clarity. It might also mention if any ties were resolved by random choice.

* Next, the bot checks how many winners there are. Suppose there were 4 groups, so 4 winners. If more than one winner exists, it means we need another round unless the final desired number of delegates is more than one. In our case, we want a single final delegate. So with 4 candidates remaining, we proceed to Round 2.

* The bot moves to **Round 2 group formation**. It will randomly group the 4 into, say, groups of 5 again. But with 4 people, likely we just have 1 group of 4 if group size is 5 (if group size is 5 or more, having <=5 means this could be final round). However, fractal sortition often continues until one group remains then picks random final. We have a design choice: we could treat having one group of 4 as the final shortlist already, or we could do one more selection to shortlist say 1 or 2. The original concept indicates continuing until a “small group” remains, then randomly choose from it. Let's follow the concept:

  * We have 4 finalists. We can consider this the final shortlist. The final selection will be random among them. Or we could run Round 2 as a discussion among them to possibly further vet and maybe reduce to, e.g., 1 (which would just pick one by vote, essentially election-like) or maybe 2 finalists. That would conflict with the random final selection idea.
  * Instead, we likely consider the final shortlist to be those 4, and jump to final random selection. However, in fractal sortition, typically you might want the final group to also have a discussion so that even final random selection is among people who've evaluated each other. But if we had just one group of 4, they can still discuss (Round 2 discussion) and then instead of voting one out (which would produce 1 winner immediately), we choose one randomly. The difference is subtle but important: If they were to vote one, then randomness isn't used. If they randomly pick from 4, no further merit filter at that step, but all 4 are presumably already merit-vetted from Round 1.

  For this prototype, to stick to the described method, we will treat the last group as the final and do random pick:

  * So essentially, if after Round 1 the number of winners <= group size, we stop the PlayUp elimination and do sortition on that set.
  * If it was larger, we’d do Round 2 grouping and voting first.
    In our example with 4 winners and group size 5, 4 <= 5 triggers final sortition directly.

* If there were more participants (say 25 initial, 5 groups, 5 winners, then 5 finalists, we might consider 5 as the final shortlist and random pick from them, or do one more round with group size 5 where they all are in one group and would have to pick 1 by consensus or vote, effectively giving 1 without randomness. But fractal sortition specifically emphasizes final random selection to avoid even final politics. So yes, we should random pick from the final 5.)

* Therefore, the bot decides if the next step is another PlayUp round or final selection. Criteria: if `winners_count > group_size`, do another round; if `winners_count <= group_size` or only 1 group can be formed, go to final random selection.

In scenario with 4 winners:

* The bot announces moving to final selection stage. Possibly it could still allow a short discussion among finalists if needed (but in chat context, maybe skip).
* **Final Selection:** The bot uses a random generator to pick one of the finalists as the delegate. Before picking, it might announce: “Finalists (shortlist): Alice, Henry, Maya, Raj. The final delegate will now be randomly selected from these candidates to ensure fairness.” Then it performs the random draw on-chain.
* The bot announces the final result in a celebratory way:

> *Bot:* “🎖️ **Final Result:** Congratulations **Alice**! She has been randomly selected as the new Governance Delegate from the final shortlist. 🎉🎉
> Thank you to all participants. Alice, please prepare to assume your new responsibilities. The other finalists will serve as a reserve pool.”

It should mention the delegate’s name clearly. It may also mention the term duration or next steps (e.g., “Alice’s term is slated to run for 3 months. After that, or in case of removal, a new sortition can be conducted.”).

* The bot updates its `currentDelegate` state to Alice (principal and name, start time etc.). It also stores the list of reserve candidates (Henry, Maya, Raj in this case) in order or as a set. These reserves might be used if needed (see next).
* The cycle is now formally completed. The bot might post a summary or store a record of cycle results. It transitions to an “idle” state waiting for governance actions (like petitions or next cycle eventually).

**5. Post-Selection – Delegate’s Term:** Now the community has a delegate (Alice). The OpenChat group might use this outcome in their broader DAO (for example, giving Alice certain permissions or asking her to make certain decisions). The bot’s role during this period is mostly to handle the accountability mechanism:

* It might occasionally post a reminder of who the delegate is and how to call a non-confidence vote if needed (especially if it’s a long term).
* If configured, the bot could automatically schedule the next sortition cycle at term end (e.g., after 3 months). But in this prototype, we assume a new `/startsortition` will be triggered manually for the next term when the time comes (though we note future automation in Future Considerations).

**6. Petition for Non-Confidence (Accountability Stage):** At some point, if members are unhappy with Alice’s performance, they can initiate a removal. A member starts a petition by sending **`/petition`**.

* If no petition is currently active, the bot treats this as the start. It creates a petition record with that user’s signature. It immediately announces to the group something along the lines of:

> *Bot:* “⚠️ **No-Confidence Petition Initiated:** @Bob has started a petition to remove the current delegate **Alice** from her position. To support this petition, other members can type `/petition`. We need at least 10 members (20% of the group) to sign within 7 days for it to succeed. If the threshold is met, a vote of no-confidence will be triggered (or the delegate will be removed).”

The announcement includes the conditions: required number of signatures and the deadline date/time for gathering them. These conditions come from the configuration. For example, maybe threshold = 10 or 30% of active members, whichever is higher; and time window = say 7 days.

* The bot enters a “petition collecting” sub-state. Now each time another distinct user types `/petition`, if the petition is ongoing and they haven’t signed yet, the bot adds them and updates the count. It might not announce every single signature individually to avoid spam (or it could say “@Charlie has signed the petition (3/10)”). We can aggregate updates, for example every time the count increases, we update the original message or post a brief update: “Petition support: 3/10.” This gives momentum feedback.
* If someone tries to sign twice, bot will ignore and possibly PM them “You have already signed.” If someone tries to petition when one is already active, it just counts as signature.
* If the threshold of signatures is reached **before the deadline**, the petition succeeds:

  * The bot announces: “The petition has reached the required support (10/10 signatures). The no-confidence vote is successful.” Depending on design, at this point either the delegate is immediately removed or a formal vote phase begins. For simplicity, we can assume that reaching the threshold itself is enough to remove (i.e., treat the petition like a direct vote).
  * The bot then proceeds to remove the delegate: it clears `currentDelegate` Alice from its state (or marks as removed) and announces that she is no longer delegate.
  * It then checks if a reserve candidate pool exists from the last cycle (it does: Henry, Maya, Raj). The bot picks the next leader from the reserves. We could either automatically pick one at random from those reserves to become the new delegate or pick the next in some preserved order. The original design implies *randomly draw from shortlist upon a delegate removal to replace them*, which keeps fairness. Alternatively, since all reserves are equal finalists, random is fine. We do that:

    * Bot randomly selects, say, Maya as replacement.
    * Bot announces: “As Delegate Alice is removed, a replacement is selected from the vetted candidates. **Maya** will take over as the new delegate for the remainder of the term.”
    * The bot updates currentDelegate to Maya and possibly removes Maya from reserve list (the others remain as further backup if needed).
  * If no reserves were available (say the shortlist was empty or all used), the bot would recommend starting a fresh `/startsortition` to elect someone new. (This edge case is unlikely unless the previous cycle had only one candidate which was removed.)
* If the petition period ends without reaching threshold (e.g., only 5/10 signatures in 7 days):

  * The bot announces the petition failed: “The no-confidence petition against Alice has expired with insufficient support (5 signatures). Alice remains as delegate.”
  * It may impose a cool-down (maybe no new petition can start for another 7 days or until some condition) to prevent constant spamming. The state resets the petition so a new one can be initiated later.
* Notably, early in Alice’s term, the threshold might be set high (like 66% of members) to discourage immediate petitions, whereas later it could drop (the bot could internally calculate threshold as a function of time passed, if configured). For simplicity in prototype, we might set a fixed threshold or manually adjust it per term stage (config can have e.g. `earlyTermThreshold` vs `lateTermThreshold` and the bot can linearly interpolate or step down after certain days). This detail can be adjusted in config and noted in Future Considerations.

**7. Cycle Completion and Next Cycle:** Eventually, when Alice’s term ends (either naturally or via removal), a new cycle will be conducted (likely manually by admin calling `/startsortition` again for the next election). The bot can handle continuous use: after one cycle is finished, its state is ready for another. It retains knowledge of current delegate (and reserves until term ends), but once a new cycle starts, those reserves are typically obsolete (unless we consider carrying them over which we won’t – each cycle is separate).

Throughout the process, the **user experience** is intended to be interactive and clear:

* The bot guides users at each step with announcements and instructions.
* All critical actions (joining, voting, petitioning) are confirmed by the bot.
* Users can always check `/status` or `/help` to know what to do.
* Edge conditions like errors or invalid commands receive polite error messages, so users know what went wrong (e.g., “You can’t start a cycle now, one is already running.” or “Vote target not recognized.”).

Where possible, the bot uses **mentions and markdown** for readability (if OpenChat supports basic markdown in messages). For example, bold text for key terms, lists for grouping, etc., as shown in examples.

This UX flow demonstrates a single-threaded process. The bot, in this prototype, will **not** run multiple sortitions in parallel (the assumption is one community uses one cycle at a time). If parallel elections for different roles were needed, that’s beyond scope (though the design could be extended to handle role identifiers, etc., see Future Considerations).

Finally, after the cycle, the community hopefully has more trust in the chosen delegate due to the thorough fractal sortition vetting, and the process itself builds community engagement (as many people had to discuss and choose reps in small groups, leading to camaraderie).

## Data Models and State Persistence

To implement the above logic, we define clear data models (data structures) in the Motoko canister. These models represent participants, groups, rounds, and overall cycle state. Equally important, we ensure that the data is persisted in canister memory and survives upgrades using stable storage. Below we describe the key data models and how state is managed:

### Key Data Structures

* **Participant**: Represents a community member who is taking part in the current sortition cycle.

  * **Fields**:

    * `principal: Principal` – the unique principal identifier of the user (their OpenChat identity).
    * `name: Text` – the username or display name of the user (for readable output). This is optional but we cache it when possible for convenience in messages.
    * `groupId: Nat?` – the identifier of the group they are currently in for the ongoing round (if any). This is assigned during grouping. If not in a round (e.g., after elimination or before grouping), it could be null.
    * `hasVoted: Bool` – flag to indicate if this participant has cast their vote in the current round (resets each round).
    * (We do not store sensitive info; just identity and status. No private keys – the OpenChat handles authentication.)

* **Group**: Represents a small group of participants in a given round of the PlayUp process.

  * **Fields**:

    * `id: Nat` – group number or ID (e.g., 1, 2, 3…).
    * `members: [Principal]` – list of participant principals who are in this group.
    * `winner: Principal?` – once voting is done, this may be set to the principal of the member who was chosen to advance.
    * `votes: Map<Principal, Principal>` – a mapping from voter principal to the principal they voted for. (This helps ensure one vote per voter and to tally results. Alternatively, we could just tally counts, but storing the map allows detecting double voting attempts and potentially changing vote if we allowed that before finalizing, though we likely don’t allow changing votes in this prototype.)
    * We may also derive convenience info like `voteCounts: Map<Principal, Nat>` when tallying.

* **SortitionRound**: Represents a round within the fractal sortition cycle.

  * **Fields**:

    * `roundNumber: Nat` – 1 for Round 1, 2 for Round 2, etc. (0 might be used to refer to the Nomination stage in some contexts).
    * `groups: [Group]` – array of Group structs for this round.
    * `stage: Text` – could be "Voting" or "Final" etc., but since we already know it’s a round, maybe not needed beyond identifying final round.
    * `deadline: Time` – the timestamp by which this round’s voting ends.
    * Possibly `nextParticipants: [Principal]` – list of winners who advance (computed at end of round).
    * We might not need a separate struct if we handle round-by-round in code flow, but having a record can be useful for logging or debugging.

* **SortitionCycle**: Represents the overall state of an ongoing sortition process.

  * **Fields**:

    * `cycleId: Nat` – an identifier for the cycle (could be incremented each time a new cycle starts, useful for logging or distinguishing cycles).
    * `status: Text` – e.g., "Nomination", "VotingRound", "Finalizing", "Completed". We can use an enum (or variant type in Motoko) for stage for type safety (e.g., `{ #Nomination; #Voting(round: Nat); #Final; #Completed }`).
    * `participants: Map<Principal, Participant>` – a map of all participants currently in the cycle, for quick lookup. (We use principal as key).
    * `currentRound: SortitionRound?` – details of the round in progress (groups, etc.), or null if between rounds or finished.
    * `finalists: [Principal]` – if we have reached the final shortlist but not yet picked, they would be listed here.
    * `delegateSelected: Principal?` – the principal of the final selected delegate (moves to `currentDelegate` global state once cycle is done, but we keep it here until cycle completes).
    * `reserveCandidates: [Principal]` – list of other finalists who were not chosen (will become the reserve pool for potential replacements).
    * `nominationDeadline: Time?` – if in Nomination stage, the end time.
    * `votingDeadline: Time?` – if in a voting stage, the end time.
    * `admin: Principal` – who initiated the cycle (for reference/permissions).
    * We also include any config overrides for this cycle (e.g., if admin set a custom group size or durations this time).

* **DelegateInfo**: Represents the currently serving delegate (post-selection).

  * **Fields**:

    * `principal: Principal`
    * `name: Text`
    * `termStart: Time`
    * `termEnd: Time?` (if a fixed term duration is set)
    * We may also store `termNumber` or such if needed.
    * This is stored outside cycles as persistent governance state.

* **PetitionState**: Represents an ongoing petition for no-confidence.

  * **Fields**:

    * `petitionActive: Bool`
    * `petitionStartTime: Time`
    * `petitionDeadline: Time`
    * `signers: Set<Principal>` – set of users who have signed the petition.
    * `requiredSigners: Nat` – threshold required.
    * Possibly store `targetDelegate: Principal` (should match currentDelegate at start; if delegate changes mid-petition due to some odd scenario, petition likely invalidated).
    * `stage: Text` – e.g., "collecting", or "voting" if we had a second stage (but we likely treat threshold as the vote itself).

* **Config**: A configuration object for adjustable parameters (either constants or modifiable by admin via code or command).

  * Fields could include:

    * `groupSize: Nat` – desired number of people per group in PlayUp rounds (e.g., 5).
    * `nominationPeriod: Duration` – time that nominations stay open (e.g., 24h or configurable by admin each time).
    * `votingPeriod: Duration` – time for each voting round (maybe shorter, e.g., 1h or 1 day depending on real-time vs async).
    * `maxRounds: Nat` – safety cap on rounds (the process should ideally reduce to <=groupSize in log2 rounds, but just in case).
    * `minParticipants: Nat` – minimum required participants to actually proceed (if fewer join, maybe cancel or auto-choose).
    * `petitionThresholdEarly: Float` and `petitionThresholdLate: Float` – e.g., 0.6 vs 0.5 of membership, or absolute numbers.
    * `petitionPeriod: Duration` – time a petition stays open (e.g., 7 days).
    * `cooldownAfterPetition: Duration` – how long after a failed petition to wait until a new one can start.
    * etc.
    * These could be hardcoded or stored as part of state so they can be updated without redeploying (maybe via a privileged command or config file).

### State Persistence and Stability

The bot’s state must persist beyond the lifecycle of a single message or even canister upgrade. We achieve this via:

* **Stable Variables**: In Motoko, variables declared with the `stable` keyword (or using stable structures pre-upgrade/post-upgrade) will persist through canister upgrades. We will declare critical state as stable:

  * e.g., `stable var currentDelegate: ?DelegateInfo = null;`
  * `stable var cycle: ?SortitionCycle = null;`
  * `stable var petition: PetitionState;`
  * `stable var config: Config;`

  This ensures that if we upgrade the canister (to deploy bug fixes or enhancements), the data is not lost. We will implement the `preupgrade` and `postupgrade` hooks if needed (Motoko automatically preserves stable vars of stable types, but if we have complex types, we might manage serialization explicitly). Contributors must carefully handle any changes to data structure between versions (e.g., if we modify the Participant struct in the future, we need to write upgrade code to migrate old data).

* **Memory usage**: The data sizes are expected to be modest. Even if 100 people participate, Participant map has 100 entries, groups a few dozen. All textual data (names, etc.) and messages don’t persist beyond usage except logs if any. The canister has plenty of capacity for this scale. We will still drop references when not needed (e.g., after a cycle completes, we might archive or null out the cycle data except maybe a summary). For safety, we could implement a mechanism to archive past cycles in a compressed form (just store winner and maybe participant count for record) and free detailed data.

* **Cycle Reset/Cleanup**: Once a cycle ends (delegate chosen), the `SortitionCycle` struct can be either kept around marked as completed, or cleared. We likely set it to null and rely on `currentDelegate` to know a cycle finished successfully. If needed, we could log past cycles in an array for history (e.g., `pastCycles : [CycleSummary]` stable var). For now, not strictly required.

* **Concurrent Access & Data Integrity**: The IC’s model ensures canister calls are serialized (no two calls run at the same time). So we don’t have to worry about race conditions within our single-threaded bot state as long as we handle each command atomically. The main risk would be re-entrancy or external calls mid-update – but our bot likely doesn’t make cross-canister calls within the same message (except maybe sending a message out via OpenChat, but that can be done asynchronously after state commit, or synchronously if the OpenChat call is quick). We will be cautious to update state first, then call external if needed, or handle responses carefully. The OpenChat send message function could be an async call; if so, we ensure state reflecting a message being sent is consistent (though sending a message out of band doesn’t affect our core state besides maybe a log).

* **Testing Data Model**: We will write unit tests (if possible with MOPS) or at least simulate scenarios to ensure state transitions work (e.g., adding participants, forming groups yields expected count, voting yields correct winners, etc.). The data model definitions in code will mirror the structures above.

In summary, the data models are designed to capture all needed information about the governance process and ensure no important information is lost. They allow the bot to enforce rules (by cross-checking who is in which group, who has voted, etc., via these structures). Persisting them reliably means the bot can run long processes (like a 24h nomination or a week-long petition) without issue – even if the canister is upgraded or experiences a node restart, the state remains intact when it resumes.

## Time-Based Flow Control: PlayUp and Sortition Stages

The fractal sortition process is inherently time-based, with distinct phases that occur in sequence. The bot needs to manage stage transitions on a timeline to keep the process moving. This section details how the **PlayUp rounds and Sortition stages are scheduled and controlled over time**:

### Stage Timeline and Durations

A typical cycle timeline can be summarized as follows (durations are examples and can be configured):

1. **Nomination Stage (Sign-up)** – **Duration: e.g., 24 hours to several days.** This is the period after `/startsortition` where members join via `/join`. The duration should be enough to let most active members see the announcement and decide to participate. In more real-time scenarios (like an event-based meeting), this could be minutes, but in an async community maybe a day or two. The specific duration is set in `config.nominationPeriod`. The bot captures the start time when the stage begins (when `/startsortition` is invoked) and computes `nominationDeadline = startTime + nominationPeriod`. This deadline is announced. The bot’s heartbeat will regularly check the current time; once it passes the deadline, the Nomination stage automatically closes and transitions.

2. **Grouping and Round 1 Discussion** – **Starts immediately after nominations close.** Duration for Round 1 discussion/voting: e.g., 1 hour (for live meetings) up to a few days (for asynchronous chat voting). In many cases, to maintain momentum, a shorter window (like a few hours) is beneficial if members are around. However, for global communities, 24 hours might be used to allow everyone a chance. We will allow this to be configured per deployment.

   * The transition from Nomination to Round 1 is immediate in implementation: as soon as nomination deadline hits, the bot does grouping and posts the groups. However, we consider the Round 1 “discussion period” to start at that announcement time.
   * The bot sets `votingDeadline` for Round 1 = now + votingPeriod (with votingPeriod from config, say 2 hours).
   * It announces the time by which votes should be cast (“until 20:00 UTC”).
   * Participants ideally should cast votes within that period. The bot’s heartbeat monitors this as well.

3. **Round 1 Voting End** – At `votingDeadline` for Round 1, the bot triggers the tally and results announcement. If all votes were received earlier, the bot *could* end early, but by default it will wait to the deadline. (We might implement an early finish check on heartbeat: if `all groups votesComplete and now > startTime + minVotingDuration`, then end early. But careful that we only do this if absolutely all votes are in or all but one and that one’s presence maybe indicates they abstained. For this spec, we keep it simple: end at deadline, unless manually ended by admin.)

   * Once Round 1 is done, if a next round is needed, the bot immediately forms groups and sets a new `votingDeadline` for Round 2.
   * If the community wanted a break between rounds (say they want each round on a separate day or meeting), we could incorporate a pause. For now, assume immediate progression to keep cycles short.

4. **Round 2, 3, ...** – Each subsequent round follows the same pattern: announce groups, allow discussion/voting, wait for deadline or votes, then announce winners, proceed.

   * The number of rounds needed is roughly `ceil(log_groupSize (number_of_participants))`. Example: 125 participants with group size 5 -> Round1 yields \~25 winners, Round2 yields \~5 winners, Round3 final selection from those 5.
   * The config can include `maxRounds` to avoid something weird, but in normal conditions it stops when finalists count <= groupSize (which likely happens when finalists count <=5 in our example).
   * We ensure not to schedule another round if it’s final. (The code checks condition and if not final, does next grouping).
   * Each round’s `votingPeriod` could be same as Round1 or possibly shorter if fewer people (we might keep consistent, or allow decreasing time as fewer voters – not necessary though).

5. **Final Selection Stage** – This is effectively instantaneous once you have finalists. In the timeline, as soon as the prior round ends and we identify finalists, we perform the random selection. There is no extended time window here (unless one wanted to schedule a public live drawing event at a specific time – we won’t in automated bot, we just do it).

   * The bot might introduce a tiny delay for dramatic effect (like wait a couple seconds or prompt something like “Selecting final delegate... 🎲”). But functionally, it’s immediate in that same call.
   * The final delegate announcement is made. The cycle is then marked complete.

6. **Delegate Tenure** – Now an ongoing timeline outside the sortition rounds:

   * If we define a term length (e.g., 3 months), we could store `termEnd = termStart + termDuration`. The bot could schedule or at least remind when term is nearing end.
   * The bot does **not** automatically remove the delegate at term end (unless we want to automate next election). Possibly better to just notify and let admins initiate the next cycle. But as future improvement, could auto-run /startsortition at term end.
   * The timeline for petition within the term is event-driven (when someone starts it).
   * We also implement the rule about petition threshold easing over time: one way is to linearly or stepwise reduce required signatures from a high percentage down to a lower one as `now` approaches `termEnd`. For example: in first 1/3 of term, require 66% support; in middle 1/3, require 50%; in last 1/3, require 33%. Or simpler: early threshold and late threshold with a midpoint switch. The exact scheme can be configured. The medium article suggests progressively easier, which we interpret as threshold lowers with time.
   * The bot can calculate threshold at petition initiation by checking how far into the term we are. Or we can just set one threshold and expect fewer people needed later because maybe some will have left – but better to implement the intended concept.
   * Because this is somewhat advanced logic, we will keep it straightforward: perhaps define two values: e.g., if petition starts within first half of term, threshold = 60%; if in second half, threshold = 40%. These values can be tweaked. If no term length defined (open-ended delegate), perhaps we then just set a moderately high threshold always, or base it on time since election (like after 2 months lower it).
   * Petition itself has a timeline (collection window). E.g., 7 days from initiation for signatures. The bot sets `petitionDeadline = now + petitionPeriod` on start. Heartbeat monitors it. If at the deadline threshold isn’t met, petition fails.
   * If threshold is met early, the bot can immediately conclude petition success (no need to wait further).
   * If threshold is huge and not likely to be met, petition just fails quietly at deadline.

7. **Next Cycle scheduling** – If auto scheduling is desired, we can incorporate a timer for next election. For example, set `nextElectionTime = termStart + termDuration`. Heartbeat could then announce “Term ending soon, next sortition will start in 1 week” and automatically call `startsortition` if desired. However, in this spec we will treat it as a manual step (so that communities can decide timing).

### Use of Heartbeat and Timer Implementation

We rely on the canister’s **heartbeat** function as a timer mechanism:

* The heartbeat runs roughly every couple of seconds (on IC mainnet it’s every consensus round or so). We will implement a check that runs maybe every few heartbeat invocations to avoid too-frequent checks if not needed (but even every 2 seconds is fine for just a couple of if statements).
* In heartbeat, we check:

  * If `cycle.status == Nomination` and `Time.now() >= cycle.nominationDeadline`: invoke the transition to grouping (Round 1 start).
  * If `cycle.status == VotingRound` and `Time.now() >= cycle.currentRound.deadline`: end that round (tally votes and move on).
  * We likely only handle one round at a time, so we don’t need to check multiple deadlines concurrently. Once round1 done, we create round2 with its deadline, etc.
  * If `cycle.status == Finalizing` (though finalizing is instantaneous, so we might not need a state; if final done, status becomes Completed).
  * If `petition.petitionActive` and `Time.now() >= petition.petitionDeadline`: finalize petition failed (if threshold not reached yet).
  * Possibly check term end: if we wanted auto election at term end, could check if now >= currentDelegate.termEnd (and no cycle active) then alert or auto-start next cycle (but we leave this as future).
* The heartbeat ensures timely execution even if no user is sending commands at that exact moment. For example, if nomination ended at 3 AM when no one is around to send a command, the heartbeat will still catch it and proceed, so by morning group formation is done.
* One caveat: heartbeats consume cycles continuously. We must ensure our heartbeat logic is efficient. Our checks are simple comparisons and maybe calling an internal function to change state and send a couple messages. That’s fine. We will ensure not to do heavy loops in heartbeat. Also, after a cycle completes and if no petition active, we can short-circuit the heartbeat (do nothing if idle).
* Additionally, OpenChat itself might have some limits: if the bot tries to send messages when no one is around, is that fine? It should be fine, messages just appear in chat history. There might be rate limits, but our events are infrequent.

### Handling Stage Transitions and Notifications

Each stage transition is accompanied by a bot notification (as described in the UX flow). To reiterate:

* **Nomination -> Round1:** Bot closes nominations (could send a last message like “Nominations closed with X participants.”), then immediately posts Round1 group assignments.
* **Between Rounds:** After each round results, it immediately (within same heartbeat tick or call) posts the next round’s groups (or final selection if that was last round). A tiny delay for readability is possible but not needed technically.
* **Final -> Delegate:** Bot posts final result and then possibly a summary or next steps.
* **Delegate -> Petition or Next Cycle:** Bot posts if delegate removed or at term end (if auto).
* **Petition -> Removal or fail:** Bot posts outcome accordingly.

We ensure that these transitions are atomic in our code – meaning we do all state updates and message sends as one transaction where possible. This avoids inconsistent states visible to users.

### Edge Timing Considerations

* **Extending Time:** If for some reason more time is needed (say a big debate and not enough have voted by deadline), an admin can manually extend. We don’t have a formal command, but an admin could potentially call a special function (if we expose it) or simply restart if it fails. Perhaps better: implement an admin-only command like `/extend 1h` applicable during a voting stage to extend that round’s deadline. This is an advanced feature, could be considered.
* **Early Completion:** If all groups finish early, the bot could expedite. But careful not to disadvantage someone who might use allotted time to deliberate. If all votes are in, probably okay to finalize. We might implement it.
* **Time zones and formats:** The bot announces times in UTC or a fixed reference. OpenChat doesn’t have per-user time zone, so UTC is safe.
* **Clock drift:** The IC provides a stable network time. Should be fine. We just rely on that.

In summary, the time-based flow is mostly handled by internal scheduling (via heartbeat) and clearly communicated deadlines in chat messages. The combination of automated checks and user commands ensures the process advances smoothly even if users are only intermittently online. This design prevents the cycle from stalling – it will progress at the configured times unless an admin intervenes (or something like not enough participants cause an abort).

## OpenChat Bot API Design and Interface

Interfacing with OpenChat requires that our canister adhere to the OpenChat bot API. Below, we outline the design of this interface and how the bot communicates with OpenChat and vice versa.

**Bot Registration in OpenChat:** Before the bot can operate, it must be registered with the OpenChat system. The OpenChat team provides a `/register_bot` command for users to register bot canisters. In our case, after deploying the canister, an admin will perform the following one-time setup:

1. In an OpenChat group (or via the UI), use `/register_bot`. This opens a form where the admin enters:

   * The **Principal ID** of our bot canister.
   * The **Bot Name** (a human-friendly name, e.g., "FractalBot").
   * The **Bot Endpoint**. For an on-chain bot, this might just be a placeholder or a special URL. (For local dev, it could be `http://localhost:4000` as in docs, but on mainnet, the OpenChat backend likely knows how to call a canister by principal through IC management. Possibly leaving endpoint blank or using a dummy like `https://ic0.app` might suffice. We will verify in testing.)
   * The **Description** of the bot and **Command prefixes** it will handle (list of commands like `/startsortition, /join, /vote, /petition, /status, /help`).
2. After registration, the bot becomes discoverable by name. The admin then **installs the bot into the desired group**. This is done by inviting the bot user to the group and granting it permissions. In OpenChat, there's an "Install bot into group" flow:

   * Search for the bot by name, select it.
   * Choose permissions (at least "Send text messages" so it can reply/announce).
   * Confirm installation.
3. Now the bot is an active participant in that group chat. OpenChat will route any recognized bot commands to the bot canister.

**OpenChat Calls to Bot (Incoming):** When a user in the group sends a message starting with one of the bot's registered commands, OpenChat's backend identifies which bot it’s meant for and calls the bot’s canister. The exact interface isn't publicly documented in detail, but based on patterns and SDK usage, it's likely one of:

* A generic function like `update func process(botCommand: Text, args: Text, context: MessageContext) -> async Response`.
* Or distinct methods for each command as listed in registration (less likely, because dynamic commands would make more sense with one entry).
* The `openchat-bot-sdk` for Motoko probably abstract this. Possibly it provides a base actor class with a message handler we override.

For concreteness, let's assume the interface is something akin to:

```motoko
type MessageContext = {
   chatId: Nat;
   userId: Principal;
   userName: Text;
   messageId: Nat;
};
type CommandRequest = {
   command: Text;  // e.g. "join", "vote", "startsortition" (without slash)
   arguments: Text; // everything after the command, e.g. a username or parameters
   context: MessageContext;
};
type CommandResponse = {
   messages: [Text]; // list of messages for the bot to send as reply (OpenChat might post them)
   // maybe some flag if ephemeral, etc.
};
public shared func handleCommand(req: CommandRequest) : async CommandResponse;
```

This is hypothetical, but something along these lines.

In our implementation:

* We will parse `req.command` and decide which logic to invoke.
* Use `req.context.userId` to identify the sender for permission checks and record-keeping.
* Use `req.arguments` for things like who was voted for.

Alternatively, the SDK might route to different functions, e.g., there might be an annotation like `//@oc bot command "join"` above a function to tie it to that command. If so, then the canister could have separate endpoints:

```motoko
public func oc_join(context: MessageContext) -> async Text;
public func oc_vote(context: MessageContext, target: Text) -> async Text;
```

But likely they keep it generic.

**Bot Responses to OpenChat:** After processing, our bot often needs to send one or more messages to the chat. There are a couple of ways:

* **Return Value as Message:** If the interface allows returning a string or list of strings, OpenChat might directly post those as bot messages. For quick one-off replies (like confirmation of join or help text), this is convenient.
* **Calling Send API:** For more complex interactions (especially multiple messages or delayed messages), the bot can call OpenChat’s send method. In Rust or off-chain, you do an HTTP call. In our on-chain scenario, possibly the OpenChat system provides a canister method we can call (maybe on some OpenChat canister providing messaging capability). However, likely how it works is:

  * If the bot canister returns a response, OpenChat posts it.
  * If the bot canister needs to post something not directly tied to a command (like scheduled announcements via heartbeat), how do we do that? We might need to call OpenChat’s message API. Since our bot is a canister, one approach is to perform an inter-canister call to OpenChat’s group canister. However, the group canister might require the sender to be the bot's principal and have permission. Possibly OpenChat has a universal canister method like `send(chatId, text)` which was shown in the forum snippet. If that snippet is anything to go by, maybe the bot canister itself implements `send(chatId, message)` and OpenChat calls *that* via an agent on behalf of the user controlling the bot (like the code example did). Actually, that snippet was from a user perspective calling their own bot canister to have it send a message. Wait, let's clarify:

    * The snippet used `bot.send({chatId, message})` and expected a response. Possibly in that design, the bot canister’s interface included a `send` method that then internally calls OpenChat. But more likely, the `send` in snippet corresponds to an OpenChat API endpoint. The code might have been calling OpenChat via an `Actor` interface.
    * Actually, the snippet’s idlFactory defines a service with `send(chatId, message) -> text`. And they create an actor with the bot’s canister ID. That suggests *the bot canister* had a method `send`. It returned a Text (maybe an acknowledgment). That code calls `bot.send(...)`. If our canister had such a method, who calls it? Possibly the OpenChat system calls it to forward user messages or to allow sending?
    * Alternatively, maybe they mistakenly named it `send` but it actually was OpenChat's service. However, context says "Here’s the simplified Candid interface — just a send function for messages" – it might be the OpenChat's interface itself to send messages. Actually, reading closely:
      "Yo, replace with your actual OpenChat Bot Canister ID. ... Here's a simplified Candid interface — just a send function for messages."
      Possibly what they did is: they wrote an IDL for the bot that has a `send` function (so they can call their bot canister externally). The bot canister likely on receiving that call, sends a message into the chat and returns maybe "ok" text. So this suggests an off-chain script controlling the bot by calling its send method. This is a bit confusing, but maybe an alternative way to use off-chain control.

Anyway, to not overcomplicate: The simplest design for on-chain operation:

* The bot returns immediate responses for direct commands as needed (since the user expects an immediate answer usually).
* For scheduled events (like announcing round results via heartbeat), the bot can call an OpenChat message endpoint. Perhaps the OpenChat team has an agreed method name for bot to call back. The OpenChat architecture might allow a bot canister to call a specific canister (maybe the group canister or a message hub) to post messages. If not documented, we might rely on a workaround: The bot can "simulate" a user command to itself? (not likely).
* Considering the complexity, we might adopt a simpler approach: Plan to have any announcement triggered by some user or admin action. But announcements like group formation ideally are automatic.
* If the OpenChat SDK for Motoko is well-designed, it probably has a function like `Bot.send(chatId, text)` we can call in our code which internally does the correct call. We'll assume that and use it. If not, we can design a custom approach:

  * Possibly use the ManagementCanister (IC method to call another canister) if we know the group canister and method to deliver a message. But that is deep in OpenChat internals. Let's trust the SDK provides something.

So, our canister will have the following relevant interface to OpenChat:

* Implements whatever trait to handle incoming commands (like `oc_bot_message(...)`).
* Uses OpenChat SDK functions to output messages.

**API Example in Use:**

* *User triggers command:* e.g., user sends "/join". OpenChat calls our canister (maybe `handleCommand`).
* *Bot logic runs:* adds user to participants.
* *Bot responds:* It returns a text like "X joined" which OpenChat prints in chat *OR* calls `Bot.send(chatId, "...")` to post that message explicitly.
* *Scheduled event:* e.g., heartbeat decides groups are ready. Now without a user command, the bot itself needs to send a message. In code, we will do something like:

  ```motoko
  Bot.sendMessage(cycle.chatId, formattedGroupListMessage);
  ```

  where `Bot.sendMessage` comes from the SDK. This will result in a message in chat. There's no direct user trigger, but since our canister (with its principal) is an authorized participant in the group, it should be allowed to post. The bot’s principal likely acts like a user that can send messages.

**Error Handling and Idempotency:**

* If OpenChat calls our bot and for some reason the call fails (bug or state issue), OpenChat might log or ignore. We should minimize failures by thorough state checks.
* If the same command comes twice due to a retry, our code should handle gracefully (e.g., if two identical `/startsortition` come due to someone double tapping, the second should be ignored because cycle is already active).
* For voting, if duplicates, handle as one or last one counts depending on design (we choose first vote counts and ignore later).
* The interface likely ensures one call per message, but just to mention.

**Security in API:**

* We will verify the caller of incoming commands is an OpenChat canister. Possibly the SDK does this. If not, we can include a simple check using `msg.caller` (OpenChat might pass the user principal in args, and the caller is OpenChat's backend principal).
* That prevents external malicious calls forging commands.

**Testing the API:**

* In a local environment, one can deploy OpenChat’s docker and test the bot by registering it (the Qiita article shows how to do this with a local instance).
* We can also simulate a call by directly calling the canister’s command method with a crafted payload to ensure it returns expected text.

**Limits:**

* The bot will not spontaneously read non-command messages. So if users just discuss without commands, the bot stays quiet (which is fine).
* The bot only acts in the group it's installed in (the context includes chatId). If installed in multiple groups (possible scenario), we need to differentiate by chatId. Our state currently assumes one cycle at a time possibly for one community. If the bot were invited to two separate groups and someone started a cycle in each, that complicates state. Perhaps we restrict to one group usage (maybe only register it to one group). We can note as a limitation that this bot is intended for a single community. Or we would need to index state by chatId and handle separate processes concurrently. That is an expansion to consider in future (multi-community support).
* Rate of messages: If a lot of commands come at once, the bot processes sequentially. It's fast enough for human-scale usage. Should a user spam commands, the bot might respond but perhaps we could implement a rate limit per user to avoid spam (not critical now).

In conclusion, the bot’s API design leverages the OpenChat provided mechanism for bot messaging. The developer does not have to implement low-level networking; instead, they focus on the command handling logic. The **OpenChat Bot SDK for Motoko** is our friend here, abstracting away much of the boilerplate so we can focus on governance logic. We just ensure our canister meets the interface and that we properly register the commands with OpenChat so everything connects.

## Petition and No-Confidence Mechanics

The petition and vote of no-confidence system is a vital part of ensuring ongoing accountability. This section details the mechanics in technical terms and how the bot enforces the rules:

### Initiating a Petition

* A petition can only be started if there is currently a delegate in office. The bot checks `currentDelegate != null` when someone issues `/petition`. If no delegate is present (i.e., between cycles or position vacant), the bot will respond with an error like “No delegate to petition against at this time.”
* Only one petition can be active at a time. If someone attempts to start another while one is ongoing, the bot will say “A petition is already in progress. Please wait until it concludes.”
* If conditions are met, the first user’s `/petition` call triggers the creation of a **PetitionState**:

  * `petitionActive = true`
  * `petitionStartTime = now`
  * `petitionDeadline = now + petitionPeriod` (e.g., now + 7 days)
  * `signers = { <firstUserPrincipal> }`
  * The required threshold is computed as follows:

    * If configured as an absolute number (e.g., 10 signatures) or a fraction of group members, compute it. Possibly `requiredSigners = max(minThreshold, ceil(percentage * memberCount))`. We might have a config like `petitionThresholdPercentEarly` and `...Late`.
    * Determine if “early” or “late” threshold applies: Check delegate’s term progression. For example, if `now - currentDelegate.termStart < termDuration/2` use early threshold (higher), else late threshold (lower). If no term duration known, default to early threshold unless delegate has been long in office (maybe a fixed period like if > 3 months, allow late).
    * For instance, if group has 50 members and early threshold is 60%, requiredSigners = ceil(0.6 \* 50) = 30. Or if using absolute and it’s configured as 10, then 10.
    * The chosen threshold is stored in `petition.requiredSigners`.
  * The bot then broadcasts the petition initiation message (as described earlier), including how many are needed and by when.
* It’s important to note who counts as “members” for threshold calculation. It could be:

  * All members of the OpenChat group (if group membership is the community).
  * Or all participants of last cycle? But likely entire community group. We assume group membership count is the electorate.
  * We might retrieve the group member count if OpenChat provides it (maybe not easily accessible to bot). If not, maybe we treat the number of participants in last election as proxy. But better to use group size. In absence of an API, we could have an admin configure the threshold number directly for now.

### Signing the Petition

* When other users send `/petition` during an active petition:

  * The bot checks petitionActive. If false, it might interpret as start new (but that case is handled above).
  * If true, it checks if the user is already in `signers`. If yes, respond “You have already signed.” (Possibly as a DM or no response to avoid cluttering group, or maybe a small group message).
  * If not signed:

    * Add user’s principal to `signers`.
    * Count = size of `signers` set.
    * If count < requiredSigners, update still collecting. The bot might post an update: “Petition update: X/Y signatures so far.” (We should throttle these updates; possibly post every single one might be okay if threshold is small, but if threshold is like 100, posting each could flood. Maybe we post every 5th or 10th signature, plus always when about to succeed.)
    * If count == requiredSigners (threshold reached exactly):

      * Immediately trigger petition success (no need to wait further or for deadline).
      * That will lead to removal process (detailed next).
    * If count > required (just in case overshoot by multiple signers at same time), also treat as success (the extra signers don’t matter beyond threshold).
* The petition remains open until deadline or success.

### Petition Success – Removing Delegate

* When threshold is met, either via signatures or via optional subsequent vote:

  * **Direct Removal Approach:** In this design, we will consider that hitting the signature threshold itself means a successful vote of no confidence. Essentially, we treat the petition as a direct democratic vote (if majority signed, it's as good as majority vote result).

    * The bot sets `petitionActive = false` (closing it).
    * Announces success: e.g., “The community has gathered sufficient support to remove Delegate Alice. The vote of no confidence is successful.”
    * Performs delegate removal:

      * It clears `currentDelegate` (or marks them as removed, maybe record it somewhere for history).
      * It then checks `reserveCandidates`. If not empty:

        * Choose one replacement at random (or if we want a deterministic approach, we could pick the first or by some ranking, but randomness is more aligned with fairness). We'll do random selection from reserves.
        * Let’s say we randomly pick Raj from reserve. Remove Raj from `reserveCandidates`.
        * Set `currentDelegate = Raj`.
        * Announce: “**Raj** (one of the finalists from last election) will take over as the new delegate for the remainder of the term.”
        * Update Raj’s DelegateInfo (term start now, term end same as original Alice’s end or maybe we reset term? Could allow them to have a full term or just finish out – likely finish out, to keep election schedule consistent).
        * Now the community has a delegate again without a full cycle. This is quick and no downtime in leadership.
      * If `reserveCandidates` is empty (or maybe the community doesn’t want auto replacement):

        * The bot announces vacancy: “The delegate has been removed and no vetted replacement is available. The position is now vacant. A new sortition cycle should be started to elect a new delegate.”
        * `currentDelegate` becomes null (vacant).
        * It’s then up to admins to call `/startsortition` at their discretion. (We could optionally auto-start a new cycle immediately, but that might be chaotic; better to let humans decide timing.)
    * All petition state is reset (we can clear the signers set, etc. to be ready for a future petition for the new delegate if needed).
    * Also implement a cool-down such that after a successful removal, the next petition cannot be started too soon to give the new delegate some time (maybe no petition allowed for first X days of new delegate's term or as config).
  * **Alternate Approach – Two-step (Petition then Vote):** For completeness, some systems might require that after a petition threshold is reached, a formal yes/no vote by the whole community is triggered, to confirm removal (this is like petition triggers a referendum). If we wanted that:

    * The bot would announce “Threshold reached, initiating official vote of no confidence. Please vote `/vote yes` or `/vote no`.” (We’d need separate handling of this vote, maybe treat it as a special round where all group members get to vote).
    * That might last a shorter time and then if majority of votes say yes, remove delegate.
    * This is more complex and beyond current spec, so we won’t implement it now. We assume threshold itself is direct removal (since threshold likely was majority anyway).
* The removal and replacement should also consider notifications outside chat if needed (e.g., maybe update a DAO on-chain registry – out of scope here). At least, in chat everyone is informed.

### Petition Failure

* If the petitionDeadline arrives and `signersCount < requiredSigners`:

  * Bot marks petition as failed and inactive.
  * Announces: “The petition to remove Alice failed to reach enough support by the deadline. Alice remains as delegate.”
  * Optionally state the number it got (“Only 8/20 required signatures were collected.”) to show it wasn’t that close.
  * Enforce a cool-down: disallow starting a new petition for some period (maybe a few weeks or until a condition). This is to avoid spam petitions. The bot can store `lastPetitionTime` and if new petition comes too soon after fail, respond “A recent petition was already held and failed. Please wait before starting another.”
  * The delegate continues as normal. The reserve list is unchanged (no effect because delegate didn’t change).

### Constraints and Edge Cases in Petition

* **Edge: Delegate Steps Down Voluntarily:** If a delegate resigns or is removed manually by admins (outside the bot’s scope), the bot should be informed or updated (maybe admin could use a command to force remove or mark vacant). If that happened, any petition should be auto-canceled. We might provide an admin command if needed, but skip for now.
* **Edge: Delegate removed and no one left in reserve and no new election scheduled:** The community might be without delegate until next election. The bot just reflects that state.
* **Edge: Multiple petitions spam:** The bot prevents that by one at a time and cooldown.
* **Edge: Petitioner threshold calc issues:** If group membership changes drastically during petition (people leave or join the chat), our threshold is static once set. That’s acceptable, though if membership dropped, threshold might be higher than 50% of current, making it impossible. But that scenario is rare and we accept it. Next petition can recalibrate.
* **Edge: Replace from reserve multiple times:** Imagine delegates keep getting removed one by one. If the reserve had several finalists, we could theoretically handle multiple replacements until reserve is exhausted. E.g., Alice removed -> Raj takes over, leaving maybe Maya still in reserve. If Raj then removed later, pick Maya. If we run out, need an election. We should consider clearing reserve when a new full cycle is run, of course.
* **Petition by delegate themself or weird initiator:** If the current delegate themself types `/petition` (basically "petition to remove me"), the bot should allow it (maybe they want to test confidence or step down). It counts as a signature like any other.
* **Integration with Off-Chain or Other Systems:** For now, the petition is entirely done within OpenChat group by the bot. If the DAO had an on-chain governance system, ideally the removal should connect to that (like revoke a role). Out of scope but a possible extension is to have the bot call an external canister or trigger a proposal on SNS. We do not cover that in this spec.

All told, the petition and no-confidence mechanism is implemented to mirror the "democracy deselects" concept. It’s a safeguard ensuring that random selection doesn't put someone unfit in power for too long – the community has a safety valve. The bot makes this process systematic and fair (clear thresholds, time windows, and automated execution of the outcome).

## Edge Case Handling and Constraints

No system is complete without considering edge cases and constraints. Here we list various scenarios and how the Fractal Sortition Bot will handle them to maintain robustness:

### Insufficient Participation

* **Too few candidates:** If only one person (or none) joins during the Nomination stage:

  * If one person joins, technically they could be declared the delegate by default (as there’s no competition). The bot might handle this as a special case: when closing nominations, if participant count = 1, skip directly to final selection and declare that person delegate (perhaps after confirming they accept). This avoids unnecessary grouping (group of 1).
  * If zero people join by deadline, the bot will announce “No participants joined. The sortition cycle is aborted.” It then ends the cycle with no result. Possibly suggest “We’ll try again later” or allow admin to extend nominations. Admin could also cancel or restart. The bot should then reset state for a new attempt.
  * We can implement a threshold like `minParticipants` (e.g., 3). If fewer than that, auto-extend nomination or abort. But simplest is above logic.

* **Odd group sizes:** If the number of participants doesn’t divide evenly into group size:

  * The bot will distribute as evenly as possible. For example, 13 participants with group size 5 yields groups of 5,5,3. A 3-person group is smaller – we allow it. It might be easier for someone in that group to win because fewer voters, but that’s just how randomness falls. It’s still fair random grouping.
  * If a group of 1 were to happen (e.g., 6 participants with group size 5 -> groups of 5 and 1), that’s not ideal. We could avoid it by adjusting group size or merging:

    * One solution: if last group would have < 3, consider redistributing: e.g., 6 with group of 5: better do 3 and 3 rather than 5 and 1.
    * In general, if the remainder is 1, maybe reduce group size by 1 to balance.
    * We can implement: if (N mod groupSize) < 2 and N > groupSize, then use smaller groups instead. Or specifically, if last group is size 1, just move one person from second last group to make two groups of 3 and 4 instead of 5 and 1.
    * This is refinement but nice to handle. The bot can detect that and adjust grouping logic for fairness.
  * If a group of 1 still somehow occurs (like if only 1 person in last round), then that person automatically is winner of their group (with no vote needed). The bot should handle it gracefully, not wait for a vote. Essentially if group size = 1, that group’s winner = that person by default.

* **Tie votes in group:** Already discussed; the bot will break ties randomly. This ensures the round can conclude. Alternatively, we could have those tied members do a quick runoff, but in chat it’s simpler to randomize.

  * The bot should log that event: possibly sending a message like “Group 2 had a tie between @X and @Y with 2 votes each, randomly selected @X as winner.” So it’s transparent.

* **Non-voting participants:** If one or more participants in a group never cast a vote (maybe they went offline):

  * When deadline comes, those votes are missing. We tally whatever votes were cast. It’s possible the remaining votes could cause a tie or even no votes at all in a group.
  * If no one in a group voted, perhaps those group members didn’t engage. The bot could:

    * Randomly pick one member from that group as winner (since no input, default to random selection among them – still fair in a sense).
    * Or consider the group invalid and exclude them entirely (meaning none from that group advance, losing those candidates – but that’s harsh).
    * Better to randomly pick, to keep someone from that group in the running. It aligns with sortition fallback.
    * We should announce, e.g., “Group 3: No votes cast, so a representative was chosen at random -> @Z”.
  * If some voted but not all, we just count those votes. If that yields a tie or only one vote or something, we treat normally (tie break if needed, or single vote wins).
  * Essentially, absence = abstain, which doesn't stop process.

* **User leaving the group or chat:** If a participant leaves the OpenChat group mid-process:

  * The bot might not be directly notified of that (OpenChat bots might not get membership change events, unless we query).
  * If we become aware (maybe via an admin command to remove them), the bot should remove them from participants list.
  * If they were still candidate, and leave, we consider them withdrawn. If that happened after grouping, it might leave an empty slot:

    * If a group lost a member, fine – smaller group, continue. If they were up for vote, others just don’t vote for them. If they were the only one left, then by leaving that group effectively yields no winner – maybe treat as no votes scenario and random pick from those remain (if any remain in group; if group becomes empty due to someone leaving and it had no one else, that means originally group of 1 left. Then no representative from that group. If other groups exist, maybe proceed without that group).
    * That scenario is complex but improbable unless someone rage quits in protest. We'll handle gracefully: we won't crash, worst-case that group’s contribution is null. If needed, we can drop that group and reduce number of finalists accordingly.
  * If the delegate (winner) leaves the group or deletes account, that’s equivalent to stepping down. The bot should detect if the current delegate is no longer reachable:

    * If possible to detect (maybe the bot tries to DM them or sees group membership changed?), not straightforward. We rely on the community to notice and an admin to remove them. Then treat like a removal: either pick reserve or start new.
    * Possibly an admin command `/delegate_resign` could be introduced to handle it explicitly.

### Command Misuse and Validation

* **Unauthorized use of admin commands:** If a normal user tries `/startsortition` or `/cancel`, the bot should reject with a polite “You don’t have permission to do that.” The bot will maintain an `adminList` (could be initial deployer, or configured via a principal, or simply the group owner’s principal if that’s gettable). We assume we know who the admin is (maybe the person who registered the bot).
* **Out-of-order commands:** If someone tries `/join` when no cycle is running or after nominations closed, bot says “No active nomination period right now.” If someone tries `/vote` when not in a voting stage, or when they’re not a participant, we handle similarly.
* **Typo in command or wrong usage:** If user types `/vote` without specifying a target, we can prompt “Usage: /vote @username”. If they specify an unknown name, “User not recognized or not in your group.” (We may need to parse the argument and match to a participant. Ideally they mention like @Name which OpenChat could give us principal or we might have to match by name in our participant list).
* **Case sensitivity:** Likely OpenChat commands are case-insensitive after slash. We'll handle command strings in lowercase to be safe.
* **Multiple parallel cycles attempt:** If an admin erroneously starts another cycle when one is running, bot will say no. If two admins try at nearly same time, one will win, second gets rejection because state now says active.
* **Group installed in multiple chats scenario:** If the bot is installed in multiple groups (which is possible if someone registers it globally and adds to two communities):

  * Our current state model doesn’t separate by chatId – which could cause data mix-ups (someone in group A triggers something that affects group B).
  * For now, we assume usage in one group only. We might set a flag on registration that it's bound to one context. Alternatively, we can incorporate chatId into our state and key cycles by chat. That would be more complex (multiple concurrent cycles in different communities, one canister).
  * Given scope, we'll clearly document that the bot is intended for a single community at a time. If needed, one could deploy separate instances for separate communities.
* **OpenChat downtime or message failures:** If OpenChat doesn’t deliver our message or goes down mid-cycle, the bot might still progress internally. When OpenChat comes back, perhaps our state moved on without users seeing announcements. This is hard to handle but rare. We might implement redundancy: e.g., store last announcement that should have been made, and on the next user interaction or even heartbeat, try sending again if it looks like it wasn’t confirmed. Possibly not needed.

### System Constraints

* **Cycle Limits:** If a huge community (say 1000 members) all join, group fractal still works but maybe require more rounds. Group of 5 with 1000 initial -> 200 winners -> 40 winners -> 8 winners -> final shortlist of 8 (since next round would yield \~1-2). Actually 8 still >5, maybe one more round yields 2 or so, then final. So could be 4-5 rounds. The bot can handle that in theory. We should ensure performance:

  * Shuffling 1000 entries is fine.
  * A bit heavy maybe posting group lists (200/5 = 40 groups with names, that's a long message, might need splitting across multiple messages as OpenChat might have message length limits). If too many groups, maybe the bot posts group info in multiple messages or a file – but file not ideal. We can paginate group announcements if needed (like post 10 groups per message).
  * Or post an external link to a group list. But probably manageable up to few hundred.
  * Memory for 1000 participants is fine.
  * So the main limit is readability. For communities bigger than a few hundred, fractal sortition might need alternate approach (like splitting into sub-chats), but outside our scope.

* **Cycle Frequency and Cycle Overlap:**

  * Should not start a new cycle until previous one is completely finished (including any petition? Petition can overlap with term, which is after cycle). But multiple cycles not allowed concurrently by design.
  * Frequency could be set (like maybe by default one cycle every quarter). The bot won't automatically throttle except by admin usage, but we can mention not to run them too often since it can be heavy on engagement.

* **Cycle Cancellation or Reset:** We have `/cancel` for emergencies. Use case: something went wrong (e.g., a bug or mis-grouping).

  * On cancel, the bot should try to gracefully inform participants that cycle ended prematurely. It should clear relevant state. Possibly it could log if needed (not mandatory).
  * Cancel permissions restricted to admin.

* **Storage and Upgrade Constraints:** We must ensure our stable state doesn’t exceed limits. It won’t in normal usage. But if we log every cycle, logs might accumulate. Perhaps limit storing of historic data, or allow manual trimming.

* **Privacy and Security:** All data (like votes) are technically stored in the canister and are not end-to-end encrypted – they are somewhat open to inspection by someone who can query canister state (though if we don’t expose it via interface, only canister controller could see raw data via dfx tools). Votes in-group are not anonymous since if someone sees who typed /vote, they know at least that person voted for someone (the content of vote might be guessable if they mention the target).

  * If anonymity was desired, this approach would need change (like secret ballot not easily done in open chat without additional crypto). For now, it's open voting to group.
  * But we can note that as a trade-off: in small group discussions, peer pressure might influence. However, that’s similar to fractal meeting where you publicly choose rep, so it's fine.

By preemptively handling these edge cases, the bot will be more resilient and provide a smoother experience. We’ll document any limitations (like single-group assumption) clearly to users of the bot in the README.

## Deployment and Setup Instructions

This section provides a step-by-step guide for developers on how to deploy the Fractal Sortition Bot canister and integrate it with OpenChat. It also covers initial configuration and any setup nuances.

### Prerequisites

* **DFX SDK**: Ensure you have the DFINITY SDK (dfx) installed, with a version compatible with Internet Computer mainnet (e.g., dfx 0.25.x or later).
* **Motoko Compiler**: dfx includes Motoko. No separate install needed beyond dfx.
* **OpenChat Local Environment (Optional)**: If you want to test locally, you can run OpenChat in a local replica via Docker as described in OpenChat docs. This requires Docker.
* **Cycles**: You will need cycles to create and sustain the canister on IC mainnet. Make sure you have an Identity and a wallet canister funded with cycles.
* **OpenChat Account**: You should have an OpenChat user account (on mainnet or local) with privileges to create groups and register bots (any user can register a bot, but you might want a fresh test account to avoid spamming real communities).

### Building and Deploying the Canister

1. **Obtain the Source Code**: Clone the repository from GitHub (the project repository where this spec resides). Ensure you are on the latest stable branch for the bot.
2. **Inspect/Modify Configuration**: Open the source file (likely `src/FractalBot.mo` or similar). Check the `Config` values (group size, durations, thresholds). Adjust if needed for your community. Defaults are provided in code, but you can tweak before deploying. You can also modify the admin principal list if needed (to your principal so you have control).
3. **Install OpenChat SDK Package**: This project depends on the `openchat-bot-sdk` Motoko package. If using Vessel or Mops as package manager, ensure the package is referenced. For example, using Mops, run:

   ```bash
   mops install openchat-bot-sdk
   ```

   This will fetch the SDK. (If the project repository already includes it or a package file, just verify it’s present).
4. **Build**: Use dfx to build the canister:

   ```bash
   dfx build fractal_sortition_bot
   ```

   or if not in a dfx project, integrate it into one. Possibly the repository includes a `dfx.json`. If not, you might create one or use `dfx canister create` etc. Usually, you would have a dfx project with the bot as one canister.
   Building should compile the Motoko code into a WASM. Ensure no compile errors. (If the OpenChat SDK is missing or version incompatible, resolve that via Mops or Vessel).
5. **Deploy to Local (optional)**: If testing locally, start a local replica with `dfx start` or use the OpenChat Docker which includes a replica. Deploy the canister locally:

   ```bash
   dfx canister install fractal_sortition_bot --mode=reinstall
   ```

   This gives you a local canister ID (e.g., `r7inp-...-glsac`).
   Then you’d integrate it with the local OpenChat per Qiita steps (using /register\_bot in local OpenChat UI and pointing to `127.0.0.1:8080` etc).
6. **Deploy to ICP Mainnet**: To deploy on the IC, use:

   ```bash
   dfx deploy fractal_sortition_bot --network ic
   ```

   Make sure your dfx identity has cycles and is controller of the canister (dfx by default will create a new canister on IC for you and you’ll be controller). Alternatively, you might want to deploy via `dfx canister create` and then `dfx canister install`.
   After deployment, note the **Canister ID**. This is crucial for registration on OpenChat.
   Also note the canister’s principal (which might be same as canister ID in text form, but if needed).
   Ensure the canister has enough cycles. A governance bot shouldn’t consume too many cycles, but if it uses heartbeat, that will slowly eat cycles. Starting with, say, 1 billion cycles is recommended (which is 1T cycles costing 0.0001 ICP, negligible). Monitor cycles usage over time.

### Registering the Bot on OpenChat

Once the bot canister is live, follow these steps to connect it to OpenChat:

1. **Create a Group**: On OpenChat (using the front-end at oc.app or local), create the community group chat where you want this governance process. If one already exists, ensure you have admin rights in it.

2. **Register Bot**: In OpenChat’s interface, in any chat (could be a private chat or the target group), type the command:

   ```
   /register_bot
   ```

   This will bring up the "Register a bot" form (in the OpenChat UI). Fill in the details:

   * *Principal*: Enter the **principal (canister ID)** of your deployed bot. (e.g., `abcde-biaaa-...-daic`). Make sure no typos; if the principal is wrong the bot won’t be reachable.
   * *Bot Name*: Choose a name (e.g., "FractalBot" or "SortitionBot"). It must be unique on the OpenChat network.
   * *Bot Endpoint*: For mainnet canister, put the base URL of the IC API that OpenChat can use. Typically this might be `https://icp0.io` or `https://ic0.app`. (OpenChat likely uses an agent to call, not an HTTP call, so this might not matter or you can use a special format. In local environment, they used `http://localhost:4000`. On mainnet, they might instruct to use `https://ic0.app` which is a boundary node address. Check OpenChat docs for any specifics. If unsure, using `https://ic0.app` should be fine as it’s a valid domain for IC. The principal ensures correct canister.)
   * *Test Context*: Select a context chat to test (not sure if required; Qiita shows selecting the group created as "Test context". Possibly they require you to pick a group that will be used initially.)
   * *Description*: Write a short description, e.g., "Bot that conducts fractal sortition governance rounds".
   * *Commands*: List out the commands the bot should respond to, separated by spaces or lines. For example:

     ```
     /startsortition
     /join
     /vote
     /petition
     /status
     /help
     ```

     Only include commands you implemented. If we included `/cancel` as admin-only, you can list it too, though it might be fine not to advertise it.
   * Submit the registration. If all goes well, OpenChat registers it in their system.

3. **Install Bot into Group**: Now you need to add the bot to your group:

   * In OpenChat UI, go to the group chat you created (or target).
   * Click the "Direct chats + button" or maybe in group settings "Add bots" (depending on UI).
   * Search for the Bot Name you just registered (e.g., "FractalBot").
   * It should appear; select it.
   * It will prompt to **Install bot into group** and ask for permissions. The crucial permission is "Send Messages" so the bot can send texts. There might be others (like receive media; not needed here).
   * Confirm to install. You should now see the bot listed as a member in the group (with a special bot icon perhaps).
   * If you had a direct chat with the bot, you can also test commands in that DM to ensure it responds (some bots allow that; if we coded accordingly, e.g., help might work in DM too).

4. **Testing**: In the group, try a simple command like `/help` to see if the bot responds with the command list. If it does, the integration is successful. If not, check:

   * That the commands were correctly listed and match what you type.
   * That the bot canister has cycles and is running (you can try a dfx ping or check status in dashboard).
   * Check the OpenChat dev forum for any notes if Motoko bot needs a particular agent or if any error is shown in UI.

5. **Running a Cycle**: Once verified, you can initiate a cycle with `/startsortition` in the group (assuming you as admin have permission in bot, which if not you may have to configure admin principal in code to your principal). The bot should respond and you can go through a full cycle in a test scenario.

### Deployment Notes

* The bot canister is an independent canister – OpenChat doesn’t host it, so **you are responsible for maintaining it**. That means:

  * **Monitoring cycles**: Keep an eye on the canister’s cycle balance. Heartbeat and usage will slowly consume cycles. Top up as needed using ICP or cycle faucets.
  * **Upgrading the canister**: If you update the code (bug fix or feature), you can deploy a new WASM to the same canister via `dfx canister install --mode upgrade`. Because we use stable vars, the state (like current delegate or even an ongoing cycle) should persist. However, upgrading mid-cycle is risky (could cause slight disruptions in timing or lost in-progress ephemeral data). Prefer upgrading between cycles. Always test upgrade in local before doing on production.
  * **Logging**: We haven’t built extensive logging. We might add print statements in Motoko for debugging. These appear in the canister’s output (in local dfx or possibly available via candid interface). On mainnet, you don’t have stdout, so consider adding an admin command like `/debug_state` that dumps some state to you, if needed for debugging any issues.
* **Security**: The canister’s controller (likely your dfx principal or wallet) can upgrade or stop it. Be cautious: if someone else gets controller, they could alter bot behavior or compromise governance. Use a secure wallet and principal.
* **OpenChat updates**: The OpenChat platform may evolve. Keep an eye on OpenChat developer updates (DFINITY forum or OpenChat GitHub). They might release a more official approach for Motoko bots or additional features. For example, if OpenChat introduces an API to fetch group members or a new permission for reading messages beyond commands, etc. We might update the bot to utilize that in future (like to automatically count group members for petition threshold).
* **Multiple Communities**: As noted, one deployment is intended for one group. If you needed the bot for two independent communities, it’s recommended to deploy two separate instances (with different principal IDs and names) to avoid mingling state. Otherwise, one could enhance the code to handle multiple chats by mapping chatId to cycle state – but this adds complexity (not covered here).
* **Cleanup**: If you ever want to remove the bot:

  * In OpenChat, uninstall the bot from the group (so it no longer responds).
  * Deregister if needed (not sure if possible, but at least remove from group).
  * On canister, you can either keep it idle or delete it (by uninstall code or freeing cycles).
  * There's no direct cost to leaving it idle aside from cycles slowly draining from heartbeat. You can disable heartbeat by upgrading with it turned off or just decommission the canister.

### Example Quickstart (Summary)

For a quick reference, here’s a high-level quickstart:

* Deploy canister to IC.
* `/register_bot` in OpenChat with canister principal, name, endpoint, commands.
* Install bot into your OpenChat group, grant send permission.
* Ensure you (admin) are set in bot’s allowed list for admin commands.
* Test `/help`.
* Start a dummy cycle with a couple friends to see it run through.
* Monitor any console or UI outputs for errors.
* Congratulations, your decentralized sortition is up and running!

By following these steps, developers and community admins should be able to get the Fractal Sortition Bot operational. The bot essentially becomes part of the community’s chat, orchestrating governance transparently in front of all members.

## Future Considerations

As a prototype, this bot covers the essential functionality for fractal sortition governance. However, there are many opportunities for improvement and extension. Future developers and contributors might consider the following:

* **Multi-Community/Group Support:** Enhance the bot to handle multiple groups concurrently. This would require indexing state by `chatId` and possibly running multiple sortition cycles in parallel (one per community). It complicates the design (especially if cycles overlap in time) but would make the bot a shared service. Alternatively, scaling by deploying multiple instances might suffice for now.
* **Multiple Roles or Positions:** Currently the bot selects one delegate at a time. A DAO might have a council of say 3 delegates chosen via sortition, or multiple roles (like Treasurer, Secretary etc). We could extend final selection to choose N random finalists instead of one, if multiple positions are needed. We’d need to adjust how votes are done (maybe each group picks top N? Or run N separate cycles?). Another idea is to run one cycle per role sequentially. The architecture could be expanded to parameterize the number of winners needed.
* **Integration with On-chain DAO/SNS:** To truly effect change, the bot’s outcome could be tied to on-chain governance. For example, if the community controls an SNS (Service Nervous System) or some token-based DAO, the bot could, after selecting a delegate, propose to assign them certain privileges in the DAO, or update a registry. This would involve calling other canisters or emitting an event. Similarly, if a delegate is removed, inform the DAO to revoke their privileges. This integration ensures the off-chain (chat-based) process is reflected on-chain for formal governance. This is complex but a worthwhile goal.
* **Automated Scheduling:** The bot could automatically start a new sortition cycle at regular intervals (like every 3 months or whatever term is). Using stable timers or a long heartbeat interval, it could notify the group “Next election starting now” without admin intervention. We’d need a reliable time-of-term tracking and possibly a consensus from group on schedule.
* **Improved Discussion Support:** One limitation is the single chat for all groups. In future, if OpenChat allows programmatic group or thread creation, the bot could create sub-groups for each fractal round’s groups and invite the relevant users to those. Then it could manage those sub-chats (maybe even automatically archive them after use). This would greatly improve the quality of deliberation. Alternatively, the bot could facilitate scheduled video calls or external links for group discussions (like providing a Jitsi/Zoom link for each group if real-time meeting is desired – outside direct OpenChat scope but could be in messages).
* **Secret Ballots:** To reduce social pressure, future versions might implement secret voting. This could be done by having users DM their vote to the bot instead of public command (the bot would still know, but others wouldn’t). Or use cryptographic commitments in chat (probably too heavy). If DM approach: each user could open a direct chat with bot and type their vote command there. The bot would need to correlate that with the group context and count accordingly. This is doable and might be a config option (e.g., allow DM voting for privacy).
* **UI Improvements:** OpenChat might support richer UI for bots (like interactive buttons or cards). If in future there's a way to present a vote UI or a poll, the bot could use that instead of requiring command syntax. That depends on OpenChat feature development.
* **Load and Performance:** If this bot is used in very large communities or very frequently, performance tuning may be needed:

  * Optimize data structures (e.g., use tries or stable structures that minimize updates).
  * Possibly distribute work (though probably not needed given the scale).
  * Ensure that even if hundreds of messages (join/vote) come in short time, the canister can handle them (the IC can queue them; our processing is fairly straightforward O(n) for some tasks).
  * We might also implement chunking of big messages (like group announcements) to avoid hitting message size limit.
* **Enhanced Logging and Analytics:** Provide more insight to the community or devs:

  * For instance, after a cycle, produce a summary: how many participated, how many rounds, maybe an anonymized breakdown of votes, etc.
  * Could store historical stats to show trends (maybe via a command `/history` that lists past delegates and dates).
  * Such transparency could help refine the process (e.g., if participation is dropping, etc.).
* **Permission Management in-Bot:** We might allow certain users to become moderators of the bot via commands (like an `/addadmin @user` command that the current admin can use to delegate admin rights).
* **Resilience:** Consider scenarios like what if the bot canister is temporarily out of cycles or stopped. After topping up or restarting, can it resume the process? We should ensure the state is intact and maybe have the heartbeat catch up on missed time. E.g., if offline and a deadline passed, when it comes back it should still proceed. Because time now > deadline, our heartbeat would then trigger transitions immediately. That should be fine as long as state was saved.
* **AI integration:** As a fun thought, one could integrate an AI to help the process. For example, the bot could summarize group discussions or prompt questions to ensure thorough evaluation (some fractal processes involve set questions). Or, if in a final interview round, the bot could auto-ask each finalist some standardized questions and gather answers (makes sense if the process is not live).
* **UI front-end**: Possibly create a dashboard outside of chat (a web frontend) that visualizes the bracket or fractal tree of the sortition as it happens. The bot could output data that a front-end can pick up. This could help users follow along visually. This is beyond the chat, but could be an optional web component reading from the bot’s state (if exposed via queries).
* **Testing and Formal Verification**: As governance is sensitive, future work could include writing property tests or even formal verification of certain properties (e.g., that the final selection is uniformly random among finalists, that if majority want someone removed it always happens, etc.). This would increase trust in the bot’s correctness.

These future considerations highlight that while the prototype is functional, governance is an evolving domain. The community should iterate on this tool based on real-world usage. Feedback loops (maybe the bot itself can poll users for feedback after a cycle) could guide what to implement next.

## User Stories with Acceptance Criteria

To ensure the specification can be translated into actual development tasks, below is a collection of **user stories** with acceptance criteria. These stories cover the major functions from various user perspectives (admin, participant, general member, etc.). Developers can use these to guide implementation and testing.

1. **User Story: Start a New Governance Cycle**
   *As a community admin, I want to initiate a new fractal sortition cycle via the bot so that the community can begin the process of selecting a new delegate.*
   **Acceptance Criteria:**

   * When an authorized admin sends the command `/startsortition` in the group chat, the bot starts a new cycle *only if* no other cycle is currently active.
   * The bot must respond with a visible announcement in the group that nominations are open, including the deadline for nominations (as per configuration) and instructions for joining ("/join").
   * If a cycle is already active, the bot should reject the command with a message like “A sortition cycle is already in progress” and not reset anything.
   * Only a user designated as admin (pre-configured principal or the user who registered the bot, etc.) can trigger this. If a non-admin tries, the bot responds with an unauthorized message.
   * Starting a cycle should initialize the internal state: clear any leftover data from previous cycle, set the nomination deadline correctly, and allow the bot to accept `/join` commands.
   * After start, the `/status` command should reflect that nominations are open and show the end time.

2. **User Story: Join as Candidate**
   *As a community member, I want to nominate myself to participate in the delegate selection so that I have a chance to be chosen.*
   **Acceptance Criteria:**

   * When a user sends `/join` during the Nomination stage:

     * The bot adds that user to the participants list (if not already added).
     * The bot sends a confirmation message tagging the user or stating they joined.
     * The total count of participants should increment. The confirmation could include the updated count.
   * If the same user sends `/join` again (duplicate), the bot should either ignore silently or reply “You have already joined.” (preferring a reply to reduce confusion).
   * If `/join` is sent after nominations closed or when no cycle is active, the bot should reply with a message like “Nominations are not open at this time,” and not add them.
   * If possible, the bot should record the user’s name and principal for use later. The user’s identity in state should match their OpenChat identity.
   * The act of joining should be reflected in subsequent `/status` queries (e.g., number of participants).

3. **User Story: Nomination Stage Timeout**
   *As the system, I want the nomination stage to automatically close after the specified duration so that the process can move forward without manual intervention.*
   **Acceptance Criteria:**

   * When the nomination deadline is reached (time now >= deadline), the bot should cease accepting new `/join` commands (any received after this point get a “closed” response).
   * The bot should automatically proceed to group formation: it randomly groups all joined participants into groups of configured size.
   * The bot must post an announcement listing the groups and their members, and provide instructions about the next stage (e.g., “discuss within your group and vote using /vote <name> by X time”).
   * The announcement should include the Round number (Round 1) and the voting deadline for that round.
   * The time from nomination closing to posting groups should be minimal (ideally immediate or within a few seconds, done via heartbeat or triggered action).
   * If there were zero participants, instead of forming groups, the bot should announce that no one joined and end the cycle gracefully (no proceeding to rounds).
   * If one participant, the bot should handle this as an edge: either directly declare them winner or put them in a single group and then final (could just declare to simplify). Acceptance: if 1 participant, bot announces that participant as delegate by default.

4. **User Story: Voting for Group Representative**
   *As a participant in a group, I want to cast my vote for who should represent my group, so that our group selects the best candidate among us.*
   **Acceptance Criteria:**

   * During a voting round, when a participant sends `/vote @Username`:

     * The bot validates that there is an active voting round and that the sender is part of a group in this round.
     * The `Username` (or mention) is parsed; the bot checks that this target is indeed in the same group as the voter.
     * If validation passes, the bot records the vote (one vote per participant).
     * The bot sends a confirmation message like “Vote recorded” (possibly mentioning voter or just generic). This can be in group chat or DM; in group is acceptable feedback.
   * If the sender is not a participant (or not in any group), the bot should respond with an error like “You are not eligible to vote in this round.” and not record anything.
   * If the target name is not recognized or not in voter’s group, bot responds “Invalid vote target. Please vote for a member of your group.”
   * If the user already voted earlier in the same round, the bot should reject the second vote: e.g., “You have already voted.” (No changing votes in this prototype.)
   * If the vote is cast after the round’s deadline (maybe due to slight lag), the bot should reject: “Voting has closed.”
   * All valid votes should be stored such that by the deadline the correct tallies per group are available.

5. **User Story: Group Round Conclusion**
   *As the system, I want to determine the winner of each group once the voting period ends, so that we can advance to the next round or final selection.*
   **Acceptance Criteria:**

   * When the voting deadline for a round is reached (or if all votes are received sooner and early-end logic triggers):

     * The bot tallies votes for each group.
     * For each group, identify the participant(s) with the highest number of votes.
     * If there is a clear single winner (highest votes unique):

       * Mark that participant as the group winner.
       * They advance to next stage.
     * If there is a tie for top votes:

       * The bot resolves the tie by randomly selecting one of the tied candidates.
       * The chosen one is winner; others are essentially eliminated (unless they tie for top so they also had top votes but we only take one).
       * The bot’s result announcement should mention that it was a tie and random selection was used.
     * If a group had no votes cast (all abstained or absent):

       * The bot randomly picks one member of that group to advance (ensuring someone advances).
       * Mention this in results (e.g., “no votes, X advances by random draw”).
     * The bot posts a **Round Results** message in the chat listing each group and who won. Use bold or names for winners. Optionally include vote counts. E.g., “Group1: Alice (3 votes) wins, Group2: Bob wins (tie-break), etc.”
   * After announcing winners, if more than one winner exists (meaning multiple groups):

     * The bot immediately initiates the next round: form new groups out of the winners. (Or if exactly one winner, skip to final outcome.)
     * The bot announces the next round group(s) similarly with deadline for that round’s voting. (This could be Round 2 or Final depending on count.)
   * If only one group was present in the round (meaning it was effectively final shortlist):

     * Instead of a “winner per group” scenario, that one group’s outcome will directly yield the final delegate. If we intended random final selection regardless of votes, then if we are in final round, after discussion we pick random among them. But likely if it’s one group and we did a vote that picks one, that somewhat contradicts the fractal final random idea. However, acceptance can be: if one group remains at any point, treat that as final. Either:

       * Option A: If one group left >1 person, perform random selection among them (no further vote).
       * Option B: Let them vote one more time and that person is final (that reduces randomness at final stage).
     * According to spec, we do random final. So acceptance: if after previous round winners <= groupSize, do final sortition. (We should already cover final selection in next story.)

6. **User Story: Final Delegate Selection**
   *As a community member, I want the final delegate to be selected fairly from the top finalists so that the outcome combines merit and luck (fractal sortition).*
   **Acceptance Criteria:**

   * When the process has narrowed down to a final shortlist of candidates (either via rounds until <= group size or explicitly marked finalists):

     * The bot performs a **random selection** among those finalists to pick the delegate. (Use secure random source IC provides.)
     * The bot announces the final delegate in the group chat with a congratulatory message, clearly naming the winner. E.g., “Alice is selected as the new delegate.”
     * The announcement should also mention that it was chosen randomly from the finalists to be transparent. Possibly list the finalists. E.g., “Finalists were Alice, Bob, Carol. Randomly selected Alice.”
   * The bot updates its state to set `currentDelegate = Alice` (the chosen one) and records the term start time.
   * The bot should also store the other finalists as `reserveCandidates` for potential use if replacement needed.
   * If any action is expected from the delegate (like to accept or some oath), that’s out of scope, but we might simply mention “Alice, please confirm you accept this role” in message optionally (not enforced technically).
   * The cycle state is marked completed after this. No further rounds.

7. **User Story: Check Status**
   *As a community member, I want to get an update on the governance process at any time so that I know what’s going on or what I need to do.*
   **Acceptance Criteria:**

   * When a user sends `/status`, the bot replies with a summary of the current stage and pertinent info. Depending on stage:

     * If in Nomination: “Nominations open until <time>. X participants have joined so far.”
     * If in a Voting Round: “Round Y in progress: <number of groups> groups formed. Voting ends by <time>. \[Optionally, number of votes cast so far in aggregate or per group if we reveal that, but likely not reveal per group status]. If the user asking is a participant, maybe indicate their group and whether they have voted.
     * If awaiting final selection (though our design immediately selects final, no waiting stage).
     * If cycle completed and delegate in office: “No active cycle. Current delegate: Alice (since Jan 1, 2025). Next election expected around <if known>.”
     * If no cycle and no delegate (vacant): “No active cycle. The delegate position is currently vacant.”
   * The status message should be concise and formatted clearly. It can be one or few lines.
   * This command should work for anyone (no restrictions).
   * The info must be real-time accurate based on bot’s state (so update it after each event). E.g., after each join it should reflect new count, after grouping show groups, etc.

8. **User Story: Get Help Information**
   *As a new or curious member, I want to see a list of commands and their usage so that I know how to interact with the bot.*
   **Acceptance Criteria:**

   * When a user sends `/help`, the bot responds with a message listing available commands and a brief description of each.
   * The list should include at least: startsortition (if user is admin, but it can list it with note like “(admin only)”), join, vote, petition, status, help. Possibly cancel if we want to expose it (maybe not, since it's admin only and potentially sensitive).
   * The help text should be well-formatted (each command on new line or bullet, with maybe bold command and dash description).
   * This message should be sent as a normal text reply in chat (or DM if the platform does that, but group is fine).
   * Ensure that the help content matches the actual implemented commands.
   * Help can be accessible at any time, even when no cycle active (just describing usage in general).

9. **User Story: Petition No-Confidence**
   *As a community member, I want to initiate a petition to remove the current delegate so that if they have lost support, we can trigger a change in leadership.*
   **Acceptance Criteria:**

   * If a delegate (currentDelegate) exists, when a user sends `/petition`:

     * If no petition is currently active, this starts a new petition. The bot records the initiator as a signer.
     * The bot announces the petition in the group, specifying the target delegate’s name, the required number of signatures (or percentage) needed to succeed, and the deadline by which they must be gathered.
     * The petition should set appropriate threshold based on config and possibly term timing (e.g., early/late term thresholds). E.g., “Needs 15 signatures within 7 days.”
     * If a petition is already active, the bot instead treats `/petition` as a user trying to sign (see next story). Possibly the initiator doing it again would be counted as duplicate, but presumably they wouldn’t do that.
     * If there is no current delegate (position vacant or during election), the bot should respond with something like “There is no delegate in office to petition against.” (So it won’t start a petition).
     * Only group members should count; if someone outside (not in group) tried (not possible since only group sees it), not an issue.
   * The state moves to petition collecting mode with deadline.

10. **User Story: Sign Petition**
    *As a community member, I want to support an existing petition by adding my signature so that I can contribute to removing a delegate I agree is unfit.*
    **Acceptance Criteria:**

    * When a petition is active and a user sends `/petition`:

      * If that user has not signed yet, add them to the petition signers.
      * The bot should acknowledge in some way: possibly an updated count. e.g., “Petition support: 5/15 signatures.” This could be either a new message or an edit (OpenChat might not allow editing easily via bot, so likely a new message or a short confirmation tagging the user like “@John signed the petition (5/15).”). The exact format is up to design, but there must be feedback that their action counted.
      * If that user already signed before, the bot should not double count. It can respond, “You have already signed this petition.”
      * If the required threshold is not yet reached, petition remains open. If threshold is reached or exceeded by this signature:

        * The bot immediately recognizes success. (E.g., if required 15 and this was the 15th)
        * The bot announces that the petition succeeded (no-confidence passes). See next story for outcome.
        * No further signatures needed; further `/petition` commands could be ignored or respond with “Petition already succeeded.”
      * Each user can only sign once; ensure set logic.
    * If current time > petitionDeadline and someone tries to sign, petition would already be concluded (we should ideally not allow signing after time; though if they sent at same moment, if bot processes after, it should reject as expired). The bot can reply “The petition period has ended.”

11. **User Story: Petition Success – Remove Delegate**
    *As a system/community, I want an approved no-confidence petition to automatically remove the delegate and install a replacement so that governance continuity is maintained according to the community’s will.*
    **Acceptance Criteria:**

    * When the petition gathers enough signatures within the time limit (or at the exact end of time if threshold reached that moment):

      * The bot finalizes the petition with success. It should post a message like “The no-confidence petition against Alice has succeeded with X signatures. Alice will be removed as delegate.”
      * The bot updates state: set currentDelegate to null or mark Alice as removed. Also, end the petition (petitionActive false).
      * The bot then checks if there are reserve candidates available from the last election:

        * If yes, it should promote one of them to delegate. The selection method as decided (random from reserves, or maybe the one who had the next highest something; but we choose random to align with fairness).
        * The bot announces the new delegate: e.g., “As a result, **Bob** will assume the role of delegate (selected from remaining finalists). He will serve the remainder of the term.”
        * Update currentDelegate to Bob, remove Bob from reserve list.
      * If no reserves exist:

        * The bot announces that the position is now vacant and a new election should be held. Possibly prompt admins to start a new cycle. (It does not auto start one in this version.)
        * currentDelegate remains null (vacant).
      * All of this should happen promptly after threshold is hit (within the same call that added the last signature, ideally).
    * If any users try petition command after success, handle gracefully (could say “already done” or ignore).
    * The bot should not allow any new petition immediately after; ideally implement a cooldown (e.g., cannot start another petition for maybe a few weeks). This is harder to test in immediate term, but can simulate by trying to start a petition right after one just succeeded. The bot should reply “A delegate was just removed. Please allow some time before the next petition.” (Acceptance: presence of a cooldown mechanism as per config.)

12. **User Story: Petition Failure**
    *As a community member, I want to know that if a petition fails to gain support, the delegate remains and the matter is closed, to avoid ongoing uncertainty.*
    **Acceptance Criteria:**

    * If the petitionDeadline passes without reaching threshold:

      * The bot declares the petition failed. It should post a message such as: “The petition against Alice has failed (only Y out of Z required signatures). Alice remains as delegate.”
      * The bot sets petitionActive to false and clears signers.
      * The delegate stays unchanged.
      * A cooldown might apply here too: maybe prevent starting a new petition for a short period to avoid continuous petitions. If someone tries immediately another `/petition`, the bot might respond “A recent petition just concluded. Please wait before trying again.” (The acceptance is that some prevention of spam is in place, can be a simple time threshold like a week).
    * If during the petition period the delegate’s term ended or they stepped down, ideally the petition becomes moot (but that scenario is unlikely to coincide; if it did, we'd cancel the petition, but okay if not implemented explicitly).
    * Ensure no memory of signers carries over once petition over (for privacy maybe drop them, and in case next petition starts fresh).

13. **User Story: Cancel Cycle (Admin)**
    *As a bot admin, I want to cancel an ongoing sortition process in case of an error or change in plans so that I can restart or stop the process safely.*
    **Acceptance Criteria:**

    * When an admin sends `/cancel` (assuming we implement it), and a cycle is currently active (nominations or rounds):

      * The bot immediately stops the cycle. It should wipe or archive the cycle state. No delegate is selected from it.
      * It sends a message to the group: “The current sortition cycle has been cancelled by an admin. No delegate was chosen.”
      * After this, the bot’s state returns to idle (no active cycle).
      * If `/cancel` is used when no cycle is active, bot might respond “No active cycle to cancel.”
      * Only admin can use it; others get unauthorized message.
    * This is mainly for admin control in exceptional cases. The acceptance is that it cleanly stops the process and communicates to participants that it’s off.

14. **User Story: Deployment/Integration Validation** (Dev/Admin story)
    *As a developer or admin, I want to verify that the bot is correctly deployed and connected to OpenChat so that users can actually use it.*
    **Acceptance Criteria:**

    * After deploying the canister and registering the bot, sending a simple command like `/help` or `/status` in the group should result in an immediate, correct response from the bot.
    * If the bot does not respond, that indicates integration issues (not acceptance, but test scenario). Acceptance is that we do get a response.
    * The bot’s name should appear as a member in the group, and the messages it sends should show that name.
    * The commands in OpenChat should not be visible to others as raw text (OpenChat typically hides the command and only shows the bot’s response, to look clean). For instance, when someone sends `/join`, ideally it triggers bot response and maybe the original message is hidden or shown only to user (depending on OpenChat). We can’t control that from bot, but ensure our part is fine.
    * The bot should handle multiple commands sequentially without crashing (if several people join at once, etc., just ensuring no race conditions).

Each user story above can be tested independently to verify that the bot meets the requirements. They also serve as a checklist during development. For example, before releasing, ensure that a petition threshold indeed triggers removal and that a replacement is chosen properly from reserves (test scenario: have 3 finalists, choose 1, remove them via petition, ensure one of the other 2 becomes delegate).

By satisfying all these acceptance criteria, we ensure the Fractal Sortition Bot behaves as intended in all critical aspects of its functionality, providing a smooth and fair governance experience for the community.
