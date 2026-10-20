use crate::error::ThemeError;
use crate::expr::{Expr, is_role_name};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Roles {
    names: Vec<String>,
    defaults: Vec<Option<Expr>>,
    lines: Vec<usize>,
    index: HashMap<String, usize>,
}

impl Roles {
    pub fn parse(source: &str) -> Result<Roles, ThemeError> {
        let mut roles = Roles {
            names: Vec::new(),
            defaults: Vec::new(),
            lines: Vec::new(),
            index: HashMap::new(),
        };
        for (i, raw) in source.lines().enumerate() {
            let line = i + 1;
            let raw = raw.trim();
            if raw.is_empty() || raw.starts_with('#') {
                continue;
            }
            let (name, default) = match raw.split_once('=') {
                Some((name, default)) => (name.trim(), Some(default.trim())),
                None => (raw, None),
            };
            if !is_role_name(name) {
                return Err(ThemeError::Syntax {
                    line,
                    text: raw.to_string(),
                });
            }
            if roles.index.contains_key(name) {
                return Err(ThemeError::DuplicateKey {
                    line,
                    key: name.to_string(),
                });
            }
            let expr = default.map(|text| Expr::parse(text, line)).transpose()?;
            roles.index.insert(name.to_string(), roles.names.len());
            roles.names.push(name.to_string());
            roles.defaults.push(expr);
            roles.lines.push(line);
        }
        roles.check_references()?;
        Ok(roles)
    }

    fn check_references(&self) -> Result<(), ThemeError> {
        for (i, default) in self.defaults.iter().enumerate() {
            let Some(expr) = default else { continue };
            for name in expr.roles() {
                if !self.index.contains_key(name) {
                    return Err(ThemeError::UnknownRole {
                        line: self.lines[i],
                        name: name.to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn name(&self, i: usize) -> &str {
        &self.names[i]
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.names.iter().map(String::as_str)
    }

    pub fn default(&self, i: usize) -> Option<&Expr> {
        self.defaults[i].as_ref()
    }

    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.index
            .get(name)
            .or_else(|| self.index.get(&name.to_ascii_lowercase()))
            .copied()
    }
}
