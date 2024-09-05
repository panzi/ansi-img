use std::{fmt::Display, str::FromStr};


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CanvasSize {
    Window,
    Image,
    Exact(u32, u32)
}

impl CanvasSize {
    #[inline]
    pub fn is_window(&self) -> bool {
        matches!(self, CanvasSize::Window)
    }

    #[inline]
    pub fn is_image(&self) -> bool {
        matches!(self, CanvasSize::Image)
    }

    #[inline]
    pub fn is_exact(&self) -> bool {
        matches!(self, CanvasSize::Exact(_, _))
    }
}

impl Display for CanvasSize {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            CanvasSize::Window => "window".fmt(f),
            CanvasSize::Image => "image".fmt(f),
            CanvasSize::Exact(width, height) => write!(f, "{width} {height}"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CanvasSizeParseError();

impl Display for CanvasSizeParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "illegal canvas size".fmt(f)
    }
}

impl std::error::Error for CanvasSizeParseError {}

impl FromStr for CanvasSize {
    type Err = CanvasSizeParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("window") {
            Ok(CanvasSize::Window)
        } else if value.eq_ignore_ascii_case("image") {
            Ok(CanvasSize::Image)
        } else {
            let mut items = value.split_ascii_whitespace();
            let Some(Ok(width)) = items.next().map(|value| value.parse()) else {
                return Err(CanvasSizeParseError());
            };

            let Some(Ok(height)) = items.next().map(|value| value.parse()) else {
                return Err(CanvasSizeParseError());
            };

            if let Some(_) = items.next() {
                return Err(CanvasSizeParseError());
            }

            Ok(CanvasSize::Exact(width, height))
        }
    }
}
