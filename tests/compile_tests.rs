#[test]
fn compile_pass_berparser() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/berparser*.rs");
}

#[test]
fn compile_pass_derparser() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/derparser*.rs");
}

#[cfg(feature = "std")]
#[test]
fn compile_pass_tober() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/tober*.rs");
}

#[cfg(feature = "std")]
#[test]
fn compile_pass_toder() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/toder*.rs");
}

#[test]
fn compile_pass_choice() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/choice*.rs");
}

#[test]
fn compile_pass_enumerated() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/enumerated*.rs");
}

#[test]
fn compile_pass_sequence() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/sequence*.rs");
}

#[test]
fn compile_pass_alias() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/alias*.rs");
}

#[test]
fn compile_pass_set() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/set*.rs");
}

#[test]
fn compile_pass_misc() {
    let t = trybuild::TestCases::new();
    t.pass("tests/run-pass/custom*.rs");
    t.pass("tests/run-pass/der_*.rs");
}

#[test]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile-fail/*.rs");
}
