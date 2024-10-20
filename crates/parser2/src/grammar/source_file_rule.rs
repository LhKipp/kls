use crate::parser::ParserState;
use crate::Token;

use super::{Rule, TopLevelStatement};

pub(crate) struct SourceFileRule {}

impl Rule for SourceFileRule {
    fn name(&self) -> String {
        "SourceFileRule".into()
    }

    fn matches(&self, s: &crate::parser::Parser) -> bool {
        s.current_state == ParserState::SourceFile
    }

    fn parse_rule(&self, p: &mut crate::parser::Parser) {
        let mut source_file = p.start(Token::SourceFile, None);
        TopLevelStatement {}.parse_rule(p);
        source_file.finish(p);
    }
}
