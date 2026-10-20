use fenth::Color;

#[test]
fn mix_endpoints() {
    let a = Color::rgb(10, 20, 30);
    let b = Color::rgb(200, 100, 0);
    assert_eq!(a.mix(b, 0.0), a);
    assert_eq!(a.mix(b, 1.0), b);
}

#[test]
fn mix_halfway() {
    assert_eq!(
        Color::BLACK.mix(Color::WHITE, 0.5),
        Color::rgb(128, 128, 128)
    );
    assert_eq!(
        Color::rgb(0, 0, 0).mix(Color::rgb(100, 200, 50), 0.5),
        Color::rgb(50, 100, 25)
    );
}

#[test]
fn mix_clamps_the_amount() {
    let a = Color::rgb(1, 2, 3);
    let b = Color::rgb(4, 5, 6);
    assert_eq!(a.mix(b, -1.0), a);
    assert_eq!(a.mix(b, 2.0), b);
}

#[test]
fn mix_blends_alpha_too() {
    let c = Color::rgba(0, 0, 0, 0).mix(Color::rgba(0, 0, 0, 200), 0.5);
    assert_eq!(c.a, 100);
}
