# Sci-Fi RTS HUD Research Reference

Compiled research from prior session. Do not re-research — use this as the canonical reference for HUD design decisions.

---

## Section 1: Homeworld 2 HUD Analysis

### Layout Zones

- **Top strip**: minimal resource readout, no decorative chrome
- **Bottom center**: command console (context-sensitive, 9 states)
- **Bottom left/right flanking**: build manager, research tree, sensor manager
- **No permanent side panels** — world viewport is maximally exposed

### Design Philosophy: "Minimize Chrome, Maximize Diegesis"

The HUD should feel like it belongs in the world — not painted on top of it. Every UI element earns its presence by being functionally necessary. Decorative chrome is eliminated. Panels dissolve at their edges into the starfield rather than presenting hard rectangular borders.

### Cinematic Qualities

- **Dissolving-edge transparent panels**: panels fade into space at their perimeter — no hard border strokes
- **Chromatic restraint**: exactly 5 semantic colors used throughout — no color creep
- **Live 3D rotating ship model** in the command console when a ship is selected — bridges diegetic and non-diegetic UI
- **Camera-zoom-responsive HUD reconfiguration**: as the camera zooms out to strategic view, the HUD simplifies; tactical zoom restores full detail
- **CRT scan-line panel texture**: subtle horizontal scan lines at ~3% opacity give panels a screen-rendered quality, reinforcing the holographic diegesis

### Command Console: 9 States

| State              | Trigger                  | Content                                             |
| ------------------ | ------------------------ | --------------------------------------------------- |
| Empty              | No selection             | Minimal chrome, ambient scan animation              |
| Single ship        | One ship selected        | 3D ship model, name/class, stat grid, ability icons |
| Multi-ship         | Box select / group       | Portrait grid of selected units, aggregate stats    |
| Fleet              | Fleet selected           | Fleet composition summary, formation controls       |
| Station            | Station selected         | Docking status, defense grid, build queue           |
| Resource collector | Harvester selected       | Cargo capacity, route, return status                |
| Research ship      | Research vessel selected | Active/queued research, progress bars               |
| Mothership         | Mothership selected      | Full production queue, fleet status aggregate       |
| Hyperspace         | Hyperspace in progress   | Jump countdown, destination, abort option           |

### Supporting UI Panels

- **Build Manager**: queued units with progress bars, cancel controls, sorted by production node
- **Research Tree**: grid of nodes with dependency lines, hover for details, gated nodes visually dimmed
- **Sensor Manager**: toggles for fog of war layers, sensor coverage map overlay

### Key Lesson

**Never use hard border strokes on primary HUD panels.** Panels must fade into space. This single rule is what separates cinematic HUD from a painted-on overlay.

---

## Section 2: Homeworld 3 Regression

### What Went Wrong

Homeworld 3 departed from HW2's transparency-first philosophy:

- Used **heavier, more opaque panels** with visible border strokes
- Panels felt painted on top of the scene rather than embedded in it
- Lost the dissolving-edge quality that made HW2 feel cinematic
- Reduced visual access to the 3D world — the defining spectacle of the game

### Reception

- Steam reviews: **Mostly Negative**
- Support ended: **November 2024**
- Critical consensus: the game failed to recapture the visual and atmospheric quality of HW2

### Lesson

Preserve HW2's transparency approach at all costs. Opacity creep is a death spiral: once panels get heavy, every subsequent designer adds more chrome to make their element "stand out," compounding the problem. Enforce the glassmorphism constraint in code (alpha cap, blur requirement) so it cannot be violated accidentally.

---

## Section 3: Classic RTS HUD — AoE2 & Rise of Nations

### Age of Empires 2 Layout

- **Top**: resource bar — wood / food / gold / stone counts + population (used / cap)
- **Bottom-left**: minimap
- **Bottom-center**: unit portrait + name + health/attack/defense stats
- **Bottom-right**: 3×5 command grid (15 command slots, contextual icons)
- **Left side**: event log (floating, scrollable, color-coded)

