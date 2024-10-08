use std::path::PathBuf;

use swc_core::ecma::{
    parser::{Syntax, TsSyntax},
    transforms::testing::test_fixture,
    visit::as_folder,
};
use swc_plugin_mark_expressions::{Config, MarkExpression};

#[testing::fixture("tests/fixture/**/input.*")]
fn fixture(input: PathBuf) {
    let ext = input.extension().unwrap();
    let output = input.with_file_name("output").with_extension(ext);
    let config_json = r#"
        {
            "title": "MARK_EXPRESSIONS",
            "functions": ["markFnA", "markFnB", "markFnC"],
            "methods": {
                "window": ["markWindowFnA", "markWindowFnB", "markWindowFnC"],
                "this": ["markThisFnA", "markThisFnB", "markThisFnC"]
            },
            "dynamicImports": ["shouldMark"]
        }
    "#;
    let config = serde_json::from_str::<Option<Config>>(config_json)
        .expect("Invalid config")
        .unwrap();

    test_fixture(
        Syntax::Typescript(TsSyntax {
            tsx: true,
            decorators: false,
            dts: false,
            no_early_errors: false,
            disallow_ambiguous_jsx_like: false,
        }),
        &|t| {
            as_folder(MarkExpression::new(
                t.comments.clone(),
                t.cm.clone(),
                &config,
            ))
        },
        &input,
        &output,
        Default::default(),
    );
}
