use std::{fmt::Display, str::FromStr};

use image::{imageops, RgbaImage};

use super::size::Size;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Style {
    Center,
    Tile,
    Position (Option<i32>, Option<i32>, Size),
    Cover,
    Contain,
    ShrinkToFit,
}

#[inline]
fn draw_contain(image: &RgbaImage, canvas: &mut RgbaImage, filter: imageops::FilterType) {
    if canvas.width() == image.width() && canvas.height() == image.height() {
        imageops::overlay(canvas, image, 0, 0);
    } else {
        let mut width = canvas.width();
        let mut height = image.height() * width / image.width();
        let x;
        let y;
        if height > canvas.height() {
            height = canvas.height();
            width = image.width() * height / image.height();
            x = (canvas.width() as i64 - width as i64) / 2;
            y = 0;
        } else {
            x = 0;
            y = (canvas.height() as i64 - height as i64) / 2;
        }
        let image = imageops::resize(image, width, height, filter);
        imageops::overlay(canvas, &image, x, y);
    }
}

#[inline]
fn draw_center(image: &RgbaImage, canvas: &mut RgbaImage) {
    let x = (canvas.width()  as i64 - image.width()  as i64) / 2;
    let y = (canvas.height() as i64 - image.height() as i64) / 2;
    imageops::overlay(canvas, image, x, y);
}

impl Style {
    pub fn paint(&self, image: &RgbaImage, canvas: &mut RgbaImage, filter: imageops::FilterType) {
        match *self {
            Style::Center => {
                draw_center(image, canvas);
            },
            Style::Tile => {
                for y in (0..canvas.height()).step_by(image.height() as usize) {
                    for x in (0..canvas.width()).step_by(image.width() as usize) {
                        imageops::overlay(canvas, image, x.into(), y.into());
                    }
                }
            },
            Style::Position(x, y, size) => {
                let image_width  = image.width();
                let image_height = image.height();
                let (w, h) = size.to_size(image_width, image_height);

                let x = if let Some(x) = x {
                    x.into()
                } else {
                    (canvas.width() as i64 - w as i64) / 2
                };

                let y = if let Some(y) = y {
                    y.into()
                } else {
                    (canvas.height() as i64 - h as i64) / 2
                };

                if w > 0 && h > 0 {
                    if w == image_width && h == image_height {
                        imageops::overlay(canvas, image, x, y);
                    } else {
                        let image = imageops::resize(image, w, h, filter);
                        imageops::overlay(canvas, &image, x, y);
                    }
                }
            },
            Style::Cover => {
                if canvas.width() == image.width() && canvas.height() == image.height() {
                    imageops::overlay(canvas, image, 0, 0);
                } else {
                    let mut width = canvas.width();
                    let mut height = image.height() * width / image.width();
                    let x;
                    let y;
                    if height < canvas.height() {
                        height = canvas.height();
                        width = image.width() * height / image.height();
                        x = (canvas.width() as i64 - width as i64) / 2;
                        y = 0;
                    } else {
                        x = 0;
                        y = (canvas.height() as i64 - height as i64) / 2;
                    }
                    let image = imageops::resize(image, width, height, filter);
                    imageops::overlay(canvas, &image, x, y);
                }
            },
            Style::Contain => {
                draw_contain(image, canvas, filter);
            },
            Style::ShrinkToFit => {
                if image.width() <= canvas.width() && image.height() <= canvas.height() {
                    draw_center(image, canvas);
                } else {
                    draw_contain(image, canvas, filter);
                }
            },
        }
    }
}

impl Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Center => {
                "center".fmt(f)
            },
            Self::Tile => {
                "tile".fmt(f)
            },
            Self::Position(x, y, size) => {
                if let Some(x) = x {
                    write!(f, "{x}")?;
                } else {
                    write!(f, "*")?;
                }

                if let Some(y) = y {
                    write!(f, " {y}")?;
                } else {
                    write!(f, " *")?;
                }

                match size {
                    Size::Scale(z) => {
                        if z < 0 {
                            write!(f, " 1/{}", -z)
                        } else {
                            write!(f, " {z}")
                        }
                    },
                    Size::Exact(w, h) => write!(f, " {w} {h}"),
                    Size::Width(w)  => write!(f, " {w} *"),
                    Size::Height(h) => write!(f, " * {h}"),
                }
            },
            Self::Cover => {
                "cover".fmt(f)
            },
            Self::Contain => {
                "contain".fmt(f)
            },
            Self::ShrinkToFit => {
                "shrink-to-fit".fmt(f)
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StyleParseError();

impl Display for StyleParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "illegal style value".fmt(f)
    }
}

impl std::error::Error for StyleParseError {}

fn parse_position_rest(x: Option<i32>, mut tokenizer: StyleTokenizer) -> Result<Style, StyleParseError> {
    let y = tokenizer.expect_int_or_asterisk()?;
    let Some(token1) = tokenizer.next() else {
        return Ok(Style::Position(x, y, Size::Scale(1)));
    };

    let token1 = token1?;
    let Some(token2) = tokenizer.next() else {
        let z = token1.expect_int()?;
        if z < 1 {
            return Err(StyleParseError());
        }
        return Ok(Style::Position(x, y, Size::Scale(z)));
    };
    let token2 = token2?;

    match (token1, token2) {
        (StyleToken::Asterisk, StyleToken::Asterisk) => {
            tokenizer.expect_end()?;
            return Ok(Style::Position(x, y, Size::Scale(1)));
        }
        (StyleToken::Asterisk, StyleToken::Int(h)) => {
            if h < 0 {
                return Err(StyleParseError());
            }
            tokenizer.expect_end()?;
            return Ok(Style::Position(x, y, Size::Height(h as u32)));
        }
        (StyleToken::Int(w), StyleToken::Asterisk) => {
            if w < 0 {
                return Err(StyleParseError());
            }
            tokenizer.expect_end()?;
            return Ok(Style::Position(x, y, Size::Width(w as u32)));
        }
        (StyleToken::Int(w), StyleToken::Int(h)) => {
            if w < 0 || h < 0 {
                return Err(StyleParseError());
            }
            tokenizer.expect_end()?;
            return Ok(Style::Position(x, y, Size::Exact(w as u32, h as u32)));
        }
        (StyleToken::Int(1), StyleToken::Slash) => {
            let divisor = tokenizer.expect_int()?;

            if divisor < 1 {
                return Err(StyleParseError());
            }

            tokenizer.expect_end()?;

            return Ok(Style::Position(x, y, Size::Scale(-divisor)));
        }
        _ => {
            return Err(StyleParseError());
        }
    }
}

impl FromStr for Style {
    type Err = StyleParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut tokenizer = StyleTokenizer::new(value);

        let Some(token) = tokenizer.next() else {
            return Err(StyleParseError());
        };

        let token = token?;
        match token {
            StyleToken::Center => {
                tokenizer.expect_end()?;
                return Ok(Style::Center);
            }
            StyleToken::Contain => {
                tokenizer.expect_end()?;
                return Ok(Style::Contain);
            }
            StyleToken::Cover => {
                tokenizer.expect_end()?;
                return Ok(Style::Cover);
            }
            StyleToken::Tile => {
                tokenizer.expect_end()?;
                return Ok(Style::Tile);
            }
            StyleToken::ShrinkToFit => {
                tokenizer.expect_end()?;
                return Ok(Style::ShrinkToFit);
            }
            StyleToken::Position => {
                let x = tokenizer.expect_int_or_asterisk()?;
                return parse_position_rest(x, tokenizer);
            },
            StyleToken::Int(x) => {
                return parse_position_rest(Some(x), tokenizer);
            },
            StyleToken::Asterisk => {
                return parse_position_rest(None, tokenizer);
            },
            _ => return Err(StyleParseError())
        }
    }
}

struct StyleTokenizer<'a> {
    src: &'a str,
    err: bool,
}

impl<'a> StyleTokenizer<'a> {
    #[inline]
    pub fn new(src: &'a str) -> Self {
        Self { src, err: false }
    }

    #[inline]
    pub fn expect_end(&mut self) -> Result<(), StyleParseError> {
        let None = self.next() else {
            return Err(StyleParseError());
        };
        Ok(())
    }

    pub fn expect_int(&mut self) -> Result<i32, StyleParseError> {
        let Some(token) = self.next() else {
            return Err(StyleParseError());
        };

        let token = token?;
        match token {
            StyleToken::Int(value) => return Ok(value),
            _ => return Err(StyleParseError()),
        }
    }

    pub fn expect_int_or_asterisk(&mut self) -> Result<Option<i32>, StyleParseError> {
        let Some(token) = self.next() else {
            return Err(StyleParseError());
        };

        let token = token?;
        match token {
            StyleToken::Asterisk => return Ok(None),
            StyleToken::Int(value) => return Ok(Some(value)),
            _ => return Err(StyleParseError()),
        }
    }
}

impl<'a> Iterator for StyleTokenizer<'a> {
    type Item = Result<StyleToken, StyleParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.err {
            return Some(Err(StyleParseError()));
        }
        let Some(index) = self.src.find(|ch: char| !ch.is_whitespace()) else {
            return None;
        };
        self.src = &self.src[index..];
        if self.src.starts_with('/') {
            self.src = &self.src[1..];
            return Some(Ok(StyleToken::Slash));
        }

        if self.src.starts_with('*') {
            self.src = &self.src[1..];
            return Some(Ok(StyleToken::Asterisk));
        }

        if self.src.starts_with(|ch: char| ch.is_ascii_alphabetic()) {
            let index = self.src.find(|ch: char| !ch.is_alphanumeric() && ch != '_' && ch != '-').unwrap_or(self.src.len());
            let value = &self.src[..index];

            let token = if value.eq_ignore_ascii_case("center") {
                StyleToken::Center
            } else if value.eq_ignore_ascii_case("tile") {
                StyleToken::Tile
            } else if value.eq_ignore_ascii_case("cover") {
                StyleToken::Cover
            } else if value.eq_ignore_ascii_case("contain") {
                StyleToken::Contain
            } else if value.eq_ignore_ascii_case("shrink-to-fit") || value.eq_ignore_ascii_case("shrinktofit") {
                StyleToken::ShrinkToFit
            } else if value.eq_ignore_ascii_case("position") {
                StyleToken::Position
            } else {
                self.err = true;
                return Some(Err(StyleParseError()));
            };
            self.src = &self.src[index..];

            return Some(Ok(token));
        }

        if !self.src.starts_with(|ch: char| ch == '-' || ch == '+' || (ch >= '0' && ch <= '9')) {
            self.err = true;
            return Some(Err(StyleParseError()));
        }

        let sign: i32;
        if self.src.starts_with('+') {
            self.src = &self.src[1..];
            sign = 1;

            if !self.src.starts_with(|ch: char| ch >= '0' && ch <= '9') {
                self.err = true;
                return Some(Err(StyleParseError()));
            }
        } else if self.src.starts_with('-') {
            self.src = &self.src[1..];
            sign = -1;

            if !self.src.starts_with(|ch: char| ch >= '0' && ch <= '9') {
                self.err = true;
                return Some(Err(StyleParseError()));
            }
        } else {
            sign = 1;
        }

        let mut value: i32 = 0;
        loop {
            let Some(ch) = self.src.chars().next() else {
                break;
            };

            if !(ch >= '0' && ch <= '9') {
                break;
            }

            if value > i32::MAX / 10 {
                self.err = true;
                return Some(Err(StyleParseError()));
            }

            value *= 10;

            let digit = ch as i32 - '0' as i32;

            if value > i32::MAX - digit {
                self.err = true;
                return Some(Err(StyleParseError()));
            }

            value += digit;
            self.src = &self.src[1..];
        }

        Some(Ok(StyleToken::Int(sign * value)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StyleToken {
    Center,
    Tile,
    Cover,
    Contain,
    ShrinkToFit,
    Position,
    Int(i32),
    Slash,
    Asterisk,
}

impl StyleToken {
    #[inline]
    pub fn expect_int(&self) -> Result<i32, StyleParseError> {
        match self {
            StyleToken::Int(value) => Ok(*value),
            _ => Err(StyleParseError())
        }
    }
}
