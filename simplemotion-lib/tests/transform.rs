use glam::Vec2;
use simplemotion_lib::KeyFrameContent;

/// Creates frame content with no authored transform properties.
fn content() -> KeyFrameContent<&'static str> {
    KeyFrameContent {
        src: "sprite",
        ox: None,
        oy: None,
        coord: None,
        angle: None,
        zx: None,
        zy: None,
        sx: None,
        sy: None,
        opa: None,
        mask: None,
    }
}

/// Interpolates translation and scale before composing the affine transform.
#[test]
fn interpolates_transform_components() {
    let mut start = content();
    start.coord = Some([0.0, 0.0, 0.0]);
    start.zx = Some(2.0);
    start.zy = Some(4.0);

    let mut end = content();
    end.coord = Some([10.0, 20.0, 0.0]);
    end.zx = Some(4.0);
    end.zy = Some(6.0);

    let transform = start.linearly_interpolate_to_affine2(&end, 0.5);

    assert_eq!(transform.transform_point2(Vec2::ZERO), Vec2::new(5.0, 10.0));
    assert_eq!(transform.transform_point2(Vec2::ONE), Vec2::new(8.0, 15.0));
}

/// Holds a supplied property when the other endpoint does not define it.
#[test]
fn preserves_sparse_transform_properties() {
    let mut start = content();
    start.zx = Some(2.0);

    let mut end = content();
    end.coord = Some([10.0, 0.0, 0.0]);

    let transform = start.linearly_interpolate_to_affine2(&end, 0.5);

    assert_eq!(transform.transform_point2(Vec2::ZERO), Vec2::new(10.0, 0.0));
    assert_eq!(transform.transform_point2(Vec2::X), Vec2::new(12.0, 0.0));
}

/// Includes shear when composing the interpolated affine transform.
#[test]
fn composes_shear_into_transform() {
    let mut start = content();
    start.sx = Some(1.0);

    let transform = start.linearly_interpolate_to_affine2(&start, 0.0);

    assert_eq!(transform.transform_point2(Vec2::Y), Vec2::new(1.0, 1.0));
}
