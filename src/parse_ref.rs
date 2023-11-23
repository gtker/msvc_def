use crate::error::{ParseError, ParseErrorKind};
use crate::parse_ref;
use crate::token_iterator::TokenIterator;

pub(crate) const COMMENT: &str = ";";
pub(crate) const ARG_SEPARATOR: &str = ",";
pub(crate) const DOUBLE_QUOTE: &str = "\"";

pub(crate) const RESERVED_WORDS: &[&str] = &[
    "APPLOADER",
    "BASE",
    "CODE",
    "CONFORMING",
    "DATA",
    "DESCRIPTION",
    "DEV386",
    "DISCARDABLE",
    "DYNAMIC",
    "EXECUTE-ONLY",
    "EXECUTEONLY",
    "EXECUTEREAD",
    "EXETYPE",
    "EXPORTS",
    "FIXED",
    "FUNCTIONS",
    "HEAPSIZE",
    "IMPORTS",
    "IMPURE",
    "INCLUDE",
    "INITINSTANCE",
    "IOPL",
    "LIBRARY",
    "LOADONCALL",
    "LONGNAMES",
    "MOVABLE",
    "MOVEABLE",
    "MULTIPLE",
    "NAME",
    "NEWFILES",
    "NODATA",
    "NOIOPL",
    "NONAME",
    "NONCONFORMING",
    "NONDISCARDABLE",
    "NONE",
    "NONSHARED",
    "NOTWINDOWCOMPAT",
    "OBJECTS",
    "OLD",
    "PRELOAD",
    "PRIVATE",
    "PROTMODE",
    "PURE",
    "READONLY",
    "READWRITE",
    "REALMODE",
    "RESIDENT",
    "RESIDENTNAME",
    "SECTIONS",
    "SEGMENTS",
    "SHARED",
    "SINGLE",
    "STACKSIZE",
    "STUB",
    "VERSION",
    "WINDOWAPI",
    "WINDOWCOMPAT",
    "WINDOWS",
];

/// File representaion that doesn't use `alloc`, but uses iterators instead.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ModuleDefinitionFileRef<'a> {
    /// Name specified by either the `NAME` or `LIBRARY` statements.
    pub name: Option<&'a str>,

    /// Is [`true`] if the file contains a `LIBRARY` statement
    /// and [`false`] if the file contains a `NAME` statement..
    pub is_library: Option<bool>,

    /// The first argument to the `HEAP` statement.
    /// `Exposes the same functionality as the /HEAP linker option.`
    pub heap_reserve: Option<u64>,
    /// The first argument to the `HEAP` statement.
    /// `Exposes the same functionality as the /HEAP linker option.`
    pub heap_commit: Option<u64>,

    /// `The reserve value specifies the total stack allocation in virtual memory. For ARM64, x86, and x64 machines, the default stack size is 1 MB.`
    pub stack_reserve: Option<u64>,
    /// `The commit value is subject to interpretation by the operating system. In WindowsRT, it specifies the amount of physical memory to allocate at a time. Committed virtual memory causes space to be reserved in the paging file. A higher commit value saves time when the application needs more stack space, but increases the memory requirements and possibly the startup time. For ARM64, x86, and x64 machines, the default commit value is 4 KB.`
    pub stack_commit: Option<u64>,

    /// `sets a base address for the program. It overrides the default location for an EXE or DLL file. The default base address for an EXE file is 0x400000 for 32-bit images or 0x140000000 for 64-bit images. For a DLL, the default base address is 0x10000000 for 32-bit images or 0x180000000 for 64-bit images. On operating systems that don't support address space layout randomization (ASLR), or when the /DYNAMICBASE:NO option was set, the operating system first attempts to load a program at its specified or default base address. If insufficient space is available there, the system relocates the program. To prevent relocation, use the /FIXED option.`
    pub base_address: Option<u64>,

    /// `When used in a module definition file that builds a virtual device driver (VxD), allows you to specify a file name that contains an IMAGE_DOS_HEADER structure (defined in WINNT.H) to be used in the virtual device driver (VxD), rather than the default header.`
    pub stub: Option<&'a str>,

    /// `Tells LINK to put a number in the header of the .exe file or DLL. The default is version 0.`
    pub major_version: Option<u16>,
    /// `Tells LINK to put a number in the header of the .exe file or DLL. The default is version 0.`
    pub minor_version: Option<u16>,

    /// `Introduces a section of one or more definitions that are access specifiers on sections in your project's output file.`
    pub sections: Sections<'a>,
    /// `Introduces a section of one or more export definitions that specify the exported names or ordinals of functions or data.`
    pub exports: Exports<'a>,
}

