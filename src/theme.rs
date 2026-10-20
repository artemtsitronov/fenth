use crate::color::Color;
use crate::error::ThemeError;
use crate::expr::{Atom, Expr};
use crate::roles::Roles;
use std::ops::Index;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    Dark,
    Light,
}

impl Variant {
    fn parse(value: &str) -> Result<Variant, ThemeError> {
        match value.to_ascii_lowercase().as_str() {
            "dark" => Ok(Variant::Dark),
            "light" => Ok(Variant::Light),
            _ => Err(ThemeError::InvalidVariant(value.to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    name: String,
    variant: Variant,
    roles: Arc<Roles>,
    colors: Vec<Color>,
}

impl Theme {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn variant(&self) -> Variant {
        self.variant
    }

    pub fn get(&self, role: &str) -> Option<Color> {
        self.roles.index_of(role).map(|i| self.colors[i])
    }

    pub fn hex(&self, role: &str) -> Option<String> {
        self.get(role).map(Color::to_hex)
    }

    pub fn len(&self) -> usize {
        self.colors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, Color)> {
        self.roles.names().zip(self.colors.iter().copied())
    }

    pub(crate) fn parse_with(source: &str, roles: &Arc<Roles>) -> Result<Theme, ThemeError> {
        let mut name: Option<String> = None;
        let mut variant: Option<Variant> = None;
        let mut overrides: Vec<Option<Expr>> = vec![None; roles.len()];

        for (i, raw) in source.lines().enumerate() {
            let line = i + 1;
            let raw = raw.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let Some((key, value)) = raw.split_once('=') else {
                return Err(ThemeError::Syntax {
                    line,
                    text: raw.to_string(),
                });
            };
            let (key, value) = (key.trim(), value.trim());
            match key {
                "name" => {
                    if name.is_some() {
                        return Err(ThemeError::DuplicateKey {
                            line,
                            key: key.to_string(),
                        });
                    }
                    name = Some(value.to_string());
                }
                "variant" => {
                    if variant.is_some() {
                        return Err(ThemeError::DuplicateKey {
                            line,
                            key: key.to_string(),
                        });
                    }
                    variant = Some(Variant::parse(value)?);
                }
                _ => {
                    let index = roles.index_of(key).ok_or_else(|| ThemeError::UnknownRole {
                        line,
                        name: key.to_string(),
                    })?;
                    if overrides[index].is_some() {
                        return Err(ThemeError::DuplicateKey {
                            line,
                            key: key.to_string(),
                        });
                    }
                    let expr = Expr::parse(value, line)?;
                    for referenced in expr.roles() {
                        if roles.index_of(referenced).is_none() {
                            return Err(ThemeError::UnknownRole {
                                line,
                                name: referenced.to_string(),
                            });
                        }
                    }
                    overrides[index] = Some(expr);
                }
            }
        }

        let name = name
            .filter(|name| !name.is_empty())
            .ok_or(ThemeError::MissingName)?;
        let variant = variant.ok_or(ThemeError::MissingVariant)?;

        let missing: Vec<String> = (0..roles.len())
            .filter(|&i| overrides[i].is_none() && roles.default(i).is_none())
            .map(|i| roles.name(i).to_string())
            .collect();
        if !missing.is_empty() {
            return Err(ThemeError::MissingRoles(missing));
        }

        let mut cache: Vec<Option<Color>> = vec![None; roles.len()];
        let mut visiting = vec![false; roles.len()];
        let mut colors = Vec::with_capacity(roles.len());
        for i in 0..roles.len() {
            colors.push(resolve(i, roles, &overrides, &mut cache, &mut visiting)?);
        }

        Ok(Theme {
            name,
            variant,
            roles: Arc::clone(roles),
            colors,
        })
    }
}

impl Index<&str> for Theme {
    type Output = Color;

    fn index(&self, role: &str) -> &Color {
        match self.roles.index_of(role) {
            Some(i) => &self.colors[i],
            None => panic!("unknown role: {role}"),
        }
    }
}

fn resolve(
    i: usize,
    roles: &Roles,
    overrides: &[Option<Expr>],
    cache: &mut [Option<Color>],
    visiting: &mut [bool],
) -> Result<Color, ThemeError> {
    if let Some(color) = cache[i] {
        return Ok(color);
    }
    if visiting[i] {
        return Err(ThemeError::CyclicRole(roles.name(i).to_string()));
    }
    visiting[i] = true;
    let expr = overrides[i]
        .as_ref()
        .or_else(|| roles.default(i))
        .ok_or_else(|| ThemeError::MissingRoles(vec![roles.name(i).to_string()]))?;
    let color = match expr {
        Expr::Atom(atom) => atom_color(atom, roles, overrides, cache, visiting)?,
        Expr::Mix(a, b, amount) => {
            let a = atom_color(a, roles, overrides, cache, visiting)?;
            let b = atom_color(b, roles, overrides, cache, visiting)?;
            a.mix(b, *amount)
        }
    };
    visiting[i] = false;
    cache[i] = Some(color);
    Ok(color)
}

fn atom_color(
    atom: &Atom,
    roles: &Roles,
    overrides: &[Option<Expr>],
    cache: &mut [Option<Color>],
    visiting: &mut [bool],
) -> Result<Color, ThemeError> {
    match atom {
        Atom::Color(color) => Ok(*color),
        Atom::Role(name) => {
            let index = roles
                .index_of(name)
                .ok_or_else(|| ThemeError::UnknownRole {
                    line: 0,
                    name: name.clone(),
                })?;
            resolve(index, roles, overrides, cache, visiting)
        }
    }
}
