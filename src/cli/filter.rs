use std::{fmt::Display, str::FromStr};

use image::imageops;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Filter(imageops::FilterType);

impl Filter {
    #[inline]
    pub fn new(filter: imageops::FilterType) -> Self {
        Self(filter)
    }
}

impl Display for Filter {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

#[derive(Debug, PartialEq)]
pub struct FilterParseError();

impl Display for FilterParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "illegal filter type".fmt(f)
    }
}

impl std::error::Error for FilterParseError {}

impl FromStr for Filter {
    type Err = FilterParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.eq_ignore_ascii_case("catmull-rom") || value.eq_ignore_ascii_case("catmullrom") {
            Ok(Filter(imageops::FilterType::CatmullRom))
        } else if value.eq_ignore_ascii_case("gaussian") {
            Ok(Filter(imageops::FilterType::Gaussian))
        } else if value.eq_ignore_ascii_case("lanczos3") {
            Ok(Filter(imageops::FilterType::Lanczos3))
        } else if value.eq_ignore_ascii_case("nearest") {
            Ok(Filter(imageops::FilterType::Nearest))
        } else if value.eq_ignore_ascii_case("triangle") {
            Ok(Filter(imageops::FilterType::Triangle))
        } else {
            Err(FilterParseError())
        }
    }
}

impl From<Filter> for imageops::FilterType {
    #[inline]
    fn from(value: Filter) -> Self {
        value.0
    }
}
