use fenth::{Color, Theme};
use std::path::Path;

const STRIP: [&str; 13] = [
    "background",
    "surface",
    "foreground",
    "accent.primary",
    "accent.secondary",
    "red",
    "orange",
    "yellow",
    "green",
    "cyan",
    "blue",
    "purple",
    "pink",
];

fn swatch(color: Color, width: usize) -> String {
    format!(
        "\x1b[48;2;{};{};{}m{}\x1b[0m",
        color.r,
        color.g,
        color.b,
        " ".repeat(width)
    )
}

fn strip(theme: &Theme) -> String {
    STRIP.iter().map(|role| swatch(theme[*role], 3)).collect()
}

fn show(theme: &Theme) {
    println!("{} ({:?})", theme.name(), theme.variant());
    for (role, color) in theme.iter() {
        println!("  {} {:<26} {}", swatch(color, 6), role, color.to_hex());
    }
    println!();
}

fn load(arg: &str) -> Theme {
    if Path::new(arg).is_file() {
        return Theme::from_file(arg).unwrap_or_else(|error| {
            eprintln!("{arg}: {error}");
            std::process::exit(1);
        });
    }
    match fenth::get(arg) {
        Some(theme) => theme.clone(),
        None => {
            eprintln!("unknown theme: {arg}");
            eprintln!(
                "available: {}",
                fenth::names().collect::<Vec<_>>().join(", ")
            );
            std::process::exit(1);
        }
    }
}

fn main() {
    match std::env::args().nth(1) {
        Some(arg) => show(&load(&arg)),
        None => {
            for theme in fenth::all() {
                println!("{:<18} {}", theme.name(), strip(theme));
            }
        }
    }
}
