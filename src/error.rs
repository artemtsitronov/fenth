use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseColorError {
    InvalidLength(usize),
    InvalidDigit(char),
}

impl fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(n) => {
                write!(
                    f,
                    "invalid hex color length: {n} (expected 3, 6 or 8 digits)"
                )
            }
            Self::InvalidDigit(c) => write!(f, "invalid hex digit: {c:?}"),
        }
    }
}

impl std::error::Error for ParseColorError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThemeError {
    Syntax { line: usize, text: String },
    InvalidColor { line: usize, error: ParseColorError },
    InvalidExpression { line: usize, text: String },
    InvalidVariant(String),
    MissingName,
    MissingVariant,
    DuplicateKey { line: usize, key: String },
    UnknownRole { line: usize, name: String },
    MissingRoles(Vec<String>),
    CyclicRole(String),
    Io(String),
}

impl fmt::Display for ThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { line, text } => {
                write!(f, "line {line}: expected `key = value`, got {text:?}")
            }
            Self::InvalidColor { line, error } => write!(f, "line {line}: {error}"),
            Self::InvalidExpression { line, text } => write!(
                f,
                "line {line}: invalid value {text:?} (expected #hex, a role name or mix(a, b, 25%))"
            ),
            Self::InvalidVariant(value) => {
                write!(f, "invalid variant {value:?} (expected dark or light)")
            }
            Self::MissingName => f.write_str("missing `name`"),
            Self::MissingVariant => f.write_str("missing `variant`"),
            Self::DuplicateKey { line, key } => {
                write!(f, "line {line}: `{key}` is defined twice")
            }
            Self::UnknownRole { line, name } => {
                write!(f, "line {line}: unknown role `{name}`")
            }
            Self::MissingRoles(roles) => {
                write!(f, "missing required roles: {}", roles.join(", "))
            }
            Self::CyclicRole(name) => write!(f, "role `{name}` depends on itself"),
            Self::Io(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for ThemeError {}
