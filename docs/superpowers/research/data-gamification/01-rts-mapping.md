# Paradigm 1: The RTS Mapping (Macro-Scale Command)

**Concept:** You are the "Overwatch Commander" aboard a flagship or orbiting command center (Homeworld/Stellaris aesthetic). The distributed system is your vast, autonomous fleet. You command resources, direct swarms of automated drones (agents), and manage macro-level logistics across star systems (knowledge graphs).

## Core Data Mappings

| Data Type | Game World Representation | Visual Appearance | Player Interaction |
| :--- | :--- | :--- | :--- |
| **Agents (10k Workers)** | Swarm Drones / Harvesters | Glowing geometric ships forming fluid, boids-like clusters. Color-coded by task (e.g., Red = Analysis, Blue = Fetch). | Select-and-drag bounding boxes, assign waypoints, group hotkeys (Ctrl+1). |
| **Agent Orchestrations** | Fleet Formations / Carrier Groups | A unified command ring enveloping a group of drones, tethered by light-beams to a central "Carrier" node. | Assigning macro-objectives ("Harvest Sector 4", "Defend Core"). |
| **Knowledge Graph** | The Star Map / Tech Tree | A glowing, 3D web of interconnected star systems or neural nodes. Expanding frontiers push back the "Fog of War." | Pinch-to-zoom, rotating the 3D map, clicking nodes to view localized telemetry. |
| **S3 Storage** | Resource Asteroids / Nebulas | Massive, crystalline structures or dense gas clouds floating in space. Size indicates storage volume. | Directing harvester swarms to orbit and extract data streams (lasers mining the rock). |
| **NATS Eventing** | Hyperspace Radar / Pings | Constant, rippling sonar pings across the UI minimap. High traffic creates glowing heatmaps of activity. | Monitoring the minimap for anomalies, clicking alerts to snap the camera to event clusters. |
| **Execution Traces** | Flight Paths / Engine Trails | Fading neon wakes left behind by drones, showing their historical travel between resource nodes and carriers. | Hovering over a trail opens a tooltip showing the exact timestamp and payload of the journey. |
| **Artifacts** | Refined Relics / Blueprints | Glowing, rotating cubes or specialized modules towed back to the flagship. | Dragging and dropping artifacts into the flagship's research bay to unlock new capabilities. |
| **Compliance Exclusions** | Orbital Strikes / Black Hole Collapses | A targeted, localized singularity or a massive, destructive beam from the flagship that obliterates a corrupted node. | Activating a "Purge Protocol" via a highly deliberate, dual-key confirmation UI (turning two digital keys). |

## Scenario Applications

### Scenario 1: The 10,000 Agent Burst
**Visuals & Feel:** 
When a massive job is triggered, the flagship's hangar bays open, and a blinding stream of 10,000 micro-drones pours out like a river of light. To prevent visual clutter, the engine dynamically abstracts them: individual ships merge into fluid, glowing "storm clouds" that engulf target resource nodes. 
**Interaction:** The Commander doesn't micro-manage these; they paint a target zone with a broad brush tool, and the swarm autonomously distributes itself based on internal NATS load-balancing.

### Scenario 2: HOTL (Human-On-The-Loop) Media Pipelines
**Visuals & Feel:** 
Represented as massive "Sensor Array Arrays" or "Refinery Ships" processing raw planetary data. A continuous, thick beam of data (video stream) flows from a planet into the refinery. Above the refinery is a "Production Queue" UI showing segmented blocks of data.
**Interaction:** The Commander watches the queue. Occasionally, a block flashes red (an AI uncertainty flag). The Commander clicks the block, opening a localized picture-in-picture (PiP) feed of the media. They make a rapid "Approve/Reject" swipe, and the block turns green, resuming the flow without stopping the overarching machine.