impl<'a> ModuleDefinitionFileRef<'a> {
    /// Parse file without `alloc`.
    ///
    /// # Errors
    ///
    /// If the file format is invalid, those described by [`ParseErrorKind`].
    pub fn new(file: &'a str) -> Result<Self, ParseError<'a>> {
        parse_ref(file)
    }

    pub(crate) fn inner_new(file: &'a str) -> Self {
        Self {
            name: None,
            is_library: None,
            heap_reserve: None,
            heap_commit: None,
            stack_reserve: None,
            stack_commit: None,
            base_address: None,
            stub: None,
            major_version: None,
            minor_version: None,
            sections: Sections::new(file),
            exports: Exports::new(file),
        }
    }

    /// Write the file to a buffer and interpret the buffer as a string.
    ///
    /// It is safe to reuse the same buffer for multiple writes.
    ///
    /// 4 spaces will be used for indentation, and statements will be on separate lines.
    ///
    /// Errors in parsing [`Sections`] and [`Exports`] will be ignored.
    ///
    /// # Errors
    ///
    /// If the buffer isn't of sufficient size, a [`core::fmt::Result`] will be returned.
    /// If the buffer isn't valid UTF-8, a [`core::str::Utf8Error`] will be returned.
    ///
    /// The buffer may contain incomplete data in case of error.
    pub fn write_to_buffer<'buf>(
        &self,
        buf: &'buf mut [u8],
    ) -> Result<Result<&'buf str, core::str::Utf8Error>, core::fmt::Error> {
        use core::fmt::Write;

        let mut buf = Wrapper { buf, offset: 0 };

        write_file_to_write(
            &mut buf,
            self.name,
            self.is_library,
            self.base_address,
            self.heap_reserve,
            self.heap_commit,
            self.stack_reserve,
            self.stack_commit,
            self.stub,
            self.major_version,
            self.minor_version,
        )?;

        let mut has_header = false;
        let sections = self.sections;
        for section in sections {
            let Ok(section) = section else {
                continue;
            };

            if !has_header {
                writeln!(buf, "SECTIONS")?;
                has_header = true;
            }

            write!(buf, "    {}", section.name)?;
            if section.read {
                write!(buf, " READ")?;
            }

            if section.write {
                write!(buf, " WRITE")?;
            }

            if section.execute {
                write!(buf, " EXECUTE")?;
            }

            if section.shared {
                write!(buf, " SHARED")?;
            }

            writeln!(buf)?;
        }

        has_header = false;
        let exports = self.exports;
        for export in exports {
            let Ok(export) = export else {
                continue;
            };

            if !has_header {
                writeln!(buf, "EXPORTS")?;
                has_header = true;
            }

            write!(buf, "    {}", export.name)?;
            if let Some(internal_name) = export.internal_name {
                write!(buf, "={}", internal_name)?;
            }

            if let Some(ordinal) = export.ordinal {
                write!(buf, " @{ordinal}")?;
                if export.noname {
                    write!(buf, " NONAME")?;
                }
            }

            if export.private {
                write!(buf, " PRIVATE")?;
            }

            if export.data {
                write!(buf, " DATA")?;
            }

            writeln!(buf)?;
        }

        Ok(core::str::from_utf8(&buf.buf[..buf.offset]))
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn write_file_to_write(
    buf: &mut impl core::fmt::Write,
    name: Option<&str>,
    is_library: Option<bool>,
    base_address: Option<u64>,
    heap_reserve: Option<u64>,
    heap_commit: Option<u64>,
    stack_reserve: Option<u64>,
    stack_commit: Option<u64>,
    stub: Option<&str>,
    major_version: Option<u16>,
    minor_version: Option<u16>,
) -> Result<(), core::fmt::Error> {
    if let Some(name) = name {
        let quote = if needs_quotes(name) { "\"" } else { "" };

        if let Some(is_lib) = is_library {
            if is_lib {
                write!(buf, "LIBRARY {quote}{}{quote}", name)?;
            } else {
                write!(buf, "NAME {quote}{}{quote}", name)?;
            }
        }

        if let Some(base) = base_address {
            write!(buf, " BASE={base:#X}")?;
        }

        writeln!(buf)?;
    }

    if let Some(reserve) = heap_reserve {
        if let Some(commit) = heap_commit {
            writeln!(buf, "HEAPSIZE {reserve:#X},{commit:#X}")?;
        } else {
            writeln!(buf, "HEAPSIZE {reserve:#X}")?;
        }
    }

    if let Some(reserve) = stack_reserve {
        if let Some(commit) = stack_commit {
            writeln!(buf, "STACKSIZE {reserve:#X},{commit:#X}")?;
        } else {
            writeln!(buf, "STACKSIZE {reserve:#X}")?;
        }
    }

    if let Some(stub) = stub {
        let quote = if crate::parse_ref::needs_quotes(stub) {
            "\""
        } else {
            ""
        };

        writeln!(buf, "STUB:{quote}{}{quote}", stub)?;
    }

    if let Some(major_version) = major_version {
        if let Some(minor_version) = minor_version {
            writeln!(buf, "VERSION {major_version}.{minor_version}")?;
        } else {
            writeln!(buf, "VERSION {major_version}")?;
        }
    }

    Ok(())
}

/// Iterator over [`ExportRef`]s.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Exports<'a> {
    it: TokenIterator<'a>,
}

impl<'a> Exports<'a> {
    /// Create a new iterator from a `str`.
    /// This should be the same as is passed to [`ModuleDefinitionFileRef::new`].
    pub fn new(inner: &'a str) -> Self {
        let mut it = TokenIterator::new(inner);

        while let Some(token) = it.eat_token() {
            if token == "EXPORTS" {
                break;
            }
        }

        Self { it }
    }
}

impl<'a> Iterator for Exports<'a> {
    type Item = Result<ExportRef<'a>, ParseError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut internal_name = None;
        let mut ordinal = None;
        let mut noname = false;
        let mut private = false;
        let mut data = false;

        if let Some(name) = self.it.eat_token() {
            while let Some(token) = self.it.peek_token() {
                match token {
                    "=" => {
                        let Some(_equals) = self.it.eat_token() else {
                            return Some(Err(ParseError::missing_arg("EXPORTS", self.it.offset)));
                        };

                        let Some(internal_name2) = self.it.peek_token() else {
                            return Some(Err(ParseError::missing_arg("EXPORTS", self.it.offset)));
                        };
                        internal_name = Some(internal_name2);
                    }
                    "NONAME" => noname = true,
                    "PRIVATE" => private = true,
                    "DATA" => data = true,
                    ord if ord.starts_with('@') => {
                        let ord = ord.trim_start_matches('@');

                        let ord = match parse_number(ord, self.it.offset) {
                            Ok(o) => o,
                            Err(e) => return Some(Err(e)),
                        };

                        ordinal = Some(ord);
                    }
                    _ => break,
                }

                self.it.eat_token().unwrap();
            }

            // Next token isn't part of this sections
            if self.it.next_token_is_keyword() {
                while let Some(token) = self.it.eat_token() {
                    if token == "EXPORTS" {
                        break;
                    }
                }
            }

            return Some(Ok(ExportRef::new(
                name,
                internal_name,
                ordinal,
                noname,
                private,
                data,
            )));
        }

        None
    }
}

/// `[A] section of one or more export definitions that specify the exported names or ordinals of functions or data.`
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ExportRef<'a> {
    /// The public name of the exported function.
    ///
    /// If [`internal_name`](Self::internal_name) is [`None`] this is also the internal name.
    pub name: &'a str,
    /// The internal name of the function to export.
    ///
    /// If this is [`None`] the [`name`](Self::name) will be used instead.
    pub internal_name: Option<&'a str>,
    /// The ordinal associated with the export.
    ///
    /// If [`noname`](Self::noname) is [`true`] then only the ordinal is exported.
    pub ordinal: Option<u64>,
    /// `By using the optional NONAME keyword, you can export by ordinal only and reduce the size of the export table in the resulting DLL. However, if you want to use GetProcAddress on the DLL, you must know the ordinal because the name will not be valid.`
    pub noname: bool,
    /// `The optional keyword PRIVATE prevents entryname from being included in the import library generated by LINK. It does not affect the export in the image also generated by LINK.`
    pub private: bool,
    /// `The optional keyword DATA specifies that an export is data, not code.`
    pub data: bool,
}

impl<'a> ExportRef<'a> {
    /// Create a new export item.
    pub const fn new(
        name: &'a str,
        internal_name: Option<&'a str>,
        ordinal: Option<u64>,
        noname: bool,
        private: bool,
        data: bool,
    ) -> Self {
        Self {
            name,
            internal_name,
            ordinal,
            noname,
            private,
            data,
        }
    }
}

/// Iterator over [`SectionRef`]s.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Sections<'a> {
    it: TokenIterator<'a>,
}

impl<'a> Sections<'a> {
    /// Create a new iterator from a `str`.
    /// This should be the same as is passed to [`ModuleDefinitionFileRef::new`].
    pub fn new(inner: &'a str) -> Self {
        let mut it = TokenIterator::new(inner);

        while let Some(token) = it.eat_token() {
            if token == "SECTIONS" {
                break;
            }
        }

        Self { it }
    }
}

impl<'a> Iterator for Sections<'a> {
    type Item = Result<SectionRef<'a>, ParseError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut read = false;
        let mut write = false;
        let mut execute = false;
        let mut shared = false;

        if let Some(name) = self.it.eat_token() {
            while let Some(token) = self.it.peek_token() {
                match token {
                    "READ" => read = true,
                    "WRITE" => write = true,
                    "EXECUTE" => execute = true,
                    "SHARED" => shared = true,
                    "CLASS" => {
                        // Deprecated "CLASS 'classname'" syntax is supported but ignored
                        let Some(_token) = self.it.eat_token() else {
                            return Some(Err(ParseError::missing_arg("SECTIONS", self.it.offset)));
                        };

                        let Some(_classname) = self.it.eat_token() else {
                            return Some(Err(ParseError::missing_arg("SECTIONS", self.it.offset)));
                        };

                        break;
                    }
                    _ => break,
                }

                self.it.eat_token().unwrap();
            }

            // Next token isn't part of this sections
            if self.it.next_token_is_keyword() {
                while let Some(token) = self.it.eat_token() {
                    if token == "SECTIONS" {
                        break;
                    }
                }
            }

            return Some(Ok(SectionRef::new(name, read, write, execute, shared)));
        }

        None
    }
}

/// Reference based section in the image.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct SectionRef<'a> {
    /// `Name of the section in program image`.
    pub name: &'a str,
    /// `Allows read operations on data`
    pub read: bool,
    /// `Allows write operations on data`
    pub write: bool,
    /// `The section is executable`
    pub execute: bool,
    /// `Shares the section among all processes that load the image`
    pub shared: bool,
}

impl<'a> SectionRef<'a> {
    /// Create new [`SectionRef`].
    pub const fn new(name: &'a str, read: bool, write: bool, execute: bool, shared: bool) -> Self {
        Self {
            name,
            read,
            write,
            execute,
            shared,
        }
    }
}

pub fn parse_ref_inner(s: &str) -> Result<ModuleDefinitionFileRef<'_>, ParseError<'_>> {
    let mut it = TokenIterator::new(s);

    let mut file = ModuleDefinitionFileRef::inner_new(s);
    while let Some(token) = it.eat_token() {
        parser_inner(token, &mut it, &mut file)?;
    }

    Ok(file)
}

fn parser_inner<'a>(
    token: &'a str,
    it: &mut TokenIterator<'a>,
    file: &mut ModuleDefinitionFileRef<'a>,
) -> Result<(), ParseError<'a>> {
    match token {
        "NAME" | "LIBRARY" => {
            file.is_library = Some(token == "LIBRARY");

            if let Some(next_token) = it.peek_token() {
                if !next_token.starts_with("BASE") && !RESERVED_WORDS.contains(&next_token) {
                    let name = it.eat_token().unwrap();

                    file.name = Some(strip_ident(name));
                }

                if it.next_token_is("BASE") {
                    let _base = it.eat_token().unwrap();

                    let Some(_equals_sign) = it.eat_token() else {
                        return Err(ParseError::new(
                            ParseErrorKind::MissingDesignatorFor("BASE"),
                            it.offset,
                        ));
                    };

                    let Some(base) = it.eat_token() else {
                        return Err(ParseError::missing_arg("BASE", it.offset));
                    };

                    let base = parse_number(base, it.offset)?;
                    file.base_address = Some(base);
                }
            }
        }
        "HEAPSIZE" => {
            let (reserve, commit) = parse_double_arg(it, "HEAPSIZE")?;

            file.heap_reserve = Some(reserve);
            file.heap_commit = commit;
        }
        "STACKSIZE" => {
            let (reserve, commit) = parse_double_arg(it, "STACKSIZE")?;

            file.stack_reserve = Some(reserve);
            file.stack_commit = commit;
        }
        "STUB" => {
            let Some(_colon) = it.eat_token() else {
                return Err(ParseError::new(
                    ParseErrorKind::MissingDesignatorFor("STUB"),
                    it.offset,
                ));
            };

            let Some(stub) = it.eat_token() else {
                return Err(ParseError::missing_arg("STUB", it.offset));
            };

            file.stub = Some(strip_ident(stub));
        }
        "VERSION" => {
            let Some(major) = it.eat_token() else {
                return Err(ParseError::missing_arg("VERSION", it.offset));
            };

            let major = parse_u16(major, it.offset - major.len())?;
            file.major_version = Some(major);

            if it.next_token_is(".") {
                let _period = it.eat_token().unwrap();

                let Some(minor) = it.eat_token() else {
                    return Err(ParseError::missing_arg("VERSION", it.offset));
                };
                let minor = parse_u16(minor, it.offset)?;

                file.minor_version = Some(minor);
            }
        }
        "SECTIONS" => {
            while let Some(token) = it.peek_token() {
                if RESERVED_WORDS.contains(&token) {
                    break;
                }

                it.eat_token();
            }
        }

        _ => {}
    }

    Ok(())
}

fn parse_double_arg<'a>(
    it: &mut TokenIterator<'a>,
    keyword: &'static str,
) -> Result<(u64, Option<u64>), ParseError<'a>> {
    let Some(argument) = it.peek_token() else {
        return Err(ParseError::new(
            ParseErrorKind::MissingDesignatorFor(keyword),
            it.offset,
        ));
    };
    let reserve = parse_number(argument, it.offset)?;
    let _ = it.eat_token();

    let commit = if it.next_token_is(ARG_SEPARATOR) {
        let _comma = it.eat_token().unwrap();
        let Some(commit) = it.eat_token() else {
            return Err(ParseError::new(
                ParseErrorKind::MissingArgumentAfterCommaFor(keyword),
                it.offset,
            ));
        };

        Some(commit)
    } else {
        None
    };

    let commit = if let Some(commit) = commit {
        Some(parse_number(commit, it.offset)?)
    } else {
        None
    };

    Ok((reserve, commit))
}

fn parse_u16(s: &str, offset: usize) -> Result<u16, ParseError<'_>> {
    let number = parse_number(s, offset)?;

    let number = match number.try_into() {
        Ok(number) => number,
        Err(_) => {
            return Err(ParseError::new(ParseErrorKind::NumberTooLarge(s), offset));
        }
    };

    Ok(number)
}

fn parse_number(s: &str, offset: usize) -> Result<u64, ParseError<'_>> {
    let err = Err(ParseError::new(
        ParseErrorKind::InvalidNumericalArgument(s),
        offset,
    ));

    Ok(if let Some(hex) = s.strip_prefix("0x") {
        let Ok(a) = u64::from_str_radix(hex, 16) else {
            return err;
        };
        a
    } else {
        let Ok(a) = s.parse() else {
            return err;
        };
        a
    })
}

fn strip_ident(s: &str) -> &str {
    s.trim_start_matches(DOUBLE_QUOTE)
}

struct Wrapper<'a> {
    buf: &'a mut [u8],
    offset: usize,
}
impl<'a> core::fmt::Write for Wrapper<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remainder = &mut self.buf[self.offset..];
        if remainder.len() < bytes.len() {
            return Err(core::fmt::Error);
        }
        let remainder = &mut remainder[..bytes.len()];
        remainder.copy_from_slice(bytes);
        self.offset += bytes.len();

        Ok(())
    }
}

pub(crate) fn needs_quotes(s: &str) -> bool {
    s.contains(' ') || s.contains(';')
}
