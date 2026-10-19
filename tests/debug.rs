use simplemotion_lib::{parse_motion_resources, AnimationContext};

#[test]
fn records_layer_transforms_only_when_enabled() {
    let resources = parse_motion_resources(include_str!("../anim/sd002/sd002.json")).unwrap();
    let root = resources.roots[0];
    let mut context = AnimationContext::new(resources);

    context.to_anim_commands(root, 0);
    assert!(context.debug_info().layer_transforms.is_empty());

    context.debug_info_mut().debug_layers = true;
    context.to_anim_commands(root, 0);

    assert!(!context.debug_info().layer_transforms.is_empty());
}
