use glam::{Affine2, Vec2};
use simplemotion_lib::{
    parse_motion_resources, AnimationContext, LayerOverride, LayerPath, OverrideLayerState,
};

#[test]
fn replaces_a_matching_layers_local_transform_and_opacity() {
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
    let root = resources.roots[0];
    let mut context = AnimationContext::new(resources);
    context.overrides_mut().override_layer(
        LayerPath::root("idle").child("parent").child("sprite"),
        LayerOverride::Completely(OverrideLayerState {
            transform: Affine2::from_translation(Vec2::new(3.0, 4.0)),
            opa: 0.5,
        }),
    );

    let commands = context.to_anim_commands(root, 0);

    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].opacity, 0.5);
    assert_eq!(
        commands[0].transform.transform_point2(Vec2::ZERO),
        Vec2::new(3.5, 4.5),
    );
}

#[test]
fn replaces_a_root_layer_with_a_donor_layer() {
    let resources = parse_motion_resources(
        r#"{
          "screenSize": { "width": 1, "height": 1, "originX": 0, "originY": 0 },
          "object": { "unit": { "type": 0, "motion": {
            "idle": { "type": 0, "lastTime": 1, "loopTime": -1, "layer": [{
              "type": 0, "label": "target", "frameList": [{
                "time": 0, "type": 2, "content": { "src": "src/unit/hero" }
              }]
            }] },
            "donor": { "type": 0, "lastTime": 1, "loopTime": -1, "layer": [{
              "type": 0, "label": "replacement", "frameList": [{
                "time": 0, "type": 2, "content": { "src": "src/unit/alternate" }
              }]
            }] }
          } } },
          "source": { "unit": { "icon": {
            "hero": { "width": 1, "height": 1 },
            "alternate": { "width": 1, "height": 1 }
          } } }
        }"#,
    )
    .unwrap();
    let root = resources.motion_key("idle").unwrap();
    let alternate = resources.icon_key("src/unit/alternate").unwrap();
    let mut context = AnimationContext::new(resources);
    context.overrides_mut().replace_layer(
        LayerPath::root("idle").child("target"),
        LayerPath::root("donor").child("replacement"),
    );

    let commands = context.to_anim_commands(root, 0);

    assert_eq!(commands.len(), 1);
    assert_eq!(
        commands[0].icon,
        alternate,
    );
}

#[test]
fn uses_the_nested_donor_motions_clock() {
    let resources = parse_motion_resources(
        r#"{
          "screenSize": { "width": 1, "height": 1, "originX": 0, "originY": 0 },
          "object": { "unit": { "type": 0, "motion": {
            "idle": { "type": 0, "lastTime": 10, "loopTime": -1, "layer": [{
              "type": 0, "label": "target", "frameList": [{
                "time": 0, "type": 2, "content": { "src": "src/unit/hero" }
              }]
            }] },
            "wrapper": { "type": 0, "lastTime": 1, "loopTime": -1, "layer": [{
              "type": 3, "label": "nested", "frameList": [{
                "time": 0, "type": 2, "content": { "src": "motion/unit/animated" }
              }]
            }] },
            "animated": { "type": 0, "lastTime": 10, "loopTime": -1, "layer": [{
              "type": 0, "label": "donor", "frameList": [
                { "time": 0, "type": 3, "content": { "src": "src/unit/hero", "coord": [0, 0, 0] } },
                { "time": 10, "type": 3, "content": { "src": "src/unit/hero", "coord": [10, 0, 0] } }
              ]
            }] }
          } } },
          "source": { "unit": { "icon": { "hero": { "width": 1, "height": 1 } } } }
        }"#,
    )
    .unwrap();
    let root = resources.motion_key("idle").unwrap();
    let mut context = AnimationContext::new(resources);
    context.overrides_mut().replace_layer(
        LayerPath::root("idle").child("target"),
        LayerPath::root("wrapper").child("nested").child("donor"),
    );

    let commands = context.to_anim_commands(root, 5);

    assert_eq!(commands.len(), 1);
    assert_eq!(
        commands[0].transform.transform_point2(Vec2::ZERO),
        Vec2::new(5.5, 0.5),
    );
}

#[test]
fn locking_a_layer_keeps_nested_motions_running() {
    let resources = parse_motion_resources(
        r#"{
          "screenSize": { "width": 1, "height": 1, "originX": 0, "originY": 0 },
          "object": { "unit": { "type": 0, "motion": {
            "idle": { "type": 0, "lastTime": 10, "loopTime": -1, "layer": [{
              "type": 3, "label": "wrapper", "frameList": [{
                "time": 0, "type": 2, "content": { "src": "motion/unit/animated" }
              }]
            }] },
            "animated": { "type": 0, "lastTime": 10, "loopTime": -1, "layer": [{
              "type": 0, "label": "sprite", "frameList": [
                { "time": 0, "type": 3, "content": { "src": "src/unit/hero", "coord": [0, 0, 0] } },
                { "time": 10, "type": 3, "content": { "src": "src/unit/hero", "coord": [10, 0, 0] } }
              ]
            }] }
          } } },
          "source": { "unit": { "icon": { "hero": { "width": 1, "height": 1 } } } }
        }"#,
    )
    .unwrap();
    let root = resources.motion_key("idle").unwrap();
    let mut context = AnimationContext::new(resources);
    context.overrides_mut().lock_layer_at_inherited_time(
        LayerPath::root("idle").child("wrapper"),
        0,
    );

    let commands = context.to_anim_commands(root, 5);

    assert_eq!(commands.len(), 1);
    assert_eq!(
        commands[0].transform.transform_point2(Vec2::ZERO),
        Vec2::new(5.5, 0.5),
    );
}