The 3×5 command grid established spatial muscle memory as a core RTS mechanic. Players memorize button positions across sessions. This must be preserved in any modern design.

### Rise of Nations Innovations

- **City status overlays**: thin rings around cities show population pressure and happiness — information delivered in world space rather than HUD
- **National borders visual**: semi-transparent territory shading on the map — diplomacy legible at a glance
- **Simultaneous research + military feedback**: RoN was one of the first RTS to show multiple parallel progress bars without cluttering the HUD
- **Cleaner iconography**: icons communicate meaning without labels — important for dense UIs

### Information Density Solution: Three-Tier Visibility

| Tier           | Trigger                  | Information Shown                                |
| -------------- | ------------------------ | ------------------------------------------------ |
| Always visible | —                        | Resources, alerts, minimap, unit count           |
| On hover       | Mouse over unit/building | Name, brief stats, hotkey reminder               |
| On select      | Click / box select       | Full stats, command grid, build queue, abilities |

This tiered approach prevents information overload while keeping all data accessible. **Never show all information at once.**

---

## Section 4: Modern Sci-Fi RTS References

### StarCraft 2

- **Per-race skinned panels**: Terran (industrial metal), Protoss (crystalline energy), Zerg (chitin organic) — same layout, radically different material
- **5×3 command card**: spatial muscle memory preserved; hotkey letters shown on each button
- **Alert banner system**: banners drop from the top of screen for critical events (unit under attack, building complete), auto-dismiss, click to jump camera
- **Portrait animation**: unit portraits are animated; idle animations reinforce faction personality

### Sins of a Solar Empire

- **Empire Tree sidebar**: vertical hierarchical list of all planets and fleets in the empire; click any entry to jump the camera to that location; health/status bars inline
- Solves the **massive scale problem** of 4X-RTS hybrids — when you have 50 planets, you need a structured list, not a minimap click hunt
- The sidebar is collapsible but defaults to open; its hierarchical grouping (Empire → Star System → Planet/Fleet) mirrors the command structure

### Ashes of the Singularity

- **Minimal chrome philosophy**: chrome kept to absolute minimum; world is the UI where possible
- **Strategic zoom levels change HUD density**: at planetary zoom, unit-level HUD elements hide; at unit zoom, strategic elements dim — the HUD is zoom-aware

### Terra Invicta

- **Information-dense sidebar**: accepts that some players want maximum data; sidebar scrolls with collapsible sections
- **Tooltip cascades**: tooltips contain tooltips — hover a stat to get its formula, hover a modifier to get its source — depth without clutter

---

## Section 5: HUD Design Principles

### Zone Architecture

| Zone            | Position                  | Width         | Height   | Purpose                                             |
| --------------- | ------------------------- | ------------- | -------- | --------------------------------------------------- |
| Resource Bar    | Top                       | 100%          | 4–6%     | Global state: resources, alerts, fleet summary      |
| World Viewport  | Center                    | variable      | variable | 3D world — sacred, never permanently obscured       |
| Left Event Feed | Left                      | 15–18%        | variable | Scrolling log: combat, build, research, diplomacy   |
| Right Panel     | Right                     | 15–18%        | variable | Secondary context: fleet tree, research, diplomacy  |
| Minimap         | Top-right or bottom-right | ~15% (square) | ~15%     | World overview: terrain, units, fog, camera frustum |
| Bottom Console  | Bottom                    | 100%          | 18–22%   | Contextual command space: selection, orders, build  |
| Modal Overlays  | Center (full-screen)      | 100%          | 100%     | Tech tree, diplomacy, settings — dimmed background  |

**The Center Viewport is Sacred.** No permanent HUD element may overlap the center viewport. World-space overlays (health bars, selection rings, waypoint lines) are acceptable because they are part of the scene.

### Color Language

