use rust_webpack_template::parser::{parse_motion_json, FrameType, LayerType, ObjectType};

/// Parses the supplied simple motion and preserves its layer tree.
#[test]
fn parses_the_example_motion() {
    let document = parse_motion_json(include_str!("../examples/sd002.json")).unwrap();
    let motion = document.motion("SD002", "SD002AA").unwrap();

    assert_eq!(document.label.as_deref(), Some("SD002"));
    assert_eq!(document.object["SD002"].object_type, ObjectType::Zero);
    assert_eq!(motion.last_time, 1.0);
    assert_eq!(motion.layers_depth_first().len(), 3);

    let bg = motion
        .layers_depth_first()
        .into_iter()
        .find(|layer| layer.label == "bg")
        .unwrap();
    let frame = bg.frame_list.first().unwrap();
    assert_eq!(frame.kind, FrameType::Two);
    assert_eq!(
        frame.content.as_ref().unwrap().src.as_deref(),
        Some("src/SD002/bg")
    );
}

/// Preserves unknown exporter type values for callers that need to handle them.
#[test]
fn preserves_unknown_type_values() {
    let frame_type: FrameType = serde_json::from_str("99").unwrap();
    let layer_type: LayerType = serde_json::from_str("99").unwrap();

    assert_eq!(frame_type, FrameType::Other(99));
    assert_eq!(layer_type, LayerType::Other(99));
}

/// Rejects malformed JSON instead of returning a partial motion.
#[test]
fn rejects_invalid_json() {
    assert!(parse_motion_json("not json").is_err());
}
