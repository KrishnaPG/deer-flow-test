# Paradigm 2: The RPG Mapping (Micro-Scale Command)

**Concept:** You are the "Captain/Protagonist" (Mass Effect/Baldur's Gate 3 aesthetic). Your AI agents are distinct, specialized companions, each with a personality, skillset, and deep backstories woven into their algorithmic functions. You lead an elite strike team through nuanced, dialogue-heavy missions (research, data extraction, and synthesis).

## Core Data Mappings

| Data Type | Game World Representation | Visual Appearance | Player Interaction |
| :--- | :--- | :--- | :--- |
| **Agents** | Companions / Party Members | Detailed 3D character models (e.g., an armored Paladin, a cloaked Rogue, a holographic AI), standing in the "Ready Room" or following you. | Opening Character Sheets, equipping gear, leveling up skills, speaking face-to-face. |
| **Agent Health/History/Performance** | HP/Stamina/XP/Skill Trees | Health bars above their heads, experience points, glowing skill icons in their UI panel, visible damage/fatigue. | Applying "Stimpaks" (restart/recover), choosing new abilities (fine-tuning models) on level up. |
| **Chat Sessions** | Dialogue Trees | A cinematic, over-the-shoulder camera angle with a dialogue wheel of choices (e.g., "Investigate," "Aggressive," "Diplomatic"). | Selecting nuanced dialogue options that steer the agent's internal logic and execution paths. |
| **Execution Traces** | Quest Logs / Journal Entries | A stylized, leather-bound journal or high-tech datapad UI listing steps, completed objectives, and failed tasks. | Reviewing past actions, clicking a failed step to reveal the exact error log disguised as a "Mission Report." |
| **Artifacts** | Loot / Crafted Gear | Unique weapons, armor, or key items (e.g., a "Datacore Drive," an "Ancient Tome") found in the world or crafted. | Equipping items to specific agents to buff their capabilities for specific tasks. |
| **Knowledge Graph** | The Codex / Investigation Board | A physical corkboard with string connecting photos, or a glowing holographic lore library. | Adding new clues, drawing connections between nodes to unlock new dialogue options or quests. |
| **Metrics/Progress Bars** | Cast Times / Action Points (AP) | A glowing ring filling up around a character casting a spell or hacking a terminal, or limited AP for turn-based moves. | Watching the bar fill; deciding whether to "Interrupt" or "Boost" the action with player abilities. |
| **S3 Storage** | Ancient Ruins / Vaults | Massive, locked doors or glowing chests in the environment containing hidden knowledge or raw materials. | Ordering a rogue companion to "Pick Lock" (decrypt) or a brute to "Smash" (brute-force extraction) the vault. |

## Scenario Applications

### Scenario 5: HITL (Human-In-The-Loop) Low-Latency Chat
**Visuals & Feel:** 
This maps directly to intense companion interactions. You are in a high-stakes conversation with your AI counterpart. The camera pushes in close. A low-latency WebRTC stream is represented as an urgent, real-time "Holo-Call" from a squadmate under fire or deep undercover.
**Interaction:** Instead of a static dialogue tree, this is a Quick-Time Event (QTE) conversation. The companion presents real-time findings, and a timer ticks down as you must choose to "Proceed," "Abort," or "Change Tactics." Your choice instantly alters their behavior in the field.

### Scenario: Agent Track Records and Leveling
**Visuals & Feel:** 
An agent who consistently succeeds at complex orchestration tasks earns XP and a visible reputation metric (e.g., "Renegade" for aggressive, fast completions; "Paragon" for safe, verifiable completions). 
**Interaction:** You visit the ship's armory. The high-performing agent has unlocked a new "Skill Node." You spend points to unlock "Deep Search Protocol" (a new backend capability) or "Stealth Extraction" (bypassing certain rate limits or firewalls). Their appearance evolves, perhaps gaining sleeker armor or brighter holographic auras to reflect their elite status.
