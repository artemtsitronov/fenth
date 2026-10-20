use fenth::{Theme, Variant};
use std::collections::HashSet;

fn lightness(theme: &Theme, role: &str) -> f32 {
    theme.get(role).unwrap().to_hsl().2
}

#[test]
fn registry_is_not_empty() {
    assert!(!fenth::all().is_empty());
    assert_eq!(fenth::all().len(), fenth::names().count());
}

#[test]
fn names_are_unique_and_kebab_case() {
    let mut seen = HashSet::new();
    for name in fenth::names() {
        assert!(seen.insert(name), "duplicate theme name: {name}");
        assert!(
            name.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
            "bad theme name: {name}"
        );
    }
}

#[test]
fn lookup_by_name() {
    let a = fenth::get("catppuccin-mocha").unwrap();
    let b = fenth::get("Catppuccin-Mocha").unwrap();
    assert_eq!(a.name(), b.name());
    assert!(fenth::get("nope").is_none());
}

#[test]
fn role_names_are_unique_and_dotted() {
    let mut seen = HashSet::new();
    for role in fenth::roles() {
        assert!(seen.insert(role), "duplicate role: {role}");
        assert!(
            role.split('.')
                .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_lowercase())),
            "bad role name: {role}"
        );
    }
    assert!(seen.contains("accent.primary"));
}

#[test]
fn every_theme_answers_every_role() {
    let role_count = fenth::roles().count();
    for theme in fenth::all() {
        assert_eq!(theme.len(), role_count);
        assert_eq!(theme.iter().count(), role_count);
        for role in fenth::roles() {
            assert_eq!(theme.get(role), Some(theme[role]));
        }
    }
}

#[test]
fn hex_lookup() {
    let mocha = fenth::get("catppuccin-mocha").unwrap();
    assert_eq!(mocha.hex("accent.primary").unwrap(), "#cba6f7");
    assert_eq!(mocha.get("background").unwrap().to_hex(), "#1e1e2e");
    assert_eq!(mocha.hex("nope"), None);
}

#[test]
fn variant_matches_background_lightness() {
    for theme in fenth::all() {
        let l = lightness(theme, "background");
        match theme.variant() {
            Variant::Dark => assert!(l < 0.5, "{} is labelled dark", theme.name()),
            Variant::Light => assert!(l > 0.5, "{} is labelled light", theme.name()),
        }
    }
}

#[test]
fn colors_are_opaque() {
    for theme in fenth::all() {
        for (role, color) in theme.iter() {
            assert_eq!(color.a, 255, "{} {}", theme.name(), role);
        }
    }
}

#[test]
fn text_is_readable_against_background() {
    for theme in fenth::all() {
        let bg = lightness(theme, "background");
        let fg = lightness(theme, "foreground");
        assert!((bg - fg).abs() > 0.4, "{}", theme.name());
    }
}

#[test]
fn syntax_colors_stand_out_from_background() {
    for theme in fenth::all() {
        let bg = lightness(theme, "background");
        for (role, color) in theme.iter() {
            if role.starts_with("syntax.") {
                let l = color.to_hsl().2;
                assert!((l - bg).abs() > 0.15, "{} {}", theme.name(), role);
            }
        }
    }
}

#[test]
fn status_backgrounds_are_subtler_than_status_colors() {
    for theme in fenth::all() {
        let bg = lightness(theme, "background");
        for status in ["error", "warning", "success", "info"] {
            let solid = lightness(theme, status);
            let tinted = lightness(theme, &format!("{status}.background"));
            assert!(
                (tinted - bg).abs() < (solid - bg).abs(),
                "{} {}",
                theme.name(),
                status
            );
        }
    }
}
