use std::{fmt::Display, str::FromStr};

use image::Rgb;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    Transparent,
    Solid(image::Rgb<u8>)
}

impl Display for Color {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::Transparent => "transparent".fmt(f),
            Color::Solid(Rgb([r, g, b])) => write!(f, "#{r:02x}{g:02x}{b:02x}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ColorParseError();

impl Display for ColorParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "illegal color value".fmt(f)
    }
}

impl std::error::Error for ColorParseError {}

impl FromStr for Color {
    type Err = ColorParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("transparent") {
            Ok(Color::Transparent)
        } else if value.starts_with('#') && value.len() == 7 {
            let Ok(r) = u8::from_str_radix(&value[1..3], 16) else { return Err(ColorParseError()); };
            let Ok(g) = u8::from_str_radix(&value[3..5], 16) else { return Err(ColorParseError()); };
            let Ok(b) = u8::from_str_radix(&value[5..7], 16) else { return Err(ColorParseError()); };

            Ok(Color::Solid(Rgb([r, g, b])))
        } else {
            Err(ColorParseError())
        }
    }
}
