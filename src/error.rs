use core::fmt::{Debug, Display, Formatter};

/// The possible errors during parsing along with an index into the string for where the problem began.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ParseError<'a> {
    /// Kind of error.
    pub kind: ParseErrorKind<'a>,
    /// Offset into string being parsed.
    pub offset: usize,
}

impl<'a> ParseError<'a> {
    /// Create new [`ParseError`].
    pub const fn new(kind: ParseErrorKind<'a>, offset: usize) -> Self {
        Self { kind, offset }
    }

    pub(crate) const fn missing_arg(keyword: &'static str, offset: usize) -> Self {
        Self::new(ParseErrorKind::MissingArgumentFor(keyword), offset)
    }
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            ParseErrorKind::MissingArgumentFor(a) => {
                write!(f, "missing argument for '{a}'")
            }
            ParseErrorKind::MissingDesignatorFor(a) => {
                write!(f, "missing designator (':' or '=') for '{a}'")
            }
            ParseErrorKind::MissingArgumentAfterCommaFor(a) => {
                write!(f, "missing argument after comma for '{a}'")
            }
            ParseErrorKind::InvalidNumericalArgument(a) => {
                write!(f, "invalid numerical argument '{a}'")
            }
            ParseErrorKind::NumberTooLarge(a) => {
                write!(f, "number '{a}' too large")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError<'_> {}

/// Kind of error.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ParseErrorKind<'a> {
    /// Missing argument immediately after keyword requiring argument.
    MissingArgumentFor(&'static str),
    /// Missing argument after designator (`=` or `:`).
    MissingDesignatorFor(&'static str),
    /// Missing argument for comma separated keyword.
    MissingArgumentAfterCommaFor(&'static str),
    /// Expected number is not parseable as a number.
    InvalidNumericalArgument(&'a str),
    /// Parsed number is outside of allowed limits.
    NumberTooLarge(&'a str),
}
