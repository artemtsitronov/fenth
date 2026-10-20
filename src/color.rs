use crate::error::ParseColorError;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

fn to_channel(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

impl Color {
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_hex(hex: u32) -> Self {
        Self::rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
    }

    pub const fn from_hex_alpha(hex: u32) -> Self {
        Self::rgba(
            (hex >> 24) as u8,
            (hex >> 16) as u8,
            (hex >> 8) as u8,
            hex as u8,
        )
    }

    pub fn from_rgb_f32(rgb: [f32; 3]) -> Self {
        Self::rgb(to_channel(rgb[0]), to_channel(rgb[1]), to_channel(rgb[2]))
    }

    pub fn from_rgba_f32(rgba: [f32; 4]) -> Self {
        Self::rgba(
            to_channel(rgba[0]),
            to_channel(rgba[1]),
            to_channel(rgba[2]),
            to_channel(rgba[3]),
        )
    }

    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let h = h.rem_euclid(360.0);
        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
        let m = l - c / 2.0;
        let (r, g, b) = match (h / 60.0) as u32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        Self::from_rgb_f32([r + m, g + m, b + m])
    }

    pub const fn with_alpha(self, a: u8) -> Self {
        Self { a, ..self }
    }

    pub fn mix(self, other: Color, amount: f32) -> Color {
        let t = amount.clamp(0.0, 1.0);
        let lerp = |a: u8, b: u8| (a as f32 * (1.0 - t) + b as f32 * t).round() as u8;
        Color::rgba(
            lerp(self.r, other.r),
            lerp(self.g, other.g),
            lerp(self.b, other.b),
            lerp(self.a, other.a),
        )
    }

    pub const fn to_rgb(self) -> (u8, u8, u8) {
        (self.r, self.g, self.b)
    }

    pub const fn to_rgba(self) -> (u8, u8, u8, u8) {
        (self.r, self.g, self.b, self.a)
    }

    pub const fn to_u32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub const fn to_u32_alpha(self) -> u32 {
        ((self.r as u32) << 24) | ((self.g as u32) << 16) | ((self.b as u32) << 8) | (self.a as u32)
    }

    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn to_hex_alpha(self) -> String {
        format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
    }

    pub fn to_rgb_f32(self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }

    pub fn to_rgba_f32(self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    pub fn to_css_rgb(self) -> String {
        format!("rgb({}, {}, {})", self.r, self.g, self.b)
    }

    pub fn to_css_rgba(self) -> String {
        format!(
            "rgba({}, {}, {}, {:.3})",
            self.r,
            self.g,
            self.b,
            self.a as f32 / 255.0
        )
    }

    pub fn to_hsl(self) -> (f32, f32, f32) {
        let [r, g, b] = self.to_rgb_f32();
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        let d = max - min;
        if d == 0.0 {
            return (0.0, 0.0, l);
        }
        let s = d / (1.0 - (2.0 * l - 1.0).abs());
        let h = if max == r {
            ((g - b) / d).rem_euclid(6.0)
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        (h * 60.0, s, l)
    }
}

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let s = s.strip_prefix('#').unwrap_or(s);
        let mut digits = [0u8; 8];
        let mut len = 0;
        for c in s.chars() {
            let d = c.to_digit(16).ok_or(ParseColorError::InvalidDigit(c))?;
            if len == digits.len() {
                return Err(ParseColorError::InvalidLength(s.chars().count()));
            }
            digits[len] = d as u8;
            len += 1;
        }
        let pair = |i: usize| (digits[i] << 4) | digits[i + 1];
        match len {
            3 => Ok(Color::rgb(digits[0] * 17, digits[1] * 17, digits[2] * 17)),
            6 => Ok(Color::rgb(pair(0), pair(2), pair(4))),
            8 => Ok(Color::rgba(pair(0), pair(2), pair(4), pair(6))),
            n => Err(ParseColorError::InvalidLength(n)),
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 255 {
            write!(f, "{}", self.to_hex())
        } else {
            write!(f, "{}", self.to_hex_alpha())
        }
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::rgb(r, g, b)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from((r, g, b, a): (u8, u8, u8, u8)) -> Self {
        Self::rgba(r, g, b, a)
    }
}

impl From<[u8; 3]> for Color {
    fn from([r, g, b]: [u8; 3]) -> Self {
        Self::rgb(r, g, b)
    }
}

impl From<[u8; 4]> for Color {
    fn from([r, g, b, a]: [u8; 4]) -> Self {
        Self::rgba(r, g, b, a)
    }
}
