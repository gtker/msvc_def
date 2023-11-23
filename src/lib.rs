//! # `msvc_def`
//!
//! A `no_std` (with optional `alloc` and `std` features) compatible library for reading
//! [Microsoft Module-Definition (`.Def`) Files](https://web.archive.org/web/20240124084213/https://learn.microsoft.com/en-us/cpp/build/reference/module-definition-dot-def-files?view=msvc-170).
//!
//! ```rust
//! # use msvc_def::{ExportRef, ParseError};
//! # fn t() -> Result<(), ParseError<'static>> {
//! const CONTENTS: &str = "
//! LIBRARY \"mylib\"
//!
//! EXPORTS
//!     myfunc = inner_func @1
//! ";
//!
//! // Available both as no_std, no_alloc references only
//! let file = msvc_def::parse_ref(CONTENTS)?;
//! assert_eq!(file.is_library, Some(true));
//! assert_eq!(file.name, Some("mylib"));
//!
//! // With iterator based variable length items
//! let mut export = file.exports;
//! assert_eq!(export.next(), Some(Ok(ExportRef::new("myfunc", Some("inner_func"), Some(1), false, false, false))));
//! assert_eq!(export.next(), None);
#![cfg_attr(
    feature = "alloc",
    doc = r##"
# use msvc_def::Export;

// And as no_std, alloc owned types
let file = msvc_def::parse(CONTENTS)?;
assert_eq!(file.is_library, Some(true));
assert_eq!(file.name, Some("mylib".to_string()));

// With Vec based variable length items
let mut export = file.exports;
assert_eq!(export.len(), 1);
assert_eq!(export.get(0), Some(Export::new("myfunc".to_string(), Some("inner_func".to_string()), Some(1), false, false, false)).as_ref());
assert_eq!(export.get(1), None);
"##
)]
//! # Ok(())
//! # }
//! ```
//!
//! # Usage
//!
//! Add the following to `Cargo.toml`:
//! ```toml
//! [dependencies]
//! msvc_def = "0.1.0"
//! ```
//!
//! # Features
//!
//! * `alloc`: Adds [`ModuleDefinitionFile`].
//! * `std`: Adds [`Error`](core::error::Error) support for [`ParseError`]. Enables `alloc` feature.
//!
//! # Notes
//!
//! Documentation items in `code highlighting` are taken directly from the [Microsoft Reference](https://web.archive.org/web/20240124084213/https://learn.microsoft.com/en-us/cpp/build/reference/module-definition-dot-def-files?view=msvc-170).
//!

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![no_std]
#![forbid(unsafe_code)]
#![forbid(unsafe_code)]
#![warn(
    clippy::perf,
    clippy::correctness,
    clippy::style,
    clippy::missing_const_for_fn,
    clippy::doc_markdown,
    clippy::unseparated_literal_suffix,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    missing_docs
)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

use crate::parse_ref::parse_ref_inner;

mod error;

#[cfg(feature = "alloc")]
mod parse;
mod parse_ref;
mod token_iterator;

#[cfg(test)]
mod test;

pub use error::*;
pub use parse_ref::{ExportRef, Exports, ModuleDefinitionFileRef, SectionRef, Sections};

#[cfg(feature = "alloc")]
pub use parse::*;

/// Parse without using `alloc`.
///
/// # Errors
///
/// If the file format is invalid, those described by [`ParseErrorKind`].
pub fn parse_ref(s: &str) -> Result<ModuleDefinitionFileRef<'_>, ParseError<'_>> {
    parse_ref_inner(s)
}

/// Parse with `alloc`.
///
/// # Errors
///
/// If the file format is invalid, those described by [`ParseErrorKind`].
#[cfg(feature = "alloc")]
pub fn parse(s: &str) -> Result<ModuleDefinitionFile, ParseError> {
    parse_inner(s)
}
