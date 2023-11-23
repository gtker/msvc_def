use crate::error::{ParseError, ParseErrorKind};
use crate::parse_ref;
use crate::parse_ref::{ExportRef, ModuleDefinitionFileRef, SectionRef};
#[cfg(feature = "alloc")]
use crate::ModuleDefinitionFile;

fn p(s: &str) -> ModuleDefinitionFileRef<'_> {
    parse_ref(s).unwrap()
}

fn err(s: &str, err: ParseError<'_>) {
    match parse_ref(s) {
        Ok(_) => panic!("expected error: {err}"),
        Err(e) => assert_eq!(e, err),
    }
}

#[test]
fn read_synthetic() {
    const FILE: &str = "\
; This is a comment 
NAME \"mylib with spaces\" ; with comment
HEAPSIZE 32768 ,1
; This is a comment2
EXPORTS
    myfunc";

    let f = p(FILE);

    assert_eq!(f.name.unwrap(), "mylib with spaces");
    assert_eq!(f.heap_reserve.unwrap(), 32768);
    assert_eq!(f.heap_commit.unwrap(), 1);
}

#[test]
fn library_name() {
    assert_eq!(p("NAME").is_library.unwrap(), false);
    assert_eq!(p("LIBRARY").is_library.unwrap(), true);

    let f = p("LIBRARY simple");
    assert_eq!(f.name.unwrap(), "simple");
    assert_eq!(f.is_library.unwrap(), true);

    let f = p("NAME simple");
    assert_eq!(f.name.unwrap(), "simple");
    assert_eq!(f.is_library.unwrap(), false);

    assert_eq!(p("LIBRARY \"simple\"").name.unwrap(), "simple");
    assert_eq!(p("NAME \"simple\"").name.unwrap(), "simple");

    assert_eq!(
        p("LIBRARY \"complex1234$;;\"").name.unwrap(),
        "complex1234$;;"
    );
    assert_eq!(
        p("NAME \"complex   123   4$;\n   ;\n\nEXPORTS\"")
            .name
            .unwrap(),
        "complex   123   4$;\n   ;\n\nEXPORTS"
    );

    assert_eq!(p("LIBRARY \"EXPORTS\"").name.unwrap(), "EXPORTS");
    assert_eq!(p("NAME \"EXPORTS\"").name.unwrap(), "EXPORTS");

    assert!(p("LIBRARY EXPORTS a").name.is_none());
    assert!(p("NAME EXPORTS a").name.is_none());

    let f = p("LIBRARY BASE=0x10");
    assert_eq!(f.base_address.unwrap(), 0x10);

    let f = p("NAME BASE=0x10");
    assert_eq!(f.base_address.unwrap(), 0x10);

    let f = p("LIBRARY simple BASE=0x10");
    assert_eq!(f.base_address.unwrap(), 0x10);

    let f = p("NAME simple BASE=0x10");
    assert_eq!(f.base_address.unwrap(), 0x10);

    let f = p("LIBRARY \"complex \n\n\n\n\" BASE=0x10");
    assert_eq!(f.base_address.unwrap(), 0x10);

    let f = p("NAME \"complex \n\n\n\n\" BASE=0x10");
    assert_eq!(f.base_address.unwrap(), 0x10);

    let f = p("NAME \n\tsimple BASE=0x10");
    assert_eq!(f.base_address.unwrap(), 0x10);

    let f = p("NAME \n\tsimple BASE =0x10");
    assert_eq!(f.base_address.unwrap(), 0x10);

    let f = p("NAME \n\tsimple BASE= 0x10");
    assert_eq!(f.base_address.unwrap(), 0x10);

    let f = p("NAME \n\tsimple BASE = 0x10");
    assert_eq!(f.base_address.unwrap(), 0x10);

    let f = p("NAME \n\tsimple BASE \t \n \t \t \n     =    \t \t \t \n \n 0x10");
    assert_eq!(f.base_address.unwrap(), 0x10);

    let f = p("NAME \n\tsimple BASE \t \n \t \t \n     =    \t \t \t \n \n 0x10 \n\n\n \t\t  ");
    assert_eq!(f.base_address.unwrap(), 0x10);
}

