use std::str::FromStr;

use crop::Rope;
use parser2::ChangedRange;

use stdx::prelude::*;

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

#[test]
fn update_package_ident() -> Result<()> {
    common::setup();

    let file = r"
package com.example
";
    let tree = parser2::parse_no_state(file);

    let diff = parser2::parse_with_state(
        &Rope::from_str(file)?,
        tree.clone(),
        &ChangedRange::Insert {
            at_byte: 13,
            new_text: "kls.".into(),
        },
    )?;

    goldie::assert!(diff.sexp());

    Ok(())
}
