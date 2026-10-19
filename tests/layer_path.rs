use simplemotion_lib::LayerPath;

#[test]
fn appends_layer_labels_to_the_root_motion_path() {
    let path = LayerPath::root("root").child("parent").child("child");

    assert_eq!(path.as_str(), "root/parent/child");
}
