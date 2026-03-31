# 03: Mechanics and Stress Tests

This document maps the 8 core distributed system stress tests into specific, playable game mechanics across both the RTS (Macro) and RPG (Micro) paradigms.

## 1. The 10,000 Agent Burst (Massive Concurrency)
* **System Event:** 10k parallel workers spinning up instantly to process a massive dataset.
* **RTS Mechanic:** **"The Swarm Deployment."** The Commander highlights a massive enemy armada or resource nebula. A "Hyperspace Gate" opens, pouring out thousands of micro-fighters. The UI handles the visual load by grouping them into fluid, glowing clouds (boids) rather than rendering 10k individual units.
* **RPG Mechanic:** **"Calling the Banners."** A cinematic off-screen battle. The Protagonist assigns companions to lead massive, unseen armies. The player watches a tactical map where "Unit Strength" bars push back and forth based on the companions' stats and chosen tactics.

## 2. HOTL Media Pipelines (Human-On-The-Loop)
* **System Event:** Continuous asynchronous video/audio processing requiring occasional human validation of AI-flagged uncertainties.
* **RTS Mechanic:** **"The Refinery Queue."** A massive space station (the pipeline) continuously extracts a beam of raw materials (data). Above the station is a queue of glowing blocks. Blocks flash red when the system is "jammed." The Commander clicks a red block, instantly viewing a PiP snippet of the issue, quickly selecting "Accept" or "Reject" to unjam the queue without halting the overarching process.
* **RPG Mechanic:** **"The Med-Bay Analyzer."** A companion is scanning a strange alien artifact. A mini-game pops up requiring the player to match frequencies or solve a quick puzzle (representing the validation step) before the companion can proceed with the analysis.

## 3. HITL Low-Latency Chat (WebRTC Human-In-The-Loop)
* **System Event:** Sub-second, synchronous interaction between a human operator and an agent requiring immediate feedback.
* **RTS Mechanic:** **"Direct Override."** The Commander zooms down from the macro-view, taking direct control of a single, crucial Hero Unit or capital ship, making split-second targeting and movement decisions.
* **RPG Mechanic:** **"The Holo-Comm Interrupt."** A real-time, high-stakes dialogue Quick-Time Event (QTE). A companion calls in under fire or mid-negotiation. A timer ticks down rapidly as the player must choose a critical dialogue option ("Fire!" or "Fall Back!").

## 4. NATS Event Flood (Message Storms)
* **System Event:** Millions of telemetry events firing rapidly, potentially overwhelming the message broker or UI.
* **RTS Mechanic:** **"Sensor Jamming / Ion Storm."** The minimap fills with intense, blinding static or a massive swirling nebula. The Commander must deploy specialized "Relay Probes" to cut through the noise and re-establish clean telemetry lines.
* **RPG Mechanic:** **"Psychic Static."** The party enters a zone of overwhelming telepathic noise. Companions suffer a "Confusion" debuff, causing their combat AI to act erratically until the source of the interference (the event flood) is located and destroyed.

## 5. S3 Rate Limiting (Throttling)
* **System Event:** The storage backend rejects requests due to exceeding IOPS or bandwidth limits.
* **RTS Mechanic:** **"Logistics Bottleneck / Asteroid Depletion."** Resource harvesters queue up endlessly around a single node. The UI flashes "Supply Line Congested." The Commander must build secondary storage depots or research "Deep Core Drilling" to increase the extraction rate.
* **RPG Mechanic:** **"Encumbrance / Fatigue."** The party's inventory is full, or they are exhausted from carrying heavy loot. They move slower, and cast times increase. The player must drop items, use a "Stamina Potion," or find a merchant/stash to offload the data.

## 6. Compliance Exclusions (Targeted Deletion/Purge)
* **System Event:** A legal or security requirement mandates the immediate, verifiable deletion of specific data or agents.
* **RTS Mechanic:** **"Exterminatus / Orbital Strike."** A highly deliberate, multi-step UI action (e.g., turning two digital keys, typing a confirmation code). A massive, destructive beam obliterates a corrupted sector of the map, leaving a permanent "Quarantine Zone" scar.
* **RPG Mechanic:** **"The Memory Wipe / Paragon Choice."** A heavy narrative moment. The Protagonist must decide to destroy a forbidden, powerful artifact or wipe a companion's corrupted memory. This is an irreversible, high-stakes dialogue choice with permanent consequences.

## 7. Knowledge Graph Expansion (Deep Linking)
* **System Event:** The system discovers vast new connections, expanding the known universe of data exponentially.
* **RTS Mechanic:** **"Fog of War Cleared / Hyperspace Lanes Opened."** A "Recon Probe" reaches the edge of the map. Suddenly, a massive web of new star systems illuminates, revealing new resource nodes, enemy factions, and hyperspace jump routes connecting previously isolated sectors.
* **RPG Mechanic:** **"Unlocking the Ancient Vault."** The party pieces together clues to open a hidden door. Inside is a glowing holographic library. The party's global "Intelligence" stat increases, unlocking new dialogue options with NPCs and revealing hidden weaknesses on bosses.

## 8. Cascading Worker Failure (Agent Health Drop)
* **System Event:** Nodes begin crashing sequentially due to a bad deployment, memory leak, or unhandled exception.
* **RTS Mechanic:** **"The Plague Ship / EMP Cascade."** A virus infects a harvester drone. If it returns to the Carrier, the infection spreads to the whole fleet. The Commander must quickly select infected units and choose "Self Destruct" to save the rest, or deploy "Repair Drones" (hotfixes) rapidly before the fleet shuts down.
* **RPG Mechanic:** **"Mind Control / Spreading Poison."** A boss enemy casts a contagious status effect. One companion turns hostile or starts rapidly losing HP. The player must quickly use specific consumable items (antidotes/dispels) or isolate the infected companion before the entire party wipes.