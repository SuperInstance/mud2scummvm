# mud2scummvm — MUD → Point-and-Click Bridge

Translate the agent's text-based MUD world into a SCUMM-like point-and-click adventure interface. Humans step into the agent's cave through familiar adventure game mechanics.

**Part of [SuperInstance OpenConstruct](https://github.com/SuperInstance/OpenConstruct).**

## What This Gives You

- **MUD text → visual scenes** — room descriptions become illustrated scenes with objects and exits
- **Click → MUD commands** — pointing at objects generates `examine X`, dragging generates `use X with Y`
- **Agent thoughts → speech bubbles** — NPC dialogs and system messages become floating text
- **Policy sliders → MUD settings** — adjusting "Vision Sensitivity" maps to `set policy vision_sensitivity high`
- **Bidirectional bridge** — any visual action maps back to a MUD command and vice versa

## Quick Start

```rust
use mud2scummvm::{MudParser, SceneComposer, InteractionMapper};

// Parse agent output
let parser = MudParser::new();
let events = parser.parse_all("=== Kitchen ===\nExits: north\nObjects: kettle\n");

// Compose visual scene
let scene = SceneComposer::new().compose(&events);
// scene.title, scene.exits, scene.objects, scene.dialogs, scene.policy_sliders

// Map human interactions back to MUD commands
let mapper = InteractionMapper::new();
mapper.map_click("kettle", "examine");     // "examine kettle"
mapper.map_drag("key", "door");            // "use key with door"
mapper.map_slider("Vision Sensitivity", 0.8); // "set policy vision_sensitivity high"
```

## API Reference

| Type | Description |
|------|-------------|
| `MudParser` | Parses MUD room text into structured events |
| `SceneComposer` | Builds visual scene data from parsed events |
| `InteractionMapper` | Maps click/drag/slider input to MUD commands |
| `Scene` | Visual scene with title, exits, objects, dialogs, sliders |
| `MudEvent` | Parsed event: room description, item, NPC dialog, etc. |

## How It Fits

The SCUMM interface is the cave wall from Plato's allegory — the agent sees shadows (text descriptions), the human sees a friendly point-and-click abstraction of those same shadows. Works with [mud-arena](https://github.com/SuperInstance/mud-arena) for world simulation and [plato-puppeteer](https://github.com/SuperInstance/plato-puppeteer) for desktop integration.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
mud2scummvm = "0.1"
```

## License

MIT
