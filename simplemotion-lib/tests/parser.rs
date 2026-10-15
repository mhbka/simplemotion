use simplemotion_lib::{
    parse_motion_resources, FrameType, Layer, LayerType, ObjectType, ParseMotionError,
    ResourceReference,
};

/// Collects layers recursively for assertions without adding a runtime traversal API.
fn collect_layers<'a>(
    layers: &'a [Layer<ResourceReference>],
    output: &mut Vec<&'a Layer<ResourceReference>>,
) {
    for layer in layers {
        output.push(layer);
        collect_layers(&layer.children, output);
    }
}

/// Parses the supplied simple motion and preserves its layer tree.
#[test]
fn parses_the_example_motion() {
    let resources = parse_motion_resources(include_str!("../../examples/sd002.json")).unwrap();
    let motion = resources
        .motion(resources.motion_key("SD002AA").unwrap())
        .unwrap();

    assert_eq!(resources.label.as_deref(), Some("SD002"));
    assert_eq!(resources.object_type, ObjectType::Zero);
    assert_eq!(motion.last_time, 1);
    assert_eq!(motion.loop_time, -1);
    let mut layers = Vec::new();
    collect_layers(&motion.layer, &mut layers);
    assert_eq!(layers.len(), 3);

    let bg = layers
        .into_iter()
        .find(|layer| layer.label == "bg")
        .unwrap();
    let frame = bg.frame_list.first().unwrap();
    assert_eq!(frame.kind, FrameType::ParentTransformsStatically);
    assert_eq!(
        &frame.content.as_ref().unwrap().src,
        &ResourceReference::Icon(resources.icon_key("src/SD002/bg").unwrap())
    );
}

/// Resolves nested motion paths to stable motion handles.
#[test]
fn resolves_nested_motion_references() {
    let resources = parse_motion_resources(include_str!("../../examples/sd002.json")).unwrap();
    let key = resources.motion_key("SD002AA").unwrap();
    let motion = resources.motion(key).unwrap();
    let mut layers = Vec::new();
    collect_layers(&motion.layer, &mut layers);
    let nested_frame = layers
        .into_iter()
        .flat_map(|layer| &layer.frame_list)
        .find(|frame| {
            matches!(
                frame.content.as_ref().map(|content| &content.src),
                Some(ResourceReference::Motion(_))
            )
        })
        .unwrap();

    assert_eq!(nested_frame.kind, FrameType::ParentTransformsStatically);
    assert_eq!(
        &nested_frame.content.as_ref().unwrap().src,
        &ResourceReference::Motion(resources.motion_key("swing1suma").unwrap())
    );

    let animated = resources
        .motion(resources.motion_key("swing1suma").unwrap())
        .unwrap();
    assert_eq!(
        animated.layer[0].frame_list[0].kind,
        FrameType::ParentTransformsDynamically
    );
}

/// Identifies motions that are not nested by another exported motion.
#[test]
fn identifies_motion_graph_roots() {
    let resources = parse_motion_resources(include_str!("../../examples/sd002.json")).unwrap();
    let expected = ["SD002AA", "SD002AB", "SD002BA", "SD002BB", "swing2suma"]
        .iter()
        .map(|label| resources.motion_key(label).unwrap())
        .collect::<Vec<_>>();

    assert_eq!(resources.roots, expected);
}

/// Maps the sample's layer types to their observed frame-resource roles.
#[test]
fn identifies_layer_resource_roles() {
    let resources = parse_motion_resources(include_str!("../../examples/sd002.json")).unwrap();
    let root = resources
        .motion(resources.motion_key("SD002AA").unwrap())
        .unwrap();
    let mut layers = Vec::new();

    collect_layers(&root.layer, &mut layers);

    assert_eq!(layers[0].layer_type, LayerType::Mask);
    assert_eq!(layers[1].layer_type, LayerType::Icon);
    assert_eq!(layers[2].layer_type, LayerType::Motion);

    let gaku = resources
        .motion(resources.motion_key("gaku").unwrap())
        .unwrap();
    assert_eq!(gaku.layer[0].layer_type, LayerType::Layout);
}

/// Resolves declared icons while preserving special and unknown paths.
#[test]
fn resolves_icon_references() {
    let json = r#"
        {
          "object": { "unit": { "type": 0, "motion": {
            "idle": { "type": 0, "lastTime": 1, "loopTime": -1, "layer": [
              { "type": 0, "label": "sprite", "frameList": [
                { "time": 0, "type": 0, "content": { "src": "src/unit/hero" } }
              ] }
            ] }
          } } },
          "source": { "unit": { "icon": { "hero": {} } } }
        }
    "#;
    let resources = parse_motion_resources(json).unwrap();
    let motion = resources
        .motion(resources.motion_key("idle").unwrap())
        .unwrap();
    let icon = &motion.layer[0].frame_list[0].content.as_ref().unwrap().src;

    assert_eq!(
        icon,
        &ResourceReference::Icon(resources.icon_key("src/unit/hero").unwrap())
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
    assert!(parse_motion_resources("not json").is_err());
}

/// Rejects timeline values that do not land on a whole animation frame.
#[test]
fn rejects_fractional_timeline_frames() {
    let json = r#"
    {
      "object": { "unit": { "type": 0, "motion": {
        "idle": { "type": 0, "lastTime": 1.5, "loopTime": -1, "layer": [] }
      } } }
    }
    "#;

    assert!(parse_motion_resources(json).is_err());
}

/// Rejects negative absolute timeline positions while preserving negative loop sentinels.
#[test]
fn rejects_negative_timeline_frames() {
    let json = r#"
    {
      "object": { "unit": { "type": 0, "motion": {
        "idle": { "type": 0, "lastTime": -1, "loopTime": -1, "layer": [] }
      } } }
    }
    "#;

    assert!(parse_motion_resources(json).is_err());
}

/// Requires loop timing to be present on every exported motion.
#[test]
fn rejects_missing_loop_time() {
    let json = r#"
    {
      "object": { "unit": { "type": 0, "motion": {
        "idle": { "type": 0, "lastTime": 1, "layer": [] }
      } } }
    }
    "#;

    assert!(parse_motion_resources(json).is_err());
}

/// Rejects documents that cannot be flattened into one resource graph.
#[test]
fn rejects_multiple_objects() {
    let json = r#"{ "object": { "first": { "type": 0, "motion": {} }, "second": { "type": 0, "motion": {} } } }"#;

    assert!(matches!(
        parse_motion_resources(json),
        Err(ParseMotionError::Resolution(
            simplemotion_lib::ResolutionError::MultipleObjects
        ))
    ));
}
