use glam::Vec2;
use simplemotion_lib::KeyFrameContent;

/// Creates frame content with no authored transform properties.
fn content() -> KeyFrameContent<&'static str> {
    KeyFrameContent {
        src: "sprite",
        ox: 0.0,
        oy: 0.0,
        coord: [0.0, 0.0, 0.0],
        motion: None,
        angle: 0.0,
        zx: 1.0,
        zy: 1.0,
        sx: 0.0,
        sy: 0.0,
        opa: 1.0,
        mask: None,
    }
}

/// Interpolates translation and scale before composing the affine transform.
#[test]
fn interpolates_transform_components() {
    let mut start = content();
    start.coord = [0.0, 0.0, 0.0];
    start.zx = 2.0;
    start.zy = 4.0;

    let mut end = content();
    end.coord = [10.0, 20.0, 0.0];
    end.zx = 4.0;
    end.zy = 6.0;

    let transform = start.linearly_interpolate_to_affine2(&end, 0.5);

    assert_eq!(transform.transform_point2(Vec2::ZERO), Vec2::new(5.0, 10.0));
    assert_eq!(transform.transform_point2(Vec2::ONE), Vec2::new(8.0, 15.0));
}

/// Interpolates an authored property toward the deserialized default value.
#[test]
fn interpolates_transform_properties_toward_defaults() {
    let mut start = content();
    start.zx = 2.0;

    let mut end = content();
    end.coord = [10.0, 0.0, 0.0];

    let transform = start.linearly_interpolate_to_affine2(&end, 0.5);

    assert_eq!(transform.transform_point2(Vec2::ZERO), Vec2::new(5.0, 0.0));
    assert_eq!(transform.transform_point2(Vec2::X), Vec2::new(6.5, 0.0));
}

/// Includes shear when composing the interpolated affine transform.
#[test]
fn composes_shear_into_transform() {
    let mut start = content();
    start.sx = 1.0;

    let transform = start.linearly_interpolate_to_affine2(&start, 0.0);

    assert_eq!(transform.transform_point2(Vec2::Y), Vec2::new(1.0, 1.0));
}

/// Interprets exported rotation angles as degrees.
#[test]
fn interprets_angles_as_degrees() {
    let mut start = content();
    start.angle = 180.0;

    let transform = start.linearly_interpolate_to_affine2(&start, 0.0);
    let point = transform.transform_point2(Vec2::X);

    assert!((point.x + 1.0).abs() < 0.0001);
    assert!(point.y.abs() < 0.0001);
}

/// Rotates across zero instead of taking the long way around the circle.
#[test]
fn interpolates_angles_across_zero_by_the_shortest_turn() {
    let start = content();
    let mut end = content();
    end.angle = 355.0;

    let transform = start.linearly_interpolate_to_affine2(&end, 0.5);
    let point = transform.transform_point2(Vec2::X);

    assert!((point.x - 2.5_f32.to_radians().cos()).abs() < 0.0001);
    assert!((point.y + 2.5_f32.to_radians().sin()).abs() < 0.0001);
}
