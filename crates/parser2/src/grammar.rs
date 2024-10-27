use crate::lexer::Token;
use crate::parser::{Parser, ParserState};
use stdx::prelude::*;

mod source_file_rule;
mod top_level_statement;
pub(crate) use source_file_rule::SourceFileRule;
pub(crate) use top_level_statement::PackageStatement;

pub type BoxedRule = Box<dyn Rule>;

pub trait Rule {
    /// Returns the name of the rule
    fn name(&self) -> String;
    fn debug_format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    /// Returns whether parser state matches this rule
    fn matches(&self, s: &Parser) -> bool;
    /// Internal function
    fn parse_rule(&self, p: &mut Parser);

    /// Expect this rule. If rule does not match, panic!
    fn expect(&self, p: &mut Parser) {
        debug!("Expecting {:?}", self.name());
        assert!(self.matches(p));
        self.parse_rule(p);
    }

    /// Only parse if this rule matches
    fn opt(&self, p: &mut Parser) {
        debug!("Testing for optional {:?}", self.name());
        if self.matches(p) {
            self.parse_rule(p)
        }
    }

    /// Parse this rule. If it doesn't match a error event will be generated
    fn parse(&self, p: &mut Parser) {
        debug!(
            "Parsing {} at token {:?}",
            self.name(),
            p.tokens.next_non_ws()
        );
        self.parse_rule(p);
        debug!(
            "Finished Parsing {}, Now at {:?}",
            self.name(),
            p.tokens.next_non_ws()
        );
    }
}

impl Rule for Token {
    fn name(&self) -> String {
        format!("{:?}", self)
    }

    fn matches(&self, p: &Parser) -> bool {
        p.tokens.next_non_ws().is_some_and(|(t, _)| *t == *self)
    }

    fn parse_rule(&self, p: &mut Parser) {
        p.expect(*self);
    }

    fn debug_format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::fmt::Debug for dyn Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.debug_format(f)
    }
}
