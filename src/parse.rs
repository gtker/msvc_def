use crate::{parse_ref, ParseError};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Owned version of [`ModuleDefinitionFileRef`](crate::ModuleDefinitionFileRef).
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ModuleDefinitionFile {
    /// Name specified by either the `NAME` or `LIBRARY` statements.
    pub name: Option<String>,

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
    pub stub: Option<String>,

    /// `Tells LINK to put a number in the header of the .exe file or DLL. The default is version 0.`
    pub major_version: Option<u16>,
    /// `Tells LINK to put a number in the header of the .exe file or DLL. The default is version 0.`
    pub minor_version: Option<u16>,

    /// `Introduces a section of one or more definitions that are access specifiers on sections in your project's output file.`
    pub sections: Vec<Section>,
    /// `Introduces a section of one or more export definitions that specify the exported names or ordinals of functions or data.`
    pub exports: Vec<Export>,
}

impl ModuleDefinitionFile {
    /// Parse a new [`ModuleDefinitionFile`].
    ///
    /// # Errors
    ///
    /// If the file format is invalid, those described by [`ParseErrorKind`].
    pub fn new(s: &str) -> Result<Self, ParseError<'_>> {
        crate::parse(s)
    }

    /// Write the file to a buffer and interpret the buffer as a string.
    ///
    /// It is safe to reuse the same buffer for multiple writes.
    ///
    /// 4 spaces will be used for indentation, and statements will be on separate lines.
    ///
    /// Errors in parsing [`Sections`](crate::Sections) and [`Exports`](crate::Exports) will be ignored.
    ///
    /// # Errors
    ///
    /// If the buffer isn't of sufficient size, a [`core::fmt::Result`] will be returned.
    /// If the buffer isn't valid UTF-8, a [`core::str::Utf8Error`] will be returned.
    ///
    /// The buffer may contain incomplete data in case of error.
    pub fn write_to_buffer(&self) -> Result<String, core::fmt::Error> {
        use core::fmt::Write;

        let mut buf = String::new();

        crate::parse_ref::write_file_to_write(
            &mut buf,
            self.name.as_ref().map(|a| a.as_ref()),
            self.is_library,
            self.base_address,
            self.heap_reserve,
            self.heap_commit,
            self.stack_reserve,
            self.stack_commit,
            self.stub.as_ref().map(|a| a.as_ref()),
            self.major_version,
            self.minor_version,
        )?;

        let mut has_header = false;
        for section in &self.sections {
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
        for export in &self.exports {
            if !has_header {
                writeln!(buf, "EXPORTS")?;
                has_header = true;
            }

            write!(buf, "    {}", export.name)?;
            if let Some(internal_name) = &export.internal_name {
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

        Ok(buf)
    }
}

/// Exported function.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Export {
    /// The public name of the exported function.
    ///
    /// If [`internal_name`](Self::internal_name) is [`None`] this is also the internal name.
    pub name: String,
    /// The internal name of the function to export.
    ///
    /// If this is [`None`] the [`name`](Self::name) will be used instead.
    pub internal_name: Option<String>,
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

impl Export {
    /// Create new [`Export`].
    pub const fn new(
        name: String,
        internal_name: Option<String>,
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

/// Section in image.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Section {
    /// `Name of the section in program image`.
    pub name: String,
    /// `Allows read operations on data`
    pub read: bool,
    /// `Allows write operations on data`
    pub write: bool,
    /// `The section is executable`
    pub execute: bool,
    /// `Shares the section among all processes that load the image`
    pub shared: bool,
}

impl Section {
    /// Create new [`Section`].
    pub const fn new(name: String, read: bool, write: bool, execute: bool, shared: bool) -> Self {
        Self {
            name,
            read,
            write,
            execute,
            shared,
        }
    }
}

pub(crate) fn parse_inner(s: &str) -> Result<ModuleDefinitionFile, ParseError> {
    let s = parse_ref(s)?;

    let mut exports = Vec::new();
    for e in s.exports {
        let e = e?;

        exports.push(Export {
            name: e.name.to_string(),
            internal_name: e.internal_name.map(ToString::to_string),
            ordinal: e.ordinal,
            noname: e.noname,
            private: e.private,
            data: e.data,
        });
    }

    let mut sections = Vec::new();
    for s in s.sections {
        let s = s?;

        sections.push(Section {
            name: s.name.to_string(),
            read: s.read,
            write: s.write,
            execute: s.execute,
            shared: s.shared,
        });
    }

    Ok(ModuleDefinitionFile {
        name: s.name.map(ToString::to_string),
        is_library: s.is_library,
        heap_reserve: s.heap_reserve,
        heap_commit: s.heap_commit,
        stack_reserve: s.stack_reserve,
        stack_commit: s.stack_commit,
        base_address: s.base_address,
        stub: s.stub.map(ToString::to_string),
        major_version: s.major_version,
        minor_version: s.minor_version,
        sections,
        exports,
    })
}
