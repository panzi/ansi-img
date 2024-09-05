use std::{fmt::Display, str::FromStr};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum LineEnd {
    Cr,
    Lf,
    CrLf,
}

impl LineEnd {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cr => "\r",
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
        }
    }
}

impl Default for LineEnd {
    #[inline]
    fn default() -> Self {
        Self::Lf
    }
}

impl ToString for LineEnd {
    fn to_string(&self) -> String {
        match self {
            Self::Cr => "Cr",
            Self::Lf => "Lf",
            Self::CrLf => "CrLf",
        }.to_string()
    }
}

#[derive(Debug, PartialEq)]
pub struct LineEndParseError();

impl Display for LineEndParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "illegal line end value".fmt(f)
    }
}

impl std::error::Error for LineEndParseError {}

impl FromStr for LineEnd {
    type Err = LineEndParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("cr") {
            Ok(LineEnd::Cr)
        } else if value.eq_ignore_ascii_case("lf") {
            Ok(LineEnd::Lf)
        } else if value.eq_ignore_ascii_case("crlf") || value.eq_ignore_ascii_case("cr-lf") {
            Ok(LineEnd::CrLf)
        } else {
            Err(LineEndParseError())
        }
    }
}
