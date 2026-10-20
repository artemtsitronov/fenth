use fenth::{Color, Theme, ThemeError, Variant};
use std::fs;

const REQUIRED: [(&str, &str); 17] = [
    ("background", "#1e1e2e"),
    ("surface", "#313244"),
    ("overlay", "#45475a"),
    ("selection", "#585b70"),
    ("border", "#6c7086"),
    ("foreground", "#cdd6f4"),
    ("foreground.muted", "#a6adc8"),
    ("accent.primary", "#cba6f7"),
    ("accent.secondary", "#b4befe"),
    ("red", "#f38ba8"),
    ("orange", "#fab387"),
    ("yellow", "#f9e2af"),
    ("green", "#a6e3a1"),
    ("cyan", "#94e2d5"),
    ("blue", "#89b4fa"),
    ("purple", "#cba6f7"),
    ("pink", "#f5c2e7"),
];

fn source(replace: &[(&str, &str)], extra: &str) -> String {
    let mut out = String::from("name = test\nvariant = dark\n");
    for (role, hex) in REQUIRED {
        let value = replace
            .iter()
            .find(|(r, _)| *r == role)
            .map_or(hex, |(_, v)| *v);
        out.push_str(&format!("{role} = {value}\n"));
    }
    out.push_str(extra);
    out
}

fn parse(extra: &str) -> Result<Theme, ThemeError> {
    Theme::parse(&source(&[], extra))
}

#[test]
fn minimal_theme_resolves_every_role() {
    let theme = parse("").unwrap();
    assert_eq!(theme.name(), "test");
    assert_eq!(theme.variant(), Variant::Dark);
    assert_eq!(theme.len(), fenth::roles().count());
    for role in fenth::roles() {
        assert!(theme.get(role).is_some(), "{role}");
    }
}

#[test]
fn defaults_are_derived_from_the_theme() {
    let theme = parse("").unwrap();
    assert_eq!(
        theme["accent.primary.hover"],
        theme["accent.primary"].mix(theme["foreground"], 0.12)
    );
    assert_eq!(theme["link"], theme["blue"]);
    assert_eq!(theme["terminal.red"], theme["red"]);
}

#[test]
fn defaults_follow_overridden_colors() {
    let theme = Theme::parse(&source(&[("blue", "#0000ff")], "")).unwrap();
    assert_eq!(theme["link"], Color::rgb(0, 0, 255));
    assert_eq!(theme["diff.modified"], Color::rgb(0, 0, 255));
}

#[test]
fn explicit_values_win() {
    let theme = parse("accent.primary.hover = #ffffff\n").unwrap();
    assert_eq!(theme["accent.primary.hover"], Color::WHITE);
}

#[test]
fn values_can_alias_other_roles() {
    let theme = parse("warning = orange\n").unwrap();
    assert_eq!(theme["warning"], theme["orange"]);
    assert_eq!(
        theme["warning.background"],
        theme["background"].mix(theme["orange"], 0.15)
    );
}

#[test]
fn values_can_mix() {
    let theme = parse("error = mix(red, #000000, 50%)\n").unwrap();
    assert_eq!(theme["error"], theme["red"].mix(Color::BLACK, 0.5));
}

#[test]
fn comments_blank_lines_and_spacing_are_ignored() {
    let text = format!("# a comment\n\n   \n{}\n# another\n", source(&[], ""));
    assert!(Theme::parse(&text).is_ok());
    let spaced = source(&[], "info   =   #ff00ff  \n");
    assert_eq!(
        Theme::parse(&spaced).unwrap()["info"],
        Color::rgb(255, 0, 255)
    );
}

#[test]
fn role_lookup_ignores_case() {
    let theme = parse("").unwrap();
    assert_eq!(theme.get("Accent.Primary"), theme.get("accent.primary"));
}

#[test]
#[should_panic(expected = "unknown role")]
fn indexing_an_unknown_role_panics() {
    let theme = parse("").unwrap();
    let _ = theme["nope"];
}

#[test]
fn missing_name_and_variant() {
    let no_name = source(&[], "").replacen("name = test\n", "", 1);
    assert_eq!(Theme::parse(&no_name).unwrap_err(), ThemeError::MissingName);
    let no_variant = source(&[], "").replacen("variant = dark\n", "", 1);
    assert_eq!(
        Theme::parse(&no_variant).unwrap_err(),
        ThemeError::MissingVariant
    );
}

#[test]
fn bad_variant() {
    let text = source(&[], "").replacen("variant = dark", "variant = purple", 1);
    assert!(matches!(
        Theme::parse(&text).unwrap_err(),
        ThemeError::InvalidVariant(_)
    ));
}

#[test]
fn missing_required_roles_are_listed() {
    let text = source(&[], "")
        .lines()
        .filter(|line| !line.starts_with("background =") && !line.starts_with("red ="))
        .collect::<Vec<_>>()
        .join("\n");
    match Theme::parse(&text).unwrap_err() {
        ThemeError::MissingRoles(roles) => assert_eq!(roles, ["background", "red"]),
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn unknown_role_key_is_rejected() {
    assert!(matches!(
        parse("accent.primry = #ffffff\n").unwrap_err(),
        ThemeError::UnknownRole { .. }
    ));
}

#[test]
fn unknown_role_reference_is_rejected() {
    assert!(matches!(
        parse("warning = oranje\n").unwrap_err(),
        ThemeError::UnknownRole { .. }
    ));
}

#[test]
fn bad_colors_and_expressions_are_rejected() {
    assert!(matches!(
        parse("info = #12345\n").unwrap_err(),
        ThemeError::InvalidColor { .. }
    ));
    assert!(matches!(
        parse("info = mix(red, blue)\n").unwrap_err(),
        ThemeError::InvalidExpression { .. }
    ));
    assert!(matches!(
        parse("info = mix(red, blue, 150%)\n").unwrap_err(),
        ThemeError::InvalidExpression { .. }
    ));
    assert!(matches!(
        parse("info = Not A Role\n").unwrap_err(),
        ThemeError::InvalidExpression { .. }
    ));
}

#[test]
fn malformed_and_duplicate_lines_are_rejected() {
    assert!(matches!(
        parse("this line has no equals sign\n").unwrap_err(),
        ThemeError::Syntax { .. }
    ));
    assert!(matches!(
        parse("info = #ffffff\ninfo = #000000\n").unwrap_err(),
        ThemeError::DuplicateKey { .. }
    ));
}

#[test]
fn cycles_are_detected() {
    assert!(matches!(
        parse("link = link.hover\n").unwrap_err(),
        ThemeError::CyclicRole(_)
    ));
}

#[test]
fn themes_load_from_files_and_directories() {
    let dir = std::env::temp_dir().join(format!("fenth-test-{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("b.theme"),
        source(&[], "").replace("name = test", "name = bee"),
    )
    .unwrap();
    fs::write(
        dir.join("a.theme"),
        source(&[], "").replace("name = test", "name = ay"),
    )
    .unwrap();
    fs::write(dir.join("ignored.txt"), "not a theme").unwrap();

    let single = Theme::from_file(dir.join("a.theme")).unwrap();
    assert_eq!(single.name(), "ay");

    let all = fenth::load_dir(&dir).unwrap();
    let names: Vec<&str> = all.iter().map(Theme::name).collect();
    assert_eq!(names, ["ay", "bee"]);

    fs::remove_dir_all(&dir).unwrap();
    assert!(matches!(
        Theme::from_file(dir.join("a.theme")).unwrap_err(),
        ThemeError::Io(_)
    ));
}
