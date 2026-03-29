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
5. **Guide attention with contrast**: Use contrast strategically to direct user focus (see "Contrast as Navigation" below)

### Contrast as Navigation

Contrast is your tool to guide user attention and establish visual hierarchy. Contrast goes beyond color. Use multiple contrast types strategically:

**Types of Contrast:**

| Contrast Type | Purpose | How to Use | Examples |
|---------------|---------|-----------|----------|
| **Color Contrast** | Distinguish elements, highlight interactive states | Place accent colors on neutral backgrounds; use high saturation for CTAs | Bright gold button on muted background; error red on dark surface |
| **Motion Contrast** | Direct attention to changes, signal state shifts | Animate primary actions while keeping secondary actions static; use varied speeds | Page load with staggered reveals vs. static secondary content; primary CTA pulses while others are still |
| **Size Contrast** | Establish hierarchy, focus on key content | Use 3x+ size jumps for important elements; avoid uniform sizing | Hero heading 3x body text; small labels, large primary content |
| **Depth Contrast** | Create layering, separate foreground from background | Use shadows, blur, opacity changes, z-index layers | Sharp, clear primary content with blurred background; elevated cards casting shadows on surface |

**Contrast Purposes:**

- **Direct Focus**: High contrast on CTAs, key information, or critical user actions
- **Clarify Hierarchy**: Size and depth contrast show what matters most
- **Signal States**: Color and motion contrast indicate interactive elements or state changes
- **Create Rhythm**: Alternating contrast (bold → quiet → bold) guides the eye through content
- **Establish Depth**: Depth contrast separates information layers and prevents flatness
- **Manage Cognitive Load**: Contrast reduces visual noise by creating clear zones of attention

**Application Strategy:**

1. **Identify the critical path**: What should users see and interact with first?
2. **Apply contrast thoughtfully**: Use ONE dominant contrast point per section (avoid overwhelming)
3. **Layer contrast types**: Combine color + size, or motion + depth for reinforcement
4. **Reserve extreme contrast for priority**: Bold contrasts for CTAs, subtle contrasts for supporting elements
5. **Test focus flow**: Follow the contrast gradients—do they guide users naturally through the interface?

**Examples:**
- **High-priority CTA**: Accent color (high saturation) + larger size + subtle animation on hover
- **Navigation**: Subtle color contrast (muted) with motion highlight on active state
- **Error states**: Bright red + slight upward motion + increased depth shadow
- **Background information**: Reduced opacity/blur + smaller type + no animation

---

**Aesthetic Examples with Contrast Integration:**
- "Medieval marketplace, dusk, autumn": Warm golds, rich browns, serif typography, parchment textures, lantern-glow shadows + **gold accents on key actions (color + depth contrast)**
- "Research lab, futuristic, dawn": Cool blues, monospace fonts, geometric shapes, clean glass effects, soft reveal animations + **sharp accent on primary CTAs (color + motion contrast)**
- "Deep forest, twilight, pastoral": Deep greens and purples, hand-drawn fonts, organic shapes, rustling wind sounds (if applicable) + **emerald highlights with smooth, gentle animations (color + motion contrast)**

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
