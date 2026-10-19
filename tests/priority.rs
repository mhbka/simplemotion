use simplemotion_lib::{parse_motion_resources, to_anim_commands, DebugInfo, ExternalOverrides};

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
    let mut debug_info = DebugInfo::new(false);

    let commands = to_anim_commands(
        &resources,
        resources.roots[0],
        0,
        &ExternalOverrides::default(),
        &mut debug_info,
    );
    let labels: Vec<_> = commands
        .iter()
        .map(|command| resources.icon(command.icon).unwrap().label.as_str())
        .collect();

    assert_eq!(labels.len(), 3);

    let commands_at_end = to_anim_commands(
        &resources,
        resources.roots[0],
        1,
        &ExternalOverrides::default(),
        &mut debug_info,
    );
    let labels_at_end: Vec<_> = commands_at_end
        .iter()
        .map(|command| resources.icon(command.icon).unwrap().label.as_str())
        .collect();

    assert_eq!(labels_at_end, labels);
}
