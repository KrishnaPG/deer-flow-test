---
name: ui-theme
description: Craft distinctive, non-generic aesthetic UI themes with intentional typography, color, motion, and atmosphere
license: MIT
compatibility: opencode
metadata:
  audience: developers, designers, AI agents
  workflow: design-system, frontend-development
  version: "1.0"
---

## What I Do

I help you create **distinctive, high-quality UI themes** that avoid generic "AI slop" aesthetics. I guide you through:

- **Typography selection**: Choosing distinctive fonts that signal quality and uniqueness
- **Color & theme design**: Building cohesive, intentional color palettes with depth
- **Motion & micro-interactions**: Adding purposeful animations for key moments
- **Atmospheric design**: Creating layered backgrounds and environments
- **Holistic aesthetics**: Ensuring every design choice connects to a unified vision

## When to Use Me

Use this skill when you are:
- Designing a new UI theme or visual system
- Creating distinctive component aesthetics
- Establishing a frontend design language
- Building atmosphere and mood into interfaces
- Avoiding generic, predictable design patterns

## Core Principles

### 1. Typography: Instantly Signals Quality

Your font choice is the first and strongest signal of design intentionality.

**AVOID these generic fonts:**
- Inter, Roboto, Open Sans, Lato, Arial
- Default system fonts
- Overused choices (Space Grotesk, despite being beautiful)

**USE distinctive alternatives** based on your aesthetic:

| Aesthetic | Font Examples |
|-----------|---------------|
| Code/Technical | JetBrains Mono, Fira Code, IBM Plex Mono |
| Editorial/Publishing | Playfair Display, Crimson Pro, Fraunces |
| Startup/Modern | Clash Display, Satoshi, Cabinet Grotesk |
| Technical/Scientific | IBM Plex Sans/Serif family, Source Sans 3 |
| Distinctive/Editorial | Bricolage Grotesque, Obviously, Newsreader |

**Font Pairing Principle:**
- **High contrast wins**: Display + monospace, serif + geometric sans, or variable fonts across extreme weights
- **Use extremes**: 100/200 weight vs 800/900 (not 400 vs 600)
- **Size jumps of 3x+**: Not 1.5x increments
- **Pick ONE distinctive font decisively**: Make it your signature. Load from Google Fonts.

**Process:**
1. State your chosen font **before coding**
2. Justify why it matches your aesthetic
3. Plan weight and size scale
4. Load from Google Fonts

### 2. Color & Theme: Commit to Cohesion

Build intentional, coordinated color systems rather than generic palettes.

**Strategy:**
- **Dominant color + sharp accents**: Beats timid, evenly-distributed colors
- **Draw inspiration from**:
  - IDE themes (VS Code, JetBrains, iTerm2, etc.)
  - Cultural aesthetics and historical periods
  - Nature: seasons, times of day, environments
  - Artistic movements and design eras

**Technical Implementation:**
- Use CSS variables for consistency: `--primary`, `--accent`, `--bg-depth-1`, etc.
- Leverage **OKLCH color space** for perceptually uniform colors
- Reference **ColorBrewer** for tested, harmonious palettes
- Consider contrast ratio requirements (WCAG AA/AAA)

**Avoid:**
- Purple gradients on white (clichéd)
- Evenly-distributed, safe color choices
- Colors without contextual meaning

### 3. Motion: Orchestrate, Don't Scatter

Use animations purposefully to delight without distraction.

**High-Impact Approach:**
- **One well-orchestrated page load** beats scattered micro-interactions
- Use **staggered reveals** with `animation-delay` for entrance
- Focus animations on:
  - Initial entry
  - State transitions (success, error, loading)
  - User interactions (hover, focus, clicks)
  - Navigation changes

**Technical Tips:**
- Use `transition` for simple state changes (color, opacity, scale)
- Use `@keyframes` + `animation` for complex choreography
- Prefer GPU-accelerated properties: `transform`, `opacity`
- Avoid animating `width`, `height`, or `left`/`top`
- Duration: 300-500ms for micro-interactions, 800-1200ms for page transitions

### 4. Backgrounds: Create Atmosphere & Depth

Transform backgrounds from static to dynamic storytelling.

**Techniques:**
- **Layered CSS gradients**: Multiple color stops, radial or linear
- **Geometric patterns**: Grid, diagonal lines, waves, noise
- **Contextual effects**: Match overall aesthetic (see "Lively" section)
- **Depth layers**: Subtle shadow or gradient changes across Z-axis
- **Motion possibilities**: Subtle, slow animations on backgrounds

**Avoid:**
- Flat, solid colors (unless deliberately minimal)
- Generic patterns (avoid overused textures)
- Backgrounds that fight content for attention

