use std::path::PathBuf;

use swc_core::ecma::{parser::{Syntax, TsConfig}, transforms::testing::test_fixture, visit::as_folder};
use swc_plugin_mark_expressions::{MarkExpression, Config};

#[testing::fixture("tests/fixture/**/input.*")]
fn fixture(input: PathBuf) {
    let ext = input.extension().unwrap();
    let output = input.with_file_name("output").with_extension(ext);
    let config = Config {
        title: "markExpression".into(),
        functions: vec!("markedFunction".into()),
        objects: vec!("window".into(), "this".into()),
    };

    test_fixture(
        Syntax::Typescript(TsConfig {
            tsx: true,
            decorators: true,
            dts: false,
            no_early_errors: true,
            disallow_ambiguous_jsx_like: false,
        }),
        &|t| as_folder(MarkExpression::new(t.comments.clone(), &config)),
        &input,
        &output,
        Default::default(),
    );
}