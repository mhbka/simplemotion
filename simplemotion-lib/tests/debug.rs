use simplemotion_lib::{parse_motion_resources, to_anim_commands, DebugInfo};

#[test]
fn records_layer_transforms_only_when_enabled() {
    let resources =
        parse_motion_resources(include_str!("../../examples/sd002/sd002.json")).unwrap();
    let mut commands = Vec::new();
    let mut debug = DebugInfo::new(false);

    to_anim_commands(&mut commands, &resources, resources.roots[0], 0, &mut debug);
    assert!(debug.layer_transforms.is_empty());

    debug.debug_layers = true;
    to_anim_commands(&mut commands, &resources, resources.roots[0], 0, &mut debug);

    assert!(!debug.layer_transforms.is_empty());
}
