use simplemotion_lib::{parse_motion_resources, LayerPath};

#[test]
fn finds_a_structural_child_layer_from_its_path() {
    let resources = parse_motion_resources(
        r#"{
          "screenSize": { "width": 1, "height": 1, "originX": 0, "originY": 0 },
          "object": { "unit": { "type": 0, "motion": {
            "idle": { "type": 0, "lastTime": 1, "loopTime": -1, "layer": [{
              "type": 2, "label": "parent", "frameList": [{
                "time": 0, "type": 2, "content": { "src": "layout" }
              }],
              "children": [{ "type": 0, "label": "sprite", "frameList": [{
                "time": 0, "type": 2, "content": { "src": "src/unit/hero" }
              }] }]
            }] }
          } } },
          "source": { "unit": { "icon": { "hero": { "width": 1, "height": 1 } } } }
        }"#,
    )
    .unwrap();
    let path = LayerPath::root("idle").child("parent").child("sprite");

    let layer = resources.layer_at(&path).unwrap();

    assert_eq!(layer.label, "sprite");
    assert!(resources.layer_at(&LayerPath::root("idle")).is_none());
}

#[test]
fn follows_a_nested_motion_to_find_its_layer() {
    let resources = parse_motion_resources(
        r#"{
          "screenSize": { "width": 1, "height": 1, "originX": 0, "originY": 0 },
          "object": { "unit": { "type": 0, "motion": {
            "idle": { "type": 0, "lastTime": 1, "loopTime": -1, "layer": [{
              "type": 3, "label": "parent", "frameList": [{
                "time": 0, "type": 2, "content": { "src": "motion/unit/child" }
              }]
            }] },
            "child": { "type": 0, "lastTime": 1, "loopTime": -1, "layer": [{
              "type": 0, "label": "sprite", "frameList": [{
                "time": 0, "type": 2, "content": { "src": "src/unit/hero" }
              }]
            }] }
          } } },
          "source": { "unit": { "icon": { "hero": { "width": 1, "height": 1 } } } }
        }"#,
    )
    .unwrap();
    let path = LayerPath::root("idle").child("parent").child("sprite");

    assert_eq!(resources.layer_at(&path).unwrap().label, "sprite");
}
