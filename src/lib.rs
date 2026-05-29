//! mud2scummvm — Bridge between agent MUD world and SCUMM-like point-and-click UI
//!
//! Humans step into the agent's cave through a point-and-click adventure game.
//! The MUD text world the agent lives in gets projected as a SCUMM-like graphical
//! interface where humans can see rooms as scenes, click objects, and adjust policies.

use std::collections::HashMap;

// ─── MUD Parsing ───────────────────────────────────────────────

/// A parsed MUD event from agent text output.
#[derive(Debug, Clone, PartialEq)]
pub enum MudEvent {
    RoomDescription {
        title: String,
        description: String,
        exits: Vec<String>,
        objects: Vec<String>,
    },
    ObjectDescription {
        name: String,
        description: String,
        actions: Vec<String>,
    },
    NpcDialog {
        speaker: String,
        text: String,
        mood: Option<String>,
    },
    ActionResult {
        command: String,
        result: String,
        success: bool,
    },
    TickReceived {
        from: String,
        topic: String,
        body: String,
    },
    StatusUpdate {
        module: String,
        status: String,
    },
}

/// Parser for MUD text output into structured events.
pub struct MudParser;

impl MudParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a MUD text line into a structured event.
    pub fn parse(&self, text: &str) -> Option<MudEvent> {
        let text = text.trim();
        if text.starts_with("=== ") && text.ends_with(" ===") {
            let title = text.trim_start_matches('=').trim_end_matches('=').trim();
            Some(MudEvent::RoomDescription {
                title: title.to_string(),
                description: String::new(),
                exits: Vec::new(),
                objects: Vec::new(),
            })
        } else if text.starts_with("Exits: ") {
            let exits_str = text.trim_start_matches("Exits: ");
            let exits: Vec<String> = exits_str.split(", ").map(|s| s.to_string()).collect();
            Some(MudEvent::RoomDescription {
                title: String::new(),
                description: String::new(),
                exits,
                objects: Vec::new(),
            })
        } else if text.starts_with("Objects: ") {
            let obj_str = text.trim_start_matches("Objects: ");
            let objects: Vec<String> = if obj_str == "none" {
                Vec::new()
            } else {
                obj_str.split(", ").map(|s| s.to_string()).collect()
            };
            Some(MudEvent::RoomDescription {
                title: String::new(),
                description: String::new(),
                exits: Vec::new(),
                objects,
            })
        } else if text.starts_with('"') {
            // NPC dialog: "Hello there" — Guard (friendly)
            if let Some((dialog, rest)) = text.split_once("\" — ") {
                let dialog_text = dialog.trim_start_matches('"');
                let parts: Vec<&str> = rest.splitn(2, " (").collect();
                let speaker = parts[0].to_string();
                let mood = parts.get(1).map(|s| s.trim_end_matches(')').to_string());
                Some(MudEvent::NpcDialog {
                    speaker,
                    text: dialog_text.to_string(),
                    mood,
                })
            } else {
                None
            }
        } else if text.starts_with("You ") {
            // Action result: "You examine the crystal ball. It shimmers with spectral data."
            let parts: Vec<&str> = text.splitn(2, ". ").collect();
            if parts.len() == 2 {
                Some(MudEvent::ActionResult {
                    command: parts[0].trim_start_matches("You ").to_string(),
                    result: parts[1].to_string(),
                    success: true,
                })
            } else {
                Some(MudEvent::ActionResult {
                    command: parts[0].trim_start_matches("You ").to_string(),
                    result: String::new(),
                    success: true,
                })
            }
        } else if text.starts_with("TICK from ") {
            // Tick: "TICK from jetson-hall [vision]: person detected at front door"
            let rest = text.trim_start_matches("TICK from ");
            let parts: Vec<&str> = rest.splitn(2, " [").collect();
            if parts.len() == 2 {
                let from = parts[0].to_string();
                let rest = parts[1];
                let topic_end = rest.find("]: ").unwrap_or(rest.len());
                let topic = rest[..topic_end].to_string();
                let body = rest.get(topic_end + 3..).unwrap_or("").to_string();
                Some(MudEvent::TickReceived { from, topic, body })
            } else {
                None
            }
        } else if text.contains(": ") && !text.starts_with(' ') {
            let parts: Vec<&str> = text.splitn(2, ": ").collect();
            Some(MudEvent::ObjectDescription {
                name: parts[0].to_string(),
                description: parts.get(1).unwrap_or(&"").to_string(),
                actions: Vec::new(),
            })
        } else {
            None
        }
    }

    /// Parse multiple lines into events.
    pub fn parse_all(&self, text: &str) -> Vec<MudEvent> {
        text.lines().filter_map(|line| self.parse(line)).collect()
    }
}

// ─── Scene Composition ─────────────────────────────────────────

/// A visual scene composed from parsed MUD data.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub title: String,
    pub description: String,
    pub exits: Vec<SceneExit>,
    pub objects: Vec<SceneObject>,
    pub characters: Vec<SceneCharacter>,
    pub dialogs: Vec<DialogBubble>,
    pub policy_sliders: Vec<PolicySlider>,
}

/// An exit in the scene (door, passage, corridor).
#[derive(Debug, Clone, PartialEq)]
pub struct SceneExit {
    pub label: String,
    pub direction: String,
    pub highlighted: bool,
}

/// An object in the scene that can be interacted with.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneObject {
    pub name: String,
    pub description: String,
    pub position: (f64, f64), // relative x,y in [0,1]
    pub interactable: bool,
    pub actions: Vec<String>,
}

/// A character (NPC or agent) in the scene.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneCharacter {
    pub name: String,
    pub mood: Option<String>,
    pub position: (f64, f64),
}

/// A speech/thought bubble.
#[derive(Debug, Clone, PartialEq)]
pub struct DialogBubble {
    pub speaker: String,
    pub text: String,
    pub bubble_type: BubbleType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BubbleType {
    Speech,
    Thought,
    System,
}

/// A policy slider for adjusting agent behavior.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicySlider {
    pub name: String,
    pub value: f64,    // [0.0, 1.0]
    pub min_label: String,
    pub max_label: String,
}

/// Composes visual scenes from parsed MUD data.
pub struct SceneComposer {
    object_positions: HashMap<String, (f64, f64)>,
    next_position: (f64, f64),
}

impl SceneComposer {
    pub fn new() -> Self {
        Self {
            object_positions: HashMap::new(),
            next_position: (0.2, 0.3),
        }
    }

    pub fn compose(&mut self, events: &[MudEvent]) -> Scene {
        let mut scene = Scene {
            title: String::new(),
            description: String::new(),
            exits: Vec::new(),
            objects: Vec::new(),
            characters: Vec::new(),
            dialogs: Vec::new(),
            policy_sliders: Vec::new(),
        };

        for event in events {
            match event {
                MudEvent::RoomDescription { title, exits, objects, .. } => {
                    if !title.is_empty() { scene.title = title.clone(); }
                    for exit in exits {
                        scene.exits.push(SceneExit {
                            label: exit.clone(),
                            direction: exit.clone(),
                            highlighted: false,
                        });
                    }
                    for obj in objects {
                        let pos = self.get_or_assign_position(obj);
                        scene.objects.push(SceneObject {
                            name: obj.clone(),
                            description: String::new(),
                            position: pos,
                            interactable: true,
                            actions: vec!["examine".to_string(), "use".to_string()],
                        });
                    }
                }
                MudEvent::ObjectDescription { name, description, actions } => {
                    if let Some(obj) = scene.objects.iter_mut().find(|o| o.name == *name) {
                        obj.description = description.clone();
                        if !actions.is_empty() { obj.actions = actions.clone(); }
                    } else {
                        let pos = self.get_or_assign_position(name);
                        scene.objects.push(SceneObject {
                            name: name.clone(),
                            description: description.clone(),
                            position: pos,
                            interactable: true,
                            actions: if actions.is_empty() {
                                vec!["examine".to_string()]
                            } else {
                                actions.clone()
                            },
                        });
                    }
                }
                MudEvent::NpcDialog { speaker, text, mood } => {
                    scene.characters.push(SceneCharacter {
                        name: speaker.clone(),
                        mood: mood.clone(),
                        position: self.get_or_assign_position(speaker),
                    });
                    scene.dialogs.push(DialogBubble {
                        speaker: speaker.clone(),
                        text: text.clone(),
                        bubble_type: BubbleType::Speech,
                    });
                }
                MudEvent::ActionResult { result, .. } => {
                    scene.dialogs.push(DialogBubble {
                        speaker: "system".to_string(),
                        text: result.clone(),
                        bubble_type: BubbleType::System,
                    });
                }
                _ => {}
            }
        }

        // Default policy sliders
        scene.policy_sliders = vec![
            PolicySlider { name: "Vision Sensitivity".into(), value: 0.7, min_label: "Low".into(), max_label: "High".into() },
            PolicySlider { name: "Action Caution".into(), value: 0.5, min_label: "Bold".into(), max_label: "Careful".into() },
            PolicySlider { name: "Tick Frequency".into(), value: 0.6, min_label: "Rare".into(), max_label: "Frequent".into() },
            PolicySlider { name: "Verbosity".into(), value: 0.4, min_label: "Terse".into(), max_label: "Detailed".into() },
        ];

        scene
    }

