# `msvc_def`

# `msvc_def`

A `no_std` (with optional `alloc` and `std` features) compatible library for reading
[Microsoft Module-Definition (`.Def`) Files](https://web.archive.org/web/20240124084213/https://learn.microsoft.com/en-us/cpp/build/reference/module-definition-dot-def-files?view=msvc-170).

```rust
const CONTENTS: &str = "
LIBRARY \"mylib\"

EXPORTS
    myfunc = inner_func @1
";

// Available both as no_std, no_alloc references only
let file = msvc_def::parse_ref(CONTENTS)?;
assert_eq!(file.is_library, Some(true));
assert_eq!(file.name, Some("mylib"));

// With iterator based variable length items
let mut export = file.exports;
assert_eq!(export.next(), Some(Ok(ExportRef::new("myfunc", Some("inner_func"), Some(1), false, false, false))));
assert_eq!(export.next(), None);

// And as no_std, alloc owned types
let file = msvc_def::parse(CONTENTS)?;
assert_eq!(file.is_library, Some(true));
assert_eq!(file.name, Some("mylib".to_string()));

// With Vec based variable length items
let mut export = file.exports;
assert_eq!(export.len(), 1);
assert_eq!(export.get(0), Some(Export::new("myfunc".to_string(), Some("inner_func".to_string()), Some(1), false, false, false)).as_ref());
assert_eq!(export.get(1), None);
```

# Usage

Add the following to `Cargo.toml`:
```toml
[dependencies]
msvc_def = "0.1.0"
```

Or add with `cargo`:
```shell
cargo add msvc_def
```

 # Features

 * `alloc`: Adds [`ModuleDefinitionFile`].
 * `std`: Adds [`Error`](core::error::Error) support for [`ParseError`]. Enables `alloc` feature.

 # Notes

 Documentation items in `code highlighting` are taken directly from the [Microsoft Reference](https://web.archive.org/web/20240124084213/https://learn.microsoft.com/en-us/cpp/build/reference/module-definition-dot-def-files?view=msvc-170).

