use fenth::{Color, ParseColorError};

#[test]
fn parses_six_digit_hex() {
    assert_eq!("#1e1e2e".parse::<Color>(), Ok(Color::rgb(0x1e, 0x1e, 0x2e)));
    assert_eq!("1E1E2E".parse::<Color>(), Ok(Color::rgb(0x1e, 0x1e, 0x2e)));
    assert_eq!(
        "  #cba6f7 ".parse::<Color>(),
        Ok(Color::rgb(0xcb, 0xa6, 0xf7))
    );
}

#[test]
fn parses_short_hex() {
    assert_eq!("#fa0".parse::<Color>(), Ok(Color::rgb(0xff, 0xaa, 0x00)));
}

#[test]
fn parses_hex_with_alpha() {
    assert_eq!(
        "#11223380".parse::<Color>(),
        Ok(Color::rgba(0x11, 0x22, 0x33, 0x80))
    );
}

#[test]
fn rejects_bad_input() {
    assert_eq!("".parse::<Color>(), Err(ParseColorError::InvalidLength(0)));
    assert_eq!(
        "#12".parse::<Color>(),
        Err(ParseColorError::InvalidLength(2))
    );
    assert_eq!(
        "#1234567".parse::<Color>(),
        Err(ParseColorError::InvalidLength(7))
    );
    assert_eq!(
        "#123456789".parse::<Color>(),
        Err(ParseColorError::InvalidLength(9))
    );
    assert_eq!(
        "#12345g".parse::<Color>(),
        Err(ParseColorError::InvalidDigit('g'))
    );
}

#[test]
fn hex_output() {
    let c = Color::from_hex(0xcba6f7);
    assert_eq!(c.to_hex(), "#cba6f7");
    assert_eq!(c.with_alpha(0x80).to_hex_alpha(), "#cba6f780");
    assert_eq!(c.to_string(), "#cba6f7");
    assert_eq!(c.with_alpha(0x80).to_string(), "#cba6f780");
}

#[test]
fn integer_conversions() {
    let c = Color::from_hex(0x1e1e2e);
    assert_eq!(c.to_u32(), 0x1e1e2e);
    assert_eq!(c.to_rgb(), (0x1e, 0x1e, 0x2e));
    assert_eq!(c.to_rgba(), (0x1e, 0x1e, 0x2e, 0xff));
    assert_eq!(Color::from_hex_alpha(0x11223380).to_u32_alpha(), 0x11223380);
}

#[test]
fn tuple_and_array_conversions() {
    assert_eq!(Color::from((1, 2, 3)), Color::rgb(1, 2, 3));
    assert_eq!(Color::from([1, 2, 3, 4]), Color::rgba(1, 2, 3, 4));
}

#[test]
fn float_conversions() {
    assert_eq!(Color::rgb(255, 0, 51).to_rgb_f32(), [1.0, 0.0, 0.2]);
    assert_eq!(Color::from_rgb_f32([1.0, 0.0, 0.2]), Color::rgb(255, 0, 51));
    assert_eq!(Color::from_rgb_f32([2.0, -1.0, 0.0]), Color::rgb(255, 0, 0));
    let c = Color::rgba(10, 20, 30, 40);
    assert_eq!(Color::from_rgba_f32(c.to_rgba_f32()), c);
}

#[test]
fn css_output() {
    assert_eq!(Color::rgb(30, 30, 46).to_css_rgb(), "rgb(30, 30, 46)");
    assert_eq!(
        Color::rgba(30, 30, 46, 255).to_css_rgba(),
        "rgba(30, 30, 46, 1.000)"
    );
    assert_eq!(
        Color::rgba(0, 0, 0, 0).to_css_rgba(),
        "rgba(0, 0, 0, 0.000)"
    );
}

#[test]
fn hsl_of_primary_colors() {
    assert_eq!(Color::rgb(255, 0, 0).to_hsl(), (0.0, 1.0, 0.5));
    assert_eq!(Color::rgb(0, 255, 0).to_hsl(), (120.0, 1.0, 0.5));
    assert_eq!(Color::rgb(0, 0, 255).to_hsl(), (240.0, 1.0, 0.5));
    assert_eq!(Color::WHITE.to_hsl(), (0.0, 0.0, 1.0));
    assert_eq!(Color::BLACK.to_hsl(), (0.0, 0.0, 0.0));
}

#[test]
fn hsl_to_color() {
    assert_eq!(Color::from_hsl(0.0, 1.0, 0.5), Color::rgb(255, 0, 0));
    assert_eq!(Color::from_hsl(120.0, 1.0, 0.5), Color::rgb(0, 255, 0));
    assert_eq!(Color::from_hsl(240.0, 1.0, 0.5), Color::rgb(0, 0, 255));
    assert_eq!(Color::from_hsl(360.0, 1.0, 0.5), Color::rgb(255, 0, 0));
    assert_eq!(Color::from_hsl(-120.0, 1.0, 0.5), Color::rgb(0, 0, 255));
    assert_eq!(Color::from_hsl(50.0, 0.0, 1.0), Color::WHITE);
}

#[test]
fn hsl_roundtrip_stays_within_one_step() {
    for theme in fenth::all() {
        for (role, color) in theme.iter() {
            let (h, s, l) = color.to_hsl();
            let back = Color::from_hsl(h, s, l);
            let diff = |a: u8, b: u8| a.abs_diff(b);
            assert!(
                diff(color.r, back.r) <= 1
                    && diff(color.g, back.g) <= 1
                    && diff(color.b, back.b) <= 1,
                "{} {}: {} -> {}",
                theme.name(),
                role,
                color,
                back
            );
        }
    }
}
