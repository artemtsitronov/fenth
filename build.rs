#![allow(dead_code)]

#[path = "src/color.rs"]
mod color;
#[path = "src/error.rs"]
mod error;
#[path = "src/expr.rs"]
mod expr;
#[path = "src/roles.rs"]
mod roles;
#[path = "src/theme.rs"]
mod theme;

use roles::Roles;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{env, fs, process};
use theme::Theme;

fn fail(message: String) -> ! {
    println!("cargo::error={}", message.replace('\n', " "));
    process::exit(1);
}

fn literal(path: &Path) -> String {
    format!("{:?}", path.to_str().expect("path is not valid UTF-8"))
}

fn main() {
    println!("cargo::rerun-if-changed=roles.list");
    println!("cargo::rerun-if-changed=themes");
    println!("cargo::rerun-if-changed=src");

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let roles_path = manifest.join("roles.list");
    let roles_source = fs::read_to_string(&roles_path)
        .unwrap_or_else(|error| fail(format!("roles.list: {error}")));
    let roles = Arc::new(
        Roles::parse(&roles_source).unwrap_or_else(|error| fail(format!("roles.list: {error}"))),
    );

    let themes_dir = manifest.join("themes");
    let mut files: Vec<PathBuf> = fs::read_dir(&themes_dir)
        .unwrap_or_else(|error| fail(format!("themes/: {error}")))
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "theme"))
        .collect();
    files.sort();

    let mut seen = HashSet::new();
    let mut code = format!(
        "const ROLES_SOURCE: &str = include_str!({});\n",
        literal(&roles_path)
    );
    code.push_str("const THEME_SOURCES: &[(&str, &str)] = &[\n");
    for path in &files {
        let file = path.file_name().unwrap().to_string_lossy().into_owned();
        let source = fs::read_to_string(path)
            .unwrap_or_else(|error| fail(format!("themes/{file}: {error}")));
        let theme = Theme::parse_with(&source, &roles)
            .unwrap_or_else(|error| fail(format!("themes/{file}: {error}")));
        if !seen.insert(theme.name().to_ascii_lowercase()) {
            fail(format!(
                "themes/{file}: a theme named `{}` already exists",
                theme.name()
            ));
        }
        code.push_str(&format!(
            "    ({:?}, include_str!({})),\n",
            file,
            literal(path)
        ));
    }
    code.push_str("];\n");

    fs::write(out.join("builtin.rs"), code).unwrap();
}
