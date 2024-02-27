use std::path::PathBuf;

use swc_core::ecma::{transforms::testing::test_fixture, visit::as_folder};
use swc_plugin_mark_expressions::{MarkExpression, Config};

#[testing::fixture("tests/fixture/**/input.js")]
fn fixture(input: PathBuf) {
    let output = input.with_file_name("output.js");
    let config: Config = Default::default();

    test_fixture(
        Default::default(),
        &|t| as_folder(MarkExpression::new(t.comments.clone(), &config)),
        &input,
        &output,
        Default::default(),
    );
}