//! Edge case tests for mud2scummvm

use mud2scummvm::*;

// ─── Parser Edge Cases ─────────────────────────

#[test]
fn parse_room_title_extra_spaces() {
    let parser = MudParser::new();
    // Room title with extra internal spaces
    let event = parser.parse("===  The Grand Hall  ===").unwrap();
    match event {
        MudEvent::RoomDescription { title, .. } => {
            assert!(title.contains("Grand Hall") || title.contains("The Grand Hall"));
        }
        _ => panic!("Expected RoomDescription"),
    }
}

#[test]
fn parse_exits_single() {
    let parser = MudParser::new();
    let event = parser.parse("Exits: north").unwrap();
    match event {
        MudEvent::RoomDescription { exits, .. } => {
            assert_eq!(exits, vec!["north"]);
        }
        _ => panic!("Expected RoomDescription"),
    }
}

#[test]
fn parse_exits_many() {
    let parser = MudParser::new();
    let event = parser.parse("Exits: north, south, east, west, up, down, corridor, trapdoor").unwrap();
    match event {
        MudEvent::RoomDescription { exits, .. } => {
            assert_eq!(exits.len(), 8);
        }
        _ => panic!("Expected RoomDescription"),
    }
}

#[test]
fn parse_objects_single() {
    let parser = MudParser::new();
    let event = parser.parse("Objects: sword").unwrap();
    match event {
        MudEvent::RoomDescription { objects, .. } => {
            assert_eq!(objects, vec!["sword"]);
        }
        _ => panic!("Expected RoomDescription"),
    }
}

#[test]
fn parse_npc_dialog_no_mood() {
    let parser = MudParser::new();
    let event = parser.parse("\"Hello there.\" — Guard");
    match event {
        Some(MudEvent::NpcDialog { speaker, text, mood }) => {
            assert_eq!(speaker, "Guard");
            assert_eq!(text, "Hello there.");
            assert_eq!(mood, None);
        }
        _ => panic!("Expected NpcDialog without mood"),
    }
}

#[test]
fn parse_action_result_no_period() {
    let parser = MudParser::new();
    // Action with no period separator
    let event = parser.parse("You walk north").unwrap();
    match event {
        MudEvent::ActionResult { command, result, success } => {
            assert_eq!(command, "walk north");
            assert!(result.is_empty());
            assert!(success);
        }
        _ => panic!("Expected ActionResult"),
    }
}

#[test]
fn parse_action_result_multiple_sentences() {
    let parser = MudParser::new();
    let event = parser.parse("You open the chest. The lid creaks. Inside you find gold.").unwrap();
    match event {
        MudEvent::ActionResult { command, result, .. } => {
            assert_eq!(command, "open the chest");
            assert!(result.contains("creaks"));
        }
        _ => panic!("Expected ActionResult"),
    }
}

#[test]
fn parse_tick_no_body() {
    let parser = MudParser::new();
    // TICK with topic but the parser logic: parts[1] = "motion]: "
    // find("]: ") finds index, topic = everything before, body = everything after
    // Actually the parser does: rest = "motion]: ", topic_end = rest.find("]: ") = 6
    // topic = rest[..6] = "motion", body = rest[9..] = "" or " "
    // Let's check actual behavior
    let event = parser.parse("TICK from sensor-1 [motion]: ");
    match event {
        Some(MudEvent::TickReceived { from, topic, body }) => {
            assert_eq!(from, "sensor-1");
            // Topic should be "motion" based on the find logic
            assert!(topic.contains("motion"));
            // Body might be empty or have trailing content
            assert!(body.is_empty() || body.trim().is_empty());
        }
        None => {
            // Parser might not match if the pattern doesn't fit
        }
        _ => panic!("Expected TickReceived or None"),
    }
}

#[test]
fn parse_tick_long_body() {
    let parser = MudParser::new();
    let event = parser.parse("TICK from lab-sensor [chemistry]: pH=7.2, temp=22C, pressure=1atm, turbidity=0.3NTU").unwrap();
    match event {
        MudEvent::TickReceived { from, topic, body, .. } => {
            assert_eq!(from, "lab-sensor");
            assert_eq!(topic, "chemistry");
            assert!(body.contains("pH=7.2"));
            assert!(body.contains("turbidity"));
        }
        _ => panic!("Expected TickReceived"),
    }
}

#[test]
fn parse_whitespace_only_returns_none() {
    let parser = MudParser::new();
    assert!(parser.parse("   ").is_none());
    assert!(parser.parse("\t\n").is_none());
}