    fn get_or_assign_position(&mut self, name: &str) -> (f64, f64) {
        if let Some(&pos) = self.object_positions.get(name) {
            pos
        } else {
            let pos = self.next_position;
            self.next_position.0 += 0.2;
            if self.next_position.0 > 0.8 {
                self.next_position.0 = 0.2;
                self.next_position.1 += 0.2;
            }
            self.object_positions.insert(name.to_string(), pos);
            pos
        }
    }
}

// ─── Interaction Mapping ───────────────────────────────────────

/// Maps point-and-click actions back to MUD commands.
pub struct InteractionMapper;

impl InteractionMapper {
    pub fn new() -> Self { Self }

    /// Map a click on an object to a MUD command.
    pub fn map_click(&self, object: &str, action: &str) -> String {
        match action {
            "examine" => format!("examine {}", object),
            "use" => format!("use {}", object),
            "take" => format!("take {}", object),
            "talk" => format!("talk to {}", object),
            _ => format!("{} {}", action, object),
        }
    }

    /// Map a drag action (drag item onto object) to a MUD command.
    pub fn map_drag(&self, item: &str, target: &str) -> String {
        format!("use {} with {}", item, target)
    }

    /// Map an exit click to a MUD movement command.
    pub fn map_exit(&self, direction: &str) -> String {
        format!("go {}", direction)
    }

