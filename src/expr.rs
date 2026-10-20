use crate::color::Color;
use crate::error::ThemeError;

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Color(Color),
    Role(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Atom(Atom),
    Mix(Atom, Atom, f32),
}

pub fn is_role_name(text: &str) -> bool {
    !text.is_empty()
        && text.split('.').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

impl Atom {
    fn parse(text: &str, line: usize) -> Result<Atom, ThemeError> {
        let text = text.trim();
        if text.starts_with('#') {
            text.parse::<Color>()
                .map(Atom::Color)
                .map_err(|error| ThemeError::InvalidColor { line, error })
        } else if is_role_name(text) {
            Ok(Atom::Role(text.to_string()))
        } else {
            Err(ThemeError::InvalidExpression {
                line,
                text: text.to_string(),
            })
        }
    }
}

impl Expr {
    pub fn parse(text: &str, line: usize) -> Result<Expr, ThemeError> {
        let text = text.trim();
        let invalid = || ThemeError::InvalidExpression {
            line,
            text: text.to_string(),
        };
        let Some(inner) = text
            .strip_prefix("mix(")
            .and_then(|rest| rest.strip_suffix(')'))
        else {
            return Atom::parse(text, line).map(Expr::Atom);
        };
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        if parts.len() != 3 {
            return Err(invalid());
        }
        let amount = parts[2]
            .strip_suffix('%')
            .and_then(|number| number.trim().parse::<f32>().ok())
            .filter(|number| (0.0..=100.0).contains(number))
            .ok_or_else(invalid)?;
        Ok(Expr::Mix(
            Atom::parse(parts[0], line)?,
            Atom::parse(parts[1], line)?,
            amount / 100.0,
        ))
    }

    pub fn roles(&self) -> Vec<&str> {
        let atoms: Vec<&Atom> = match self {
            Expr::Atom(atom) => vec![atom],
            Expr::Mix(a, b, _) => vec![a, b],
        };
        atoms
            .into_iter()
            .filter_map(|atom| match atom {
                Atom::Role(name) => Some(name.as_str()),
                Atom::Color(_) => None,
            })
            .collect()
    }
}