### 5. Lively: Purpose and Connection

Create designs where everything relates to something larger.

**Design through Context:**
Every element should connect to a unified aesthetic vision. Choose from these dimensions and combine them:

| Dimension | Inspiration Examples |
|-----------|----------------------|
| **Environment** | Militaristic bunkers, space orbits, factories, cities, villages, research labs, forests, archives |
| **Civilization/Era** | Bronze age, medieval, pre-historic, apocalyptic, far future, sea-faring, deep space, steampunk |
| **Location/Place** | Distant meadows, mountain ranges, river sides, tropical jungles, deserts, town squares, markets, living rooms, laboratories |
| **Time of Day** | Dawn, dusk, noon, night, twilight, golden hour |
| **Weather/Season** | Winter, summer, autumn, spring, rain, snow, wind, drought, storms |
| **Artistic Tone** | Cartoon, pastel, pencil drawing, Ghibli-inspired, noir, retro-futurism, brutalism, minimalism |

**Design Process:**

1. **Pick 2-3 dimensions** that complement each other
2. **Establish visual language**: Colors, typography, texture, patterns, and UI shapes that match
3. **Connect everything**: Colors tie to environment/season, typography matches tone/era, motion feels natural to place
4. **Tell a story**: Users should sense the intentional aesthetic, even subconsciously.
5. **Guide the user with contrast**: Use contrast like a film director uses light, weather, stillness, and crowding to decide what the viewer feels first, last, and most intensely (see "Cinematic Contrast" below)

### Cinematic Contrast

Contrast is not decoration. It is staging. It tells the eye where to land, what to fear, what to trust, and what to remember.

Think cinematically: a quiet figure against a storm, one banner moving in a field of stillness, a bright doorway at the end of a dark corridor, a lone rider crossing a wide empty plain. Great contrast creates emotional gravity before the user reads a word.

Inspired by the discipline of classic cinema: use contrast to create tension, rhythm, scale, and inevitability. Do not just make one thing brighter. Make that thing feel destined.

**Creative forms of contrast:**

| Contrast Form | What it does | How to use it in UI |
|---------------|--------------|---------------------|
| **Stillness vs. Movement** | Makes the moving element feel important or alive | Keep most of the interface calm, then animate only the critical path: a CTA, active step, notification, or selected card |
| **Emptiness vs. Density** | Creates pressure, solitude, significance, or relief | Surround a primary message with generous empty space, then place dense metadata or controls elsewhere so the user knows what matters |
| **Order vs. Chaos** | Signals safety against danger, system against entropy | Use rigid grids and disciplined alignment for core workflows; introduce rough texture, asymmetry, or broken rhythm only where tension or energy is useful; or Vice-versa |
| **Light vs. Weather** | Gives atmosphere and urgency | Use soft surfaces for the baseline, then add haze, rain-like linework, glow, dust, fog, or shadow only around major moments or sections |
| **Monument vs. Detail** | Establishes hierarchy through scale and permanence | Make one element feel architectural: a massive heading, oversized number, or dominant panel; let secondary details feel fine-grained and human |
| **Silence vs. Ornament** | Prevents visual fatigue and makes ornament meaningful | Keep most surfaces restrained; reserve pattern, illustration, texture, or flourish for the moment that deserves ceremony |
| **Near vs. Far** | Creates longing, orientation, and narrative depth | Sharpen the current focal plane and soften distant layers with blur, fade, reduced contrast, smaller scale, or atmospheric color shifts |
| **Ritual vs. Interruption** | Makes state changes feel consequential | Build a steady interaction rhythm, then break it deliberately for warnings, confirmations, unlocks, or irreversible actions |
| **Human vs. Machine** | Creates character and memorable tension | Pair precise grids and technical UI with warm textures, imperfect illustrations, serif typography, or tactile color accents |
| **Seasonal Contrast** | Grounds the interface in time and mood | Contrast winter austerity with hearth-like warmth, noon clarity with dusk mystery, dry stone with wet reflections, harvest richness with archival restraint |

**What contrast is for:**

- **Attention**: goes first to the element that breaks the established visual climate
- **Meaning**: Contrast tells users whether something is sacred, dangerous, alive, dormant, temporary, or structural
- **Pacing**: Alternating calm and intensity creates rhythm; everything loud means nothing is loud
- **Memory**: Users remember the moment where the interface changed weather, scale, or silence
- **Hierarchy**: Contrast determines what feels primary, ceremonial, supporting, or background without relying on labels alone
- **Emotion**: Contrast turns an interface from arranged components into a scene with tension and release

**How to compose contrast well:**

1. **Establish a baseline climate**: Decide what "normal" feels like first - quiet, foggy, dry, ceremonial, industrial, pastoral, severe
2. **Choose the interruption**: Define what deserves contrast - the CTA, the key metric, the active step, the warning, the hero statement
3. **Use one dominant break per scene**: Each section should have a primary contrast event; too many and the whole thing flattens
4. **Stack contrast with intent**: Combine two or three forms when something truly matters - e.g. stillness + accent color + elevation
5. **Let contrast fit the world**: In a medieval theme, contrast may come from torchlight and parchment emptiness; in a research lab, from sterile grids and sudden signal glows
6. **Preserve aftermath**: After a high-contrast moment, let nearby elements relax so the important thing keeps its force

**World-building examples:**

- **Medieval marketplace, dusk, autumn**: Use crowded texture at the edges but keep the trade action isolated in a pool of lantern light; primary actions feel like lit stalls in a dark square
- **Research lab, futuristic, dawn**: Keep the UI clinically still and pale, then use one sharp spectral accent and subtle pulse only for live systems or primary actions
- **Deep forest, twilight, pastoral**: Let most surfaces feel soft, mossy, and receded; bring focus forward with a single clear path, firefly-like glow, or crisp hand-lettered callout
- **Industrial archive, winter morning**: Use severe alignment, cold paper tones, and restrained typography; key records emerge through warmth, density, and a slight rise in contrast like a document pulled into light
- **Orbital navigation deck, storm alert**: Maintain a quiet starfield and measured interface rhythm, then interrupt with directional sweeps, warning bands, and sudden illumination to create urgency

## Workflow: How Agents Should Use This Skill

### Step 1: Define the Aesthetic Vision
Before writing any code, establish:
- Primary aesthetic dimensions (environment, era, tone)
- Target emotion (playful, serious, calming, energetic)
- Distinctive font choice + justification
- 3-5 core colors with OKLCH values

### Step 2: Build the Design System
- Create CSS custom properties for consistent application
- Define type scale, spacing scale, animation timing
- Document color roles (primary, accent, semantic, interactive)

### Step 3: Implement Holistically
- Typography first (sets tone for everything)
- Backgrounds second (creates atmosphere)
- Color palette third (ties everything together)
- Motion last (adds life to completed design)

### Step 4: Avoid Generic Traps
- **Never default to**: Inter, system fonts, purple/white combos
- **Always question**: "Would an LLM make this choice?" If yes, reconsider
- **Seek divergence**: What's unusual but still functional for your context?

## Technical Resources

**Font Resources:**
- Google Fonts: https://fonts.google.com
- Fontshare: https://www.fontshare.com (free fonts)
- Adobe Fonts: https://fonts.adobe.com

**Color Tools:**
- ColorBrewer: https://colorbrewer2.org
- OKLCH Color Picker: https://oklch.com
- Contrast Checker: https://www.tpgi.com/color-contrast-checker/

**Animation Tools:**
- Easings: https://easings.net
- Keyframe Generator: https://keyframes.app

## Examples of Good vs. Problematic Choices

### ❌ Problematic (Generic AI Default)
- Font: Inter + system colors
- Color: Purple gradient on white
- Background: Solid, flat color
- Motion: Scattered, fast micro-interactions
- Feeling: Unsafe, "AI-generated"

### ✅ Strong (Intentional Design)
- Font: Playfair Display (display) + JetBrains Mono (code) = Editorial + Technical fusion
- Color: Deep emerald (#1a4d3e), gold accent (#d4a574), warm ivory background
- Background: Layered gradient with subtle noise, evokes aged parchment + modern tech
- Motion: Staggered entrance animations on page load, subtle hover effects
- Feeling: Established, trustworthy, intentional

## Anti-Patterns to Avoid

1. **Font Convergence**: Space Grotesk/Satoshi everywhere (even though they're good)
2. **Color Timidity**: Safe, evenly-distributed colors that don't commit to a vision
3. **Motion Scatter**: Micro-interactions everywhere instead of orchestrated moments
4. **Context-Free Design**: Elements that don't connect to a larger aesthetic story
5. **Generic Backgrounds**: White, light gray, or flat colors without atmosphere
6. **Overused Combos**: Purple + pink, cyan + magenta, teal gradients, Purple & Orange, Red & Green, Orange & Yellow, Pink & Black, Neon-on-Neon

## Success Criteria

A theme is successful when:
- ✅ Font choice immediately communicates intentionality
- ✅ Color palette feels cohesive and purposeful
- ✅ Atmosphere/background creates mood
- ✅ Motion enhances (not distracts from) experience
- ✅ Design feels custom, not generic
- ✅ All choices connect to a unified aesthetic vision
- ✅ Users sense the intentional design without effort
