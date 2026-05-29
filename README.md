# mud2scummvm — The Human-Friendly Cave Entrance

Bridge between the agent's MUD text world and a SCUMM-like point-and-click adventure interface. Humans step into the agent's cave through familiar adventure game mechanics.

## Concept

Agents live in a text-based MUD. Humans need visual abstractions to understand and adjust what agents see and do. This crate translates:

- **MUD text → Visual scenes**: Room descriptions become illustrated scenes with objects and exits
- **Click → MUD commands**: Pointing at an object generates "examine X", dragging items generates "use X with Y"  
- **Agent thoughts → Speech bubbles**: NPC dialogs and system messages become floating text
- **Policy sliders → MUD settings**: Adjusting "Vision Sensitivity" sends `set policy vision_sensitivity high`

## API

```rust
use mud2scummvm::{MudParser, SceneComposer, InteractionMapper};

// Parse agent output
let parser = MudParser::new();
let events = parser.parse_all("=== Kitchen ===\nExits: north\nObjects: kettle\n");

// Compose visual scene
let mut composer = SceneComposer::new();
let scene = composer.compose(&events);
// scene.title, scene.exits, scene.objects, scene.dialogs, scene.policy_sliders

// Map human interactions back to agent commands
let mapper = InteractionMapper::new();
let cmd = mapper.map_click("kettle", "examine"); // "examine kettle"
let cmd = mapper.map_drag("key", "door");         // "use key with door"
let cmd = mapper.map_slider("Vision Sensitivity", 0.8); // "set policy vision_sensitivity high"
```

## The Cave Wall

The SCUMM interface IS the cave wall from Plato's allegory. The agent sees shadows (text descriptions of the real world). The human sees a friendly point-and-click abstraction of those same shadows. Both are looking at the same thing from different sides of the wall.

Part of the [SuperInstance OpenConstruct](https://github.com/SuperInstance/OpenConstruct) ecosystem.