    /// Map a policy slider adjustment.
    pub fn map_slider(&self, slider: &str, value: f64) -> String {
        let level = if value < 0.33 { "low" } else if value < 0.66 { "medium" } else { "high" };
        format!("set policy {} {}", slider.to_lowercase().replace(' ', "_"), level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Parser Tests ─────────────────────────────

    #[test]
    fn parse_room_title() {
        let parser = MudParser::new();
        let event = parser.parse("=== Kitchen ===").unwrap();
        assert!(matches!(event, MudEvent::RoomDescription { title, .. } if title == "Kitchen"));
    }

    #[test]
    fn parse_exits() {
        let parser = MudParser::new();
        let event = parser.parse("Exits: north, south, corridor").unwrap();
        assert!(matches!(event, MudEvent::RoomDescription { exits, .. }
            if exits == vec!["north", "south", "corridor"]));
    }

    #[test]
    fn parse_objects() {
        let parser = MudParser::new();
        let event = parser.parse("Objects: crystal_ball, key, scroll").unwrap();
        assert!(matches!(event, MudEvent::RoomDescription { objects, .. }
            if objects == vec!["crystal_ball", "key", "scroll"]));
    }

    #[test]
    fn parse_no_objects() {
        let parser = MudParser::new();
        let event = parser.parse("Objects: none").unwrap();
        assert!(matches!(event, MudEvent::RoomDescription { objects, .. } if objects.is_empty()));
    }

    #[test]
    fn parse_npc_dialog() {
        let parser = MudParser::new();
        let event = parser.parse("\"Welcome, agent.\" — Oracle (wise)").unwrap();
        assert!(matches!(event, MudEvent::NpcDialog { speaker, text, mood, .. }
            if speaker == "Oracle" && text == "Welcome, agent." && mood == Some("wise".into())));
    }

    #[test]
    fn parse_action_result() {
        let parser = MudParser::new();
        let event = parser.parse("You examine the crystal ball. It shimmers with spectral data.").unwrap();
        assert!(matches!(event, MudEvent::ActionResult { command, success, .. }
            if command == "examine the crystal ball" && success));
    }

    #[test]
    fn parse_tick() {
        let parser = MudParser::new();
        let event = parser.parse("TICK from jetson-hall [vision]: person detected at front door").unwrap();
        assert!(matches!(event, MudEvent::TickReceived { from, topic, body, .. }
            if from == "jetson-hall" && topic == "vision" && body.contains("person")));
    }

    #[test]
    fn parse_object_description() {
        let parser = MudParser::new();
        let event = parser.parse("Crystal Ball: A shimmering orb that reflects spectral data.").unwrap();
        assert!(matches!(event, MudEvent::ObjectDescription { name, .. } if name == "Crystal Ball"));
    }

    #[test]
    fn parse_invalid_returns_none() {
        let parser = MudParser::new();
        assert!(parser.parse("").is_none());
        assert!(parser.parse("random noise").is_none());
    }

    #[test]
    fn parse_all_multiple_events() {
        let parser = MudParser::new();
        let events = parser.parse_all("=== Kitchen ===\nExits: north\nObjects: kettle\n");
        assert_eq!(events.len(), 3);
    }

    // ─── Scene Composer Tests ──────────────────────

    #[test]
    fn compose_scene_from_events() {
        let mut composer = SceneComposer::new();
        let events = vec![
            MudEvent::RoomDescription {
                title: "Kitchen".into(), description: "A bright room.".into(),
                exits: vec!["north".into()], objects: vec!["kettle".into()],
            },
        ];
        let scene = composer.compose(&events);
        assert_eq!(scene.title, "Kitchen");
        assert_eq!(scene.exits.len(), 1);
        assert_eq!(scene.objects.len(), 1);
    }

    #[test]
    fn compose_scene_with_npc() {
        let mut composer = SceneComposer::new();
        let events = vec![
            MudEvent::NpcDialog { speaker: "Oracle".into(), text: "Hello".into(), mood: Some("wise".into()) },
        ];
        let scene = composer.compose(&events);
        assert_eq!(scene.characters.len(), 1);
        assert_eq!(scene.dialogs.len(), 1);
        assert_eq!(scene.dialogs[0].bubble_type, BubbleType::Speech);
    }

    #[test]
    fn compose_scene_has_policy_sliders() {
        let mut composer = SceneComposer::new();
        let scene = composer.compose(&[]);
        assert!(!scene.policy_sliders.is_empty());
        assert_eq!(scene.policy_sliders.len(), 4);
    }

    #[test]
    fn compose_scene_object_positions_assigned() {
        let mut composer = SceneComposer::new();
        let events = vec![
            MudEvent::RoomDescription {
                title: "Room".into(), description: String::new(),
                exits: vec![], objects: vec!["a".into(), "b".into(), "c".into()],
            },
        ];
        let scene = composer.compose(&events);
        assert_ne!(scene.objects[0].position, scene.objects[1].position);
    }

    #[test]
    fn compose_action_result_as_system_bubble() {
        let mut composer = SceneComposer::new();
        let events = vec![
            MudEvent::ActionResult { command: "look".into(), result: "You see a room.".into(), success: true },
        ];
        let scene = composer.compose(&events);
        assert_eq!(scene.dialogs.len(), 1);
        assert_eq!(scene.dialogs[0].bubble_type, BubbleType::System);
    }

    // ─── Interaction Mapper Tests ──────────────────

    #[test]
    fn map_click_examine() {
        let mapper = InteractionMapper::new();
        assert_eq!(mapper.map_click("crystal_ball", "examine"), "examine crystal_ball");
    }

    #[test]
    fn map_click_use() {
        let mapper = InteractionMapper::new();
        assert_eq!(mapper.map_click("key", "use"), "use key");
    }

    #[test]
    fn map_drag_item_to_target() {
        let mapper = InteractionMapper::new();
        assert_eq!(mapper.map_drag("key", "door"), "use key with door");
    }

    #[test]
    fn map_exit_direction() {
        let mapper = InteractionMapper::new();
        assert_eq!(mapper.map_exit("north"), "go north");
    }

    #[test]
    fn map_slider_adjustment() {
        let mapper = InteractionMapper::new();
        let cmd = mapper.map_slider("Vision Sensitivity", 0.8);
        assert_eq!(cmd, "set policy vision_sensitivity high");
    }

    #[test]
    fn map_slider_low() {
        let mapper = InteractionMapper::new();
        let cmd = mapper.map_slider("Verbosity", 0.1);
        assert_eq!(cmd, "set policy verbosity low");
    }
}