| Token          | Hex       | Semantic Use                                    |
| -------------- | --------- | ----------------------------------------------- |
| Background     | `#050810` | Deep space — panel backgrounds, scene base      |
| Accent Cyan    | `#00CCDD` | Active selection, friendly units, UI chrome     |
| Accent Amber   | `#F0C040` | Resources, economy indicators, neutral warnings |
| Warning Orange | `#FF8800` | Incoming threat, low resource state             |
| Critical Red   | `#FF2200` | Under attack, unit death, critical failure      |
| Success Green  | `#00FF88` | Positive events, full health, research complete |
| Special Purple | `#8844FF` | Special abilities, hyperspace, psionic effects  |

### Typography

| Use Case           | Font           | Size | Weight  | Case       |
| ------------------ | -------------- | ---- | ------- | ---------- |
| Resource counts    | JetBrains Mono | 28px | Regular | Numeric    |
| Alert text         | JetBrains Mono | 20px | Regular | ALL-CAPS   |
| Zone headers       | IBM Plex Sans  | 16px | Medium  | ALL-CAPS   |
| Labels             | IBM Plex Sans  | 14px | Regular | ALL-CAPS   |
| Unit names         | IBM Plex Sans  | 14px | Regular | Mixed Case |
| Detail text        | IBM Plex Sans  | 12px | Regular | Mixed Case |
| Ticker / timestamp | JetBrains Mono | 10px | Regular | Mixed Case |

**Critical rule**: Use monospace for ALL numeric values to prevent layout shift when numbers change. A resource count jumping from `99` to `100` must not reflow surrounding elements.

### Glassmorphism Rules

```
Panel fill:   rgba(5, 8, 16, 0.75)      — dark enough to read text, transparent enough to see space
Panel border: rgba(0, 204, 221, 0.20)   — cyan ghost border, 1px
Blur:         backdrop-filter: blur(12px)  (CSS equiv; in Bevy: render-to-texture + blur pass)
```

**Hard rules:**
- NEVER use solid opaque panels for primary HUD elements
- Solid panels only for modal overlays (full tech tree, full inventory screens)
- Panel alpha must be between 0.65 and 0.85 — never fully opaque, never below 0.65 (readability floor)

### Animation Vocabulary

| Name            | Trigger                 | Duration  | Easing         | Description                                                  |
| --------------- | ----------------------- | --------- | -------------- | ------------------------------------------------------------ |
| Scan-line sweep | Panel open              | 120ms     | Linear         | Horizontal white line descends top-to-bottom over panel      |
| Data pulse      | Numeric value changes   | 80ms      | Ease-out       | Value flashes white then returns to normal color             |
| Threat ping     | Unit under attack       | 600ms     | Ease-out       | Expanding ring + color flash at map position                 |
| Build complete  | Build finishes          | 400ms     | Ease-out       | Brief particle burst at unit portrait location               |
| Panel slide     | Selection state changes | 150ms     | Ease-out-cubic | Panel slides in from edge                                    |
| Idle flicker    | Always                  | 2s period | Sine           | 0.3% opacity sine wave on panel borders — barely perceptible |

### Minimap Conventions

- **Shape**: Rectangular for function (matches viewport aspect ratio); octagonal mask for sci-fi aesthetic — use an octagonal clip mask over rectangular data
- **Threat indicators**: Red dots at unit positions, pulsing at 1 Hz when units are under attack
- **Fog of war**: Dark overlay at 40% opacity over unexplored regions
- **Camera frustum**: White rectangle outline showing the current viewport bounds — always visible
- **Click behavior**: Click to move camera; drag to pan; right-click for context menu (set rally point, etc.)

---

## Section 6: BG3 Panel Lessons (Applicable to Sci-Fi RTS)

Baldur's Gate 3 solved several panel UX problems that directly apply to sci-fi RTS ship cards and unit management.

### Three-Layer Tooltip Cascade

| Layer          | Trigger     | Content                                              |
| -------------- | ----------- | ---------------------------------------------------- |
| Hover          | Immediate   | Unit name + type                                     |
| Extended hover | 600ms delay | Full stats block                                     |
| Comparison     | Hold Alt    | Side-by-side comparison with currently selected unit |

Apply this to ship cards: hovering a ship in the fleet tree shows its name and class immediately; holding still for 600ms expands to full specs; Alt-hovering compares to the selected ship.

### Rarity Color Coding for Ship/Tech Tiers

| Tier      | Color            | Token                         |
| --------- | ---------------- | ----------------------------- |
| Common    | Grey `#888888`   | Standard units                |
| Uncommon  | Green `#00AA44`  | Upgraded variants             |
| Rare      | Blue `#2266FF`   | Elite units                   |
| Epic      | Purple `#8844FF` | Faction-specific specials     |
| Legendary | Amber `#F0C040`  | Unique heroes / capital ships |

### Action Bar with Hotkeys Shown

- Bottom command bar shows keyboard shortcut overlays when the player holds a configurable modifier key (default: Alt or Tab)
- Reduces learning curve: new players discover hotkeys passively rather than needing to read a manual

### Portrait Strip

- Top-center strip of fleet/wing portraits with health rings
- Click to select the unit; drag to reorder position in formation
- Health ring: full circle = full health; arc depletes clockwise; color transitions green → amber → red

### Tab-Based Console Switching

- Build / Research / Orders / Intel tabs
- Each tab has a keyboard shortcut shown on the tab label (e.g., `[B]uild`, `[R]esearch`, `[O]rders`, `[I]ntel`)
- Active tab indicated by cyan underline, inactive tabs at 50% opacity

---

## Section 7: Bevy / egui Implementation Patterns

### Event Log

```rust
// Data model
struct LogEntry {
    timestamp: f32,
    message: String,
    severity: LogSeverity, // maps to color
}
// Use VecDeque<LogEntry> with max capacity 200; drain front when full
// Render with egui::ScrollArea::vertical() + stick_to_bottom(true)
// Each entry: egui::Label with RichText colored by severity
```

### Command Console Tabs

```rust
// Panel: egui::TopBottomPanel::bottom("command_console")
// Tab bar: drawn manually with ui.painter()
//   - painter.rect_filled() for tab backgrounds
//   - painter.text() for tab labels + hotkey hints
// Tab content: rendered inside egui::Frame with panel_fill background
```

### Unit Portrait Grid

```rust
// egui::Grid with fixed cell size (e.g., 48x48)
// Per cell:
//   painter.rect_filled(cell_rect, 4.0, background_color)
//   painter.image(portrait_texture, portrait_rect, uv, Color32::WHITE)
//   painter.rect_filled(health_bar_rect, 0.0, health_color)  // overlay
```

### Resource Bar

```rust
// egui::TopBottomPanel::top("resource_bar")
// Per resource: icon (TextureHandle) + monospace Label
// Animated fill: store target_value: f32 in resource component
//   each frame: current = lerp(current, target, delta * 8.0)
//   display current.round() as integer
```

### Minimap

```rust
// Fixed-size egui::Frame (e.g., 240x240) in top-right SidePanel
// Use ui.painter() to draw:
//   - terrain tiles as colored rects
//   - unit dots (2px radius circles)
//   - fog of war as dark semi-transparent rects
//   - camera frustum as white rect outline (painter.rect_stroke)
// Data sourced from Bevy Query<(&Transform, &UnitType, &Faction)>
```

### Animation State

```rust
// All animation timers are Bevy Components on UI entities:
struct ScanLineAnim { progress: f32 }  // 0.0–1.0
struct DataPulse { intensity: f32 }    // 0.0–1.0, decays each frame
struct ThreatPing { radius: f32, alpha: f32 }

// Drive via Time<Real> in Update systems:
fn update_scan_line(mut q: Query<&mut ScanLineAnim>, time: Res<Time<Real>>) {
    for mut anim in &mut q {
        anim.progress = (anim.progress + time.delta_secs() / 0.12).min(1.0);
    }
}
// Pass computed values into egui each frame via painter calls
```

### Performance Rules

- Never upload textures per-frame — use `egui::TextureHandle` retained across frames
- Unit portraits: load once, store in a `HashMap<UnitId, TextureHandle>` resource
- Batch all custom painter calls within a single `ui.painter()` scope per panel to minimize draw calls
- For the minimap, draw all terrain in one pass, then units in one pass — two loops, not one per-unit painter acquisition

---

## Section 8: Aesthetic Mood Boards

### Holographic (Homeworld 2 / Halo) — RECOMMENDED

This is the target aesthetic for this project.

- **Dominant palette**: deep navy `#050810`, cyan `#00CCDD`, white `#E8F4F8`
- **Textures**:
  - Subtle hexagonal grid overlay at 5% opacity (background pattern)
  - Horizontal scan-line texture at 3% opacity (panel surface)
- **Typography**: thin-weight sans for labels, monospace for all data; generous letter-spacing (~0.05em)
- **Shape language**: beveled rectangles, chamfered corners (45° cuts, not rounded), diagonal cut corners on panels
- **Animation character**: smooth data transitions (120–200ms), slow idle pulses (2s period), crisp fast responses (80ms)
- **Feel**: calm, intelligent, military precision — the interface of a species that has mastered space travel

### Dark Tech (Mass Effect / XCOM 2)

- **Dominant palette**: near-black `#080C10`, orange `#FF6600`, white `#FFFFFF`
- **Textures**: carbon fiber / brushed metal at 8% opacity
- **Typography**: bold condensed sans, high contrast
- **Shape language**: hard rectangles, industrial angles, thick borders (2–3px)
- **Animation**: fast hard cuts, brief flash effects on state changes

### Imperial Military (StarCraft Terran / BSG)

- **Dominant palette**: gunmetal `#2A3040`, amber `#D4A020`, red `#CC2200`
- **Textures**: riveted metal plate, worn paint, stencil lettering
- **Shape language**: rectangular, utilitarian, bolt-head details as decorative elements
- **Animation**: CRT flicker on panel open, static bursts on alert events

### Organic Alien (Zerg / Alien Isolation)

- **Dominant palette**: dark purple `#1A0828`, acid green `#44FF22`, bioluminescent blue
- **Textures**: chitin surface, membrane transparency with subsurface scatter
- **Shape language**: irregular curves, asymmetric growths, no straight lines
- **Animation**: breathing pulse (slow sine on scale), wet surface shimmer (specular ripple)

---

## Section 9: Specific Layout Proposal for Homeworld-Style Bevy Game

### Layout Diagram

```
┌─────────────────────────────────────────────────────┐
│  RESOURCE BAR          [ALERTS]      FLEET STATUS   │  4%
├──────────┬──────────────────────────────┬───────────┤
│          │                              │  MINIMAP  │
│  EVENT   │                              │  [15%w]   │
│  FEED    │     3D WORLD VIEWPORT        ├───────────┤
│  [16%w]  │         (sacred)             │  FLEET    │
│          │                              │  TREE     │
│  notifs  │                              │  [15%w]   │
│  alerts  │                              │           │
│  combat  │                              │           │
│  log     │                              │           │
├──────────┴──┬──────────────────────┬───┴───────────┤
│  SELECTION  │   COMMAND CONSOLE    │  BUILD QUEUE  │
│  PORTRAIT   │   [tabs: Orders /    │  + RESEARCH   │
│  + STATS    │    Build / Intel]    │  PROGRESS     │
│  [20%w]     │       [45%w]         │  [25%w]       │
└─────────────┴──────────────────────┴───────────────┘
                                              18-22%
```

### Zone Specifications

#### Top Resource Bar (100% width, 4% height)
- **Left-aligned**: resources — minerals / energy / population / credits, each as `icon + monospace number`
- **Center**: alert area — flashing text for critical events (under attack, starvation, etc.)
- **Right-aligned**: fleet status summary — total ships / losses this session / kills this session
- Always visible, never collapsed

#### Left Event Feed (16% width, 74% height between top bar and bottom console)
- Scrolling log of: combat events (unit destroyed, damage taken), build completions, research unlocks, diplomatic messages
- Color-coded by severity: grey for info, amber for warning, red for critical, green for positive
- Newest entries at bottom; auto-scrolls; player can pause auto-scroll by scrolling up
- **Collapsible**: collapses to a narrow icon rail (32px wide) with unread-count badge

#### 3D World Viewport (center, sacred)
- No permanent HUD elements overlap this zone
- Permitted overlays (world-space, not screen-space): unit health bars, selection rings, waypoint lines, formation indicators, damage numbers
- These are rendered as Bevy world-space UI elements, not egui panels
- Visual Effects and Polish:
  - Implement volumetric lighting for dramatic space atmosphere
  - Create holographic shader effects for all UI elements (configurable ON/OFF)
  - Add screen-space reflections and bloom for cinematic quality
  - Build particle systems for UI interactions and transitions
  - Design audio-reactive visual feedback for button presses
  - Implement camera shake and dramatic angles for events

#### Top-Right Minimap (15% width, ~30% height of right panel)
- Octagonal masked rectangle (octagonal clip mask over rectangular render)
- Displays: terrain (colored tiles), friendly units (cyan dots), enemy units (red dots), neutral (grey dots)
- Fog of war: dark overlay, 40% opacity
- Camera frustum: white rectangle outline
- Threat pings: red circles expanding from attacked unit positions, 1 Hz pulse
- Click to move camera; drag to pan camera
- Holographic Minimap system:
- Holographic Minimap System
  - Create Prometheus-style topological hologram display
  - Implement real-time terrain mesh generation with scan-line effects
  - Build unit position markers with faction color coding
  - Add ping system and tactical markers
  - Create animated scanning effects and grid overlay
  - Implement minimap camera controls and viewport indication

#### Right Fleet Tree (15% width, ~44% height of right panel, below minimap)
- Hierarchical collapsible list: Fleet → Wing → Individual Ship
- Per-entry: unit icon, name, health bar (thin horizontal), status icon
- Click to select unit; double-click to focus camera on unit
- Sins of a Solar Empire style — the primary navigation tool for large fleets
- Collapsible to icon rail

#### Bottom-Left Selection Panel (20% width, 18–22% height)
- **Large unit portrait**: 3D rendered ship model (TextureHandle updated on selection change)
- **Unit name + class**: `IBM Plex Sans 16px`, mixed case
- **Health ring**: circular progress indicator, color-coded green → amber → red
- **Stat grid**: attack / defense / speed / range in a 2×2 grid with icons
- **Special ability icons**: 4–6 ability slots with cooldown overlays and hotkey labels

#### Bottom-Center Command Console (45% width, 18–22% height)
- **Tab bar at top of console**: `[O]rders` | `[B]uild` | `[I]ntel` with keyboard shortcuts shown
- Active tab: cyan underline; inactive: 50% opacity
- **Orders tab**: 5×3 command grid (15 slots), contextual to current selection; hotkey letter on each button
- **Build tab**: unit/structure production queue with progress bars, cancel buttons, queued items as icon strip
- **Intel tab**: full data sheet for selected unit — weapon specs, movement stats, special ability descriptions

#### Bottom-Right Build Queue + Research (25% width, 18–22% height)
- **Build queue**: active items with countdown timers + cancel (×) buttons; up to 5 queued items shown as icon strip below
- **Research**: active research item with progress bar and ETA; queued research as icon strip
- **Compact layout**: two sections stacked vertically within the panel

### Interaction Model / Visibility Tiers

| Tier               | Trigger             | Elements Visible                                                                                   |
| ------------------ | ------------------- | -------------------------------------------------------------------------------------------------- |
| Tier 1 — Always    | —                   | Resource bar, minimap, event feed (icon rail or full), bottom console chrome                       |
| Tier 2 — On Select | Unit/fleet selected | Full selection panel populates, command grid fills, build queue shows if relevant                  |
| Tier 3 — Modal     | Player opens screen | Full tech tree, diplomacy screen, settings — full-screen overlay with 70% dimmed background behind |

Tier 3 overlays use solid opaque panels (rgba 0.95+) because they require full reading focus. The glassmorphism rule applies only to Tier 1 and Tier 2 elements.
