use crate::error::ThemeError;
use crate::roles::Roles;
use crate::theme::Theme;
use std::fs;
use std::path::Path;
use std::sync::{Arc, OnceLock};

include!(concat!(env!("OUT_DIR"), "/builtin.rs"));

static ROLES: OnceLock<Arc<Roles>> = OnceLock::new();
static THEMES: OnceLock<Vec<Theme>> = OnceLock::new();

fn builtin_roles() -> &'static Arc<Roles> {
    ROLES.get_or_init(|| {
        Arc::new(Roles::parse(ROLES_SOURCE).expect("roles.list is validated by build.rs"))
    })
}

pub fn roles() -> impl Iterator<Item = &'static str> {
    builtin_roles().names()
}

pub fn all() -> &'static [Theme] {
    THEMES.get_or_init(|| {
        THEME_SOURCES
            .iter()
            .map(|(file, source)| {
                Theme::parse_with(source, builtin_roles())
                    .unwrap_or_else(|error| panic!("themes/{file}: {error}"))
            })
            .collect()
    })
}

pub fn names() -> impl Iterator<Item = &'static str> {
    all().iter().map(Theme::name)
}

pub fn get(name: &str) -> Option<&'static Theme> {
    all()
        .iter()
        .find(|theme| theme.name().eq_ignore_ascii_case(name))
}

pub fn load_dir(dir: impl AsRef<Path>) -> Result<Vec<Theme>, ThemeError> {
    let dir = dir.as_ref();
    let entries =
        fs::read_dir(dir).map_err(|error| ThemeError::Io(format!("{}: {error}", dir.display())))?;
    let mut paths: Vec<_> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "theme"))
        .collect();
    paths.sort();
    paths.iter().map(Theme::from_file).collect()
}

impl Theme {
    pub fn parse(source: &str) -> Result<Theme, ThemeError> {
        Theme::parse_with(source, builtin_roles())
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Theme, ThemeError> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)
            .map_err(|error| ThemeError::Io(format!("{}: {error}", path.display())))?;
        Theme::parse(&source)
    }
}
