use crate::parser::ParserState;
use crate::Token;

use super::{PackageStatement, Rule};

#[derive(Debug)]
pub(crate) struct SourceFileRule {}

impl Rule for SourceFileRule {
    fn name(&self) -> String {
        "SourceFileRule".into()
    }

    fn debug_format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }

    fn matches(&self, s: &crate::parser::Parser) -> bool {
        true
    }

    fn parse_rule(&self, p: &mut crate::parser::Parser) {
        let mut source_file = p.start(Token::SourceFile, None);
        PackageStatement::new(None).parse_rule(p);
        source_file.finish(p);
    }
}