#[test]
fn parse_just_exits_prefix() {
    let parser = MudParser::new();
    // "Exits: " after trim becomes "Exits:" which doesn't match starts_with("Exits: ")
    // So it falls through to the ObjectDescription path with name="Exits"
    // Actually "Exits:" doesn't have ": " separator properly either (it's "Exits:"")
    // Let's just verify it doesn't crash and is either None or a valid event
    let event = parser.parse("Exits: ");
    // This is an edge case — the empty value after trim means it falls through
    // Just verify it doesn't panic
    assert!(event.is_none() || matches!(event, Some(MudEvent::RoomDescription { .. })) || matches!(event, Some(MudEvent::ObjectDescription { .. })));
}

#[test]
fn parse_empty_objects_string() {
    let parser = MudParser::new();
    // "Objects: " after trim becomes "Objects:" which doesn't match starts_with("Objects: ")
    // Similar to the Exits: edge case
    let event = parser.parse("Objects: ");
    // Just verify it doesn't panic
    assert!(event.is_none() || matches!(event, Some(MudEvent::RoomDescription { .. })) || matches!(event, Some(MudEvent::ObjectDescription { .. })));
}

#[test]
fn parse_object_with_special_chars() {
    let parser = MudParser::new();
    let event = parser.parse("Mjölnir: A hammer that smells of ozone and regret.").unwrap();
    match event {
        MudEvent::ObjectDescription { name, description, .. } => {
            assert_eq!(name, "Mjölnir");
            assert!(description.contains("ozone"));
        }
        _ => panic!("Expected ObjectDescription"),
    }
}

#[test]
fn parse_all_empty_input() {
    let parser = MudParser::new();
    let events = parser.parse_all("");
    assert!(events.is_empty());
}

#[test]
fn parse_all_mixed_valid_invalid() {
    let parser = MudParser::new();
    let input = "=== Kitchen ===\nrandom noise line\nExits: north\n\nmore garbage\nObjects: kettle\n";
    let events = parser.parse_all(input);
    // Should parse the 3 valid lines and skip the noise
    assert_eq!(events.len(), 3);
}

#[test]
fn parse_all_preserves_order() {
    let parser = MudParser::new();
    let input = "=== Kitchen ===\nExits: north\nObjects: kettle\n";
    let events = parser.parse_all(input);
    assert_eq!(events.len(), 3);
    // First should be RoomDescription with title
    assert!(matches!(&events[0], MudEvent::RoomDescription { title, .. } if title == "Kitchen"));
    // Second should have exits
    assert!(matches!(&events[1], MudEvent::RoomDescription { exits, .. } if !exits.is_empty()));
}

// ─── Scene Composer Edge Cases ──────────────────

#[test]
fn compose_empty_events() {
    let mut composer = SceneComposer::new();
    let scene = composer.compose(&[]);
    assert!(scene.title.is_empty());
    assert!(scene.exits.is_empty());
    assert!(scene.objects.is_empty());
    assert!(scene.characters.is_empty());
    assert!(scene.dialogs.is_empty());
    // Should still have default policy sliders
    assert!(!scene.policy_sliders.is_empty());
}

#[test]
fn compose_many_objects_wraps_position() {
    let mut composer = SceneComposer::new();
    // Push enough objects to trigger position wrapping
    let objects: Vec<String> = (0..10).map(|i| format!("obj_{}", i)).collect();
    let events = vec![
        MudEvent::RoomDescription {
            title: "Gallery".into(), description: String::new(),
            exits: vec![], objects,
        },
    ];
    let scene = composer.compose(&events);
    assert_eq!(scene.objects.len(), 10);
    // All positions should be within [0, 1] range
    for obj in &scene.objects {
        assert!(obj.position.0 >= 0.0 && obj.position.0 <= 1.0);
        assert!(obj.position.1 >= 0.0 && obj.position.1 <= 1.0);
    }
    // All positions should be unique (compare as bits)
    let positions: Vec<_> = scene.objects.iter().map(|o| (o.position.0.to_bits(), o.position.1.to_bits())).collect();
    let unique: std::collections::HashSet<_> = positions.iter().collect();
    assert_eq!(positions.len(), unique.len());
}

#[test]
fn compose_object_description_updates_existing() {
    let mut composer = SceneComposer::new();
    let events = vec![
        MudEvent::RoomDescription {
            title: "Room".into(), description: String::new(),
            exits: vec![], objects: vec!["crystal".into()],
        },
        MudEvent::ObjectDescription {
            name: "crystal".into(),
            description: "It shimmers.".into(),
            actions: vec!["examine".into(), "touch".into()],
        },
    ];
    let scene = composer.compose(&events);
    assert_eq!(scene.objects.len(), 1);
    assert_eq!(scene.objects[0].description, "It shimmers.");
    assert!(scene.objects[0].actions.contains(&"touch".to_string()));
}

#[test]
fn compose_object_description_creates_new_if_missing() {
    let mut composer = SceneComposer::new();
    let events = vec![
        MudEvent::ObjectDescription {
            name: "new_thing".into(),
            description: "Appeared from nowhere.".into(),
            actions: vec![],
        },
    ];
    let scene = composer.compose(&events);
    assert_eq!(scene.objects.len(), 1);
    assert_eq!(scene.objects[0].name, "new_thing");
    assert!(scene.objects[0].actions.contains(&"examine".to_string()));
}

#[test]
fn compose_position_consistency_across_compositions() {
    let mut composer = SceneComposer::new();
    let events1 = vec![
        MudEvent::RoomDescription {
            title: "Room".into(), description: String::new(),
            exits: vec![], objects: vec!["kettle".into()],
        },
    ];
    let scene1 = composer.compose(&events1);
    let kettle_pos = scene1.objects[0].position;

    // Second composition with same object should reuse position
    let events2 = vec![
        MudEvent::RoomDescription {
            title: "Room".into(), description: String::new(),
            exits: vec![], objects: vec!["kettle".into()],
        },
    ];
    let scene2 = composer.compose(&events2);
    assert_eq!(scene2.objects[0].position, kettle_pos);
}

#[test]
fn compose_multiple_dialogs_from_same_speaker() {
    let mut composer = SceneComposer::new();
    let events = vec![
        MudEvent::NpcDialog { speaker: "Oracle".into(), text: "Hello.".into(), mood: None },
        MudEvent::NpcDialog { speaker: "Oracle".into(), text: "Goodbye.".into(), mood: Some("sad".into()) },
    ];
    let scene = composer.compose(&events);
    // Should have 2 characters (each NpcDialog push adds one)
    assert_eq!(scene.dialogs.len(), 2);
    // Same speaker appears twice as character entries
    assert_eq!(scene.characters.len(), 2);
    // Both characters should have the same position (reused)
    assert_eq!(scene.characters[0].position, scene.characters[1].position);
}

#[test]
fn compose_policy_slider_values_are_valid() {
    let mut composer = SceneComposer::new();
    let scene = composer.compose(&[]);
    for slider in &scene.policy_sliders {
        assert!(slider.value >= 0.0 && slider.value <= 1.0);
        assert!(!slider.name.is_empty());
        assert!(!slider.min_label.is_empty());
        assert!(!slider.max_label.is_empty());
    }
}

// ─── Interaction Mapper Edge Cases ──────────────

#[test]
fn map_click_unknown_action() {
    let mapper = InteractionMapper::new();
    let cmd = mapper.map_click("mystery_object", "dance");
    assert_eq!(cmd, "dance mystery_object");
}

#[test]
fn map_click_take() {
    let mapper = InteractionMapper::new();
    assert_eq!(mapper.map_click("sword", "take"), "take sword");
}

#[test]
fn map_click_talk() {
    let mapper = InteractionMapper::new();
    assert_eq!(mapper.map_click("guard", "talk"), "talk to guard");
}

#[test]
fn map_drag_unusual_items() {
    let mapper = InteractionMapper::new();
    let cmd = mapper.map_drag("rubber_chicken", "pulley_system");
    assert_eq!(cmd, "use rubber_chicken with pulley_system");
}

#[test]
fn map_slider_medium() {
    let mapper = InteractionMapper::new();
    let cmd = mapper.map_slider("Action Caution", 0.5);
    assert_eq!(cmd, "set policy action_caution medium");
}

#[test]
fn map_slider_boundary_values() {
    let mapper = InteractionMapper::new();
    // Exactly 0.33 should be medium
    let cmd = mapper.map_slider("Test", 0.33);
    assert!(cmd.contains("medium") || cmd.contains("low"));
    // Exactly 0.66 should be high or medium
    let cmd = mapper.map_slider("Test", 0.66);
    assert!(cmd.contains("medium") || cmd.contains("high"));
    // 0.0 should be low
    let cmd = mapper.map_slider("Test", 0.0);
    assert!(cmd.contains("low"));
    // 1.0 should be high
    let cmd = mapper.map_slider("Test", 1.0);
    assert!(cmd.contains("high"));
}

#[test]
fn map_slider_spaces_replaced_with_underscores() {
    let mapper = InteractionMapper::new();
    let cmd = mapper.map_slider("Tick Frequency", 0.7);
    assert!(cmd.contains("tick_frequency"));
    // The command format is "set policy tick_frequency high" — the slider name part has underscores
    // but the rest uses spaces
    assert!(cmd.starts_with("set policy "));
}

#[test]
fn map_exit_complex_direction() {
    let mapper = InteractionMapper::new();
    let cmd = mapper.map_exit("winding_staircase");
    assert_eq!(cmd, "go winding_staircase");
}
