mod common;

#[test]
fn new_package() {
    common::setup();

    let file = r"
package com.example
";
    let tree = parser2::parse_no_state(file);

    goldie::assert!(tree.sexp());
}