#[test]
fn heapsize() {
    err(
        "HEAPSIZE  ",
        ParseError::new(ParseErrorKind::MissingDesignatorFor("HEAPSIZE"), 8),
    );

    // Simple read
    assert_eq!(p("HEAPSIZE 1").heap_reserve.unwrap(), 1);
    assert_eq!(p("HEAPSIZE 1024").heap_reserve.unwrap(), 1024);
    assert_eq!(p("HEAPSIZE 65536").heap_reserve.unwrap(), 65536);

    let f = p("HEAPSIZE 1024,1025");
    assert_eq!(f.heap_reserve.unwrap(), 1024);
    assert_eq!(f.heap_commit.unwrap(), 1025);

    let f = p("HEAPSIZE 1024, 1025");
    assert_eq!(f.heap_reserve.unwrap(), 1024);
    assert_eq!(f.heap_commit.unwrap(), 1025);

    let f = p("HEAPSIZE 1024 ,1025");
    assert_eq!(f.heap_reserve.unwrap(), 1024);
    assert_eq!(f.heap_commit.unwrap(), 1025);

    let f = p("HEAPSIZE 1024 , 1025");
    assert_eq!(f.heap_reserve.unwrap(), 1024);
    assert_eq!(f.heap_commit.unwrap(), 1025);

    assert_eq!(p("HEAPSIZE 0x1").heap_reserve.unwrap(), 1);
    assert_eq!(p("HEAPSIZE 0x400").heap_reserve.unwrap(), 1024);
    assert_eq!(p("HEAPSIZE 0x10000").heap_reserve.unwrap(), 65536);

    let f = p("HEAPSIZE 0x400,0x401");
    assert_eq!(f.heap_reserve.unwrap(), 1024);
    assert_eq!(f.heap_commit.unwrap(), 1025);

    let f = p("HEAPSIZE 0x400, 0x401");
    assert_eq!(f.heap_reserve.unwrap(), 1024);
    assert_eq!(f.heap_commit.unwrap(), 1025);

    let f = p("HEAPSIZE 0x400 ,0x401");
    assert_eq!(f.heap_reserve.unwrap(), 1024);
    assert_eq!(f.heap_commit.unwrap(), 1025);

    let f = p("HEAPSIZE 0x400 , 0x401");
    assert_eq!(f.heap_reserve.unwrap(), 1024);
    assert_eq!(f.heap_commit.unwrap(), 1025);

    // Whitespace
    assert_eq!(
        p("
        HEAPSIZE\n\n\n\n1\n\n\n")
        .heap_reserve
        .unwrap(),
        1
    );
    assert_eq!(p("\n\n\n\n HEAPSIZE    \t1024").heap_reserve.unwrap(), 1024);
    assert_eq!(p("\tHEAPSIZE\t65536\t\n").heap_reserve.unwrap(), 65536);

    // Only hex and dec allowed according to spec
    err(
        "HEAPSIZE 0b1",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b1"), 9),
    );
    err(
        "HEAPSIZE 0b100001",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b100001"), 9),
    );

    err(
        "HEAPSIZE 1, 0b11",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b11"), 12),
    );
    err(
        "HEAPSIZE 1 ,0b11",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b11"), 12),
    );
    err(
        "HEAPSIZE 1 , 0b11",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b11"), 13),
    );
    err(
        "HEAPSIZE 1,0b11",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b11"), 11),
    );

    err(
        "HEAPSIZE 0b1, 1",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b1"), 9),
    );
    err(
        "HEAPSIZE 0b1 ,1",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b1"), 9),
    );
    err(
        "HEAPSIZE 0b1 , 1",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b1"), 9),
    );
    err(
        "HEAPSIZE 0b1,1",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b1"), 9),
    );
}

#[test]
fn stacksize() {
    err(
        "STACKSIZE  ",
        ParseError::new(ParseErrorKind::MissingDesignatorFor("STACKSIZE"), 9),
    );

    // Simple read
    assert_eq!(p("STACKSIZE 1").stack_reserve.unwrap(), 1);
    assert_eq!(p("STACKSIZE 1024").stack_reserve.unwrap(), 1024);
    assert_eq!(p("STACKSIZE 65536").stack_reserve.unwrap(), 65536);

    let f = p("STACKSIZE 1024,1025");
    assert_eq!(f.stack_reserve.unwrap(), 1024);
    assert_eq!(f.stack_commit.unwrap(), 1025);

    let f = p("STACKSIZE 1024, 1025");
    assert_eq!(f.stack_reserve.unwrap(), 1024);
    assert_eq!(f.stack_commit.unwrap(), 1025);

    let f = p("STACKSIZE 1024 ,1025");
    assert_eq!(f.stack_reserve.unwrap(), 1024);
    assert_eq!(f.stack_commit.unwrap(), 1025);

    let f = p("STACKSIZE 1024 , 1025");
    assert_eq!(f.stack_reserve.unwrap(), 1024);
    assert_eq!(f.stack_commit.unwrap(), 1025);

    assert_eq!(p("STACKSIZE 0x1").stack_reserve.unwrap(), 1);
    assert_eq!(p("STACKSIZE 0x400").stack_reserve.unwrap(), 1024);
    assert_eq!(p("STACKSIZE 0x10000").stack_reserve.unwrap(), 65536);

    let f = p("STACKSIZE 0x400,0x401");
    assert_eq!(f.stack_reserve.unwrap(), 1024);
    assert_eq!(f.stack_commit.unwrap(), 1025);

    let f = p("STACKSIZE 0x400, 0x401");
    assert_eq!(f.stack_reserve.unwrap(), 1024);
    assert_eq!(f.stack_commit.unwrap(), 1025);

    let f = p("STACKSIZE 0x400 ,0x401");
    assert_eq!(f.stack_reserve.unwrap(), 1024);
    assert_eq!(f.stack_commit.unwrap(), 1025);

    let f = p("STACKSIZE 0x400 , 0x401");
    assert_eq!(f.stack_reserve.unwrap(), 1024);
    assert_eq!(f.stack_commit.unwrap(), 1025);

    // Whitespace
    assert_eq!(
        p("
        STACKSIZE\n\n\n\n1\n\n\n")
        .stack_reserve
        .unwrap(),
        1
    );
    assert_eq!(
        p("\n\n\n\n STACKSIZE    \t1024").stack_reserve.unwrap(),
        1024
    );
    assert_eq!(p("\tSTACKSIZE\t65536\t\n").stack_reserve.unwrap(), 65536);

    // Only hex and dec allowed according to spec
    err(
        "STACKSIZE 0b1",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b1"), 10),
    );
    err(
        "STACKSIZE 0b100001",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b100001"), 10),
    );

    err(
        "STACKSIZE 1, 0b11",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b11"), 13),
    );
    err(
        "STACKSIZE 1 ,0b11",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b11"), 13),
    );
    err(
        "STACKSIZE 1 , 0b11",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b11"), 14),
    );
    err(
        "STACKSIZE 1,0b11",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b11"), 12),
    );

    err(
        "STACKSIZE 0b1, 1",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b1"), 10),
    );
    err(
        "STACKSIZE 0b1 ,1",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b1"), 10),
    );
    err(
        "STACKSIZE 0b1 , 1",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b1"), 10),
    );
    err(
        "STACKSIZE 0b1,1",
        ParseError::new(ParseErrorKind::InvalidNumericalArgument("0b1"), 10),
    );
}

#[test]
fn stub() {
    err(
        "STUB  ",
        ParseError::new(ParseErrorKind::MissingDesignatorFor("STUB"), 4),
    );
    err("STUB :  ", ParseError::missing_arg("STUB", 6));

    assert_eq!(p("STUB:filename.x").stub.unwrap(), "filename.x");
    assert_eq!(p("STUB :filename.x").stub.unwrap(), "filename.x");
    assert_eq!(p("STUB : filename.x").stub.unwrap(), "filename.x");
    assert_eq!(p("STUB: filename.x").stub.unwrap(), "filename.x");

    assert_eq!(
        p("STUB \t\t\t\n: \n\n\n \t \t \tfilename.x").stub.unwrap(),
        "filename.x"
    );

    assert_eq!(
        p("STUB:\"EXPORTS.filename.x\"").stub.unwrap(),
        "EXPORTS.filename.x"
    );
    assert_eq!(
        p("STUB :\"EXPORTS.filename.x\"").stub.unwrap(),
        "EXPORTS.filename.x"
    );
    assert_eq!(
        p("STUB : \"EXPORTS.filename.x\"").stub.unwrap(),
        "EXPORTS.filename.x"
    );
    assert_eq!(
        p("STUB: \"EXPORTS.filename.x\"").stub.unwrap(),
        "EXPORTS.filename.x"
    );
}

#[test]
fn version() {
    err("VERSION ", ParseError::missing_arg("VERSION", 7));
    assert_eq!(p("VERSION 1").major_version.unwrap(), 1);

    let f = p("VERSION 1.2");
    assert_eq!(f.major_version.unwrap(), 1);
    assert_eq!(f.minor_version.unwrap(), 2);

    let f = p("VERSION 1 .2");
    assert_eq!(f.major_version.unwrap(), 1);
    assert_eq!(f.minor_version.unwrap(), 2);

    let f = p("VERSION 1 . 2");
    assert_eq!(f.major_version.unwrap(), 1);
    assert_eq!(f.minor_version.unwrap(), 2);

    let f = p("VERSION 1. 2");
    assert_eq!(f.major_version.unwrap(), 1);
    assert_eq!(f.minor_version.unwrap(), 2);

    let f = p("VERSION 0. 0");
    assert_eq!(f.major_version.unwrap(), 0);
    assert_eq!(f.minor_version.unwrap(), 0);

    let f = p("VERSION 65535. 65535");
    assert_eq!(f.major_version.unwrap(), 65535);
    assert_eq!(f.minor_version.unwrap(), 65535);

    err(
        "VERSION 65536. 65535",
        ParseError::new(ParseErrorKind::NumberTooLarge("65536"), 8),
    );
    err(
        "VERSION 65535. 65536",
        ParseError::new(ParseErrorKind::NumberTooLarge("65536"), 15),
    );
}

#[test]
fn sections() {
    let mut s = p("SECTIONS .rdata SECTIONS .data").sections;
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".rdata", false, false, false, false)))
    );
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".data", false, false, false, false)))
    );

    let mut s = p("SECTIONS .rdata READ").sections;
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".rdata", true, false, false, false)))
    );

    let mut s = p("SECTIONS .rdata READ\n.data READ WRITE EXECUTE").sections;
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".rdata", true, false, false, false)))
    );
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".data", true, true, true, false)))
    );

    let f = p("NAME SECTIONS .rdata READ\n.data READ WRITE EXECUTE VERSION 1.0 SECTIONS .second EXECUTE WRITE READ");
    let mut s = f.sections;
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".rdata", true, false, false, false)))
    );
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".data", true, true, true, false)))
    );
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".second", true, true, true, false)))
    );
    assert_eq!(f.major_version.unwrap(), 1);
    assert_eq!(f.minor_version.unwrap(), 0);

    let f = p("NAME SECTIONS .rdata READ\n.data READ WRITE EXECUTE SECTIONS .second EXECUTE WRITE READ VERSION 1.0");
    let mut s = f.sections;
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".rdata", true, false, false, false)))
    );
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".data", true, true, true, false)))
    );
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".second", true, true, true, false)))
    );
    assert_eq!(f.major_version.unwrap(), 1);
    assert_eq!(f.minor_version.unwrap(), 0);

    let f = p("NAME SECTIONS .rdata SHARED\n.data SHARED SECTIONS .second SHARED VERSION 1.0");
    let mut s = f.sections;
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".rdata", false, false, false, true)))
    );
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".data", false, false, false, true)))
    );
    assert_eq!(
        s.next(),
        Some(Ok(SectionRef::new(".second", false, false, false, true)))
    );
    assert_eq!(f.major_version.unwrap(), 1);
    assert_eq!(f.minor_version.unwrap(), 0);
}

#[test]
fn exports() {
    let mut e = p("EXPORTS simple").exports;
    assert_eq!(
        e.next(),
        Some(Ok(ExportRef::new(
            "simple", None, None, false, false, false
        )))
    );

    let mut e = p("EXPORTS simple VERSION 1.12 EXPORTS simple2").exports;
    assert_eq!(
        e.next(),
        Some(Ok(ExportRef::new(
            "simple", None, None, false, false, false
        )))
    );
    assert_eq!(
        e.next(),
        Some(Ok(ExportRef::new(
            "simple2", None, None, false, false, false
        )))
    );

    let mut e = p("EXPORTS simple DATA VERSION 1.12 EXPORTS simple2 PRIVATE NONAME").exports;
    assert_eq!(
        e.next(),
        Some(Ok(ExportRef::new("simple", None, None, false, false, true)))
    );
    assert_eq!(
        e.next(),
        Some(Ok(ExportRef::new("simple2", None, None, true, true, false)))
    );

    let mut e = p("EXPORTS simple\nDATA\nVERSION 1.12\nEXPORTS simple2 PRIVATE NONAME").exports;
    assert_eq!(
        e.next(),
        Some(Ok(ExportRef::new("simple", None, None, false, false, true)))
    );
    assert_eq!(
        e.next(),
        Some(Ok(ExportRef::new("simple2", None, None, true, true, false)))
    );

    let mut e = p("EXPORTS simple\nDATA\nVERSION 1.12\nEXPORTS simple2 PRIVATE NONAME").exports;
    assert_eq!(
        e.next(),
        Some(Ok(ExportRef::new("simple", None, None, false, false, true)))
    );
    assert_eq!(
        e.next(),
        Some(Ok(ExportRef::new("simple2", None, None, true, true, false)))
    );

    let mut e =
        p("EXPORTS simple=inner\nDATA\nVERSION 1.12\nEXPORTS simple2 = inner PRIVATE NONAME")
            .exports;
    assert_eq!(
        e.next(),
        Some(Ok(ExportRef::new(
            "simple",
            Some("inner"),
            None,
            false,
            false,
            true
        )))
    );
    assert_eq!(
        e.next(),
        Some(Ok(ExportRef::new(
            "simple2",
            Some("inner"),
            None,
            true,
            true,
            false
        )))
    );

    let mut e =
        p("EXPORTS simple= module.inner\nDATA\nVERSION 1.12\nEXPORTS simple2 = inner.#42 @1337 PRIVATE NONAME")
            .exports;
    assert_eq!(
        e.next(),
        Some(Ok(ExportRef::new(
            "simple",
            Some("module.inner"),
            None,
            false,
            false,
            true
        )))
    );
    assert_eq!(
        e.next(),
        Some(Ok(ExportRef::new(
            "simple2",
            Some("inner.#42"),
            Some(1337),
            true,
            true,
            false
        )))
    );
}

#[test]
fn write() {
    const FILES: &[&str] = &[
        "\
LIBRARY test
",
        "\
LIBRARY \"test with spaces\"
",
        "\
NAME test
",
        "\
NAME \"test with spaces\"
",
        "\
NAME test BASE=0x10000
",
        "\
NAME test BASE=0x10000
HEAPSIZE 0x1000,0x2000
",
        "\
NAME test BASE=0x10000
HEAPSIZE 0x1000,0x2000
STACKSIZE 0xFFFF,0xFDFD
",
        "\
NAME test BASE=0x10000
HEAPSIZE 0x1000
STACKSIZE 0xFFFF
STUB:test.x
",
        "\
NAME test BASE=0x10000
HEAPSIZE 0x1000,0x2000
STACKSIZE 0xFFFF,0xFDFD
VERSION 1.12
",
        "\
NAME test BASE=0x10000
HEAPSIZE 0x1000,0x2000
STACKSIZE 0xFFFF,0xFDFD
VERSION 1
",
        "\
NAME test BASE=0x10000
HEAPSIZE 0x1000,0x2000
STACKSIZE 0xFFFF,0xFDFD
VERSION 1
SECTIONS
    .rdata READ WRITE EXECUTE SHARED
    .idata READ EXECUTE
",
        "\
NAME test BASE=0x10000
HEAPSIZE 0x1000,0x2000
STACKSIZE 0xFFFF,0xFDFD
VERSION 1
SECTIONS
    .rdata READ WRITE EXECUTE SHARED
    .idata READ EXECUTE
EXPORTS
    name
    name=internal_name
    name=name_internal @10
    name=name_internal @10 NONAME
    name=name_internal @10 NONAME PRIVATE
    name=name_internal @10 NONAME PRIVATE DATA
    name=name_internal PRIVATE DATA
    name=module.name_internal PRIVATE DATA
    name=module.name_internal DATA
    data DATA
",
        "\
EXPORTS
    name1 @1 NONAME
    name2 @2 NONAME
    name3 @3 NONAME
    name4 @4 NONAME
    name5 @5 NONAME
",
    ];

    for &file in FILES {
        #[cfg(feature = "alloc")]
        let owned = ModuleDefinitionFile::new(file).unwrap();
        let f = ModuleDefinitionFileRef::new(file).unwrap();
        let mut buf = [0_u8; 1024];
        let buf = f.write_to_buffer(&mut buf).unwrap().unwrap();

        assert_eq!(file, buf);
        #[cfg(feature = "alloc")]
        assert_eq!(file, owned.write_to_buffer().unwrap());
    }
}
