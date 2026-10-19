use simplemotion_lib::{parse_motion_resources, AnimationContext};

/// Keeps the active priority after its terminal null marker.
#[test]
fn keeps_priority_after_a_terminal_null_frame() {
    let resources = parse_motion_resources(
        r#"
        {
          "screenSize": { "width": 1, "height": 1, "originX": 0, "originY": 0 },
          "object": { "unit": { "type": 0, "motion": {
            "idle": {
              "type": 0,
              "lastTime": 1,
              "loopTime": -1,
              "priority": [
                { "time": 0, "type": 1, "content": [2, 0, 1] },
                { "time": 1, "type": 1, "content": null }
              ],
              "layer": [
                { "type": 0, "label": "a", "frameList": [{ "time": 0, "type": 2, "content": { "src": "src/unit/a" } }] },
                { "type": 0, "label": "b", "frameList": [{ "time": 0, "type": 2, "content": { "src": "src/unit/b" } }] },
                { "type": 0, "label": "c", "frameList": [{ "time": 0, "type": 2, "content": { "src": "src/unit/c" } }] }
              ]
            }
          } } },
          "source": { "unit": { "icon": {
            "a": { "width": 1, "height": 1 },
            "b": { "width": 1, "height": 1 },
            "c": { "width": 1, "height": 1 }
          } } }
        }
        "#,
    )
    .unwrap();
    let root = resources.roots[0];
    let mut context = AnimationContext::new(resources);

    let commands = context.to_anim_commands(root, 0);
    let labels: Vec<_> = commands
        .iter()
        .map(|command| context.resources().icon(command.icon).unwrap().label.clone())
        .collect();

    assert_eq!(labels.len(), 3);

    let commands_at_end = context.to_anim_commands(root, 1);
    let labels_at_end: Vec<_> = commands_at_end
        .iter()
        .map(|command| context.resources().icon(command.icon).unwrap().label.clone())
        .collect();

    assert_eq!(labels_at_end, labels);
}
