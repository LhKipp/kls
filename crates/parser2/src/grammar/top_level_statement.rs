use crate::lexer::Token;

use super::Rule;
use stdx::prelude::*;

#[derive(new, Debug)]
pub(crate) struct PackageStatement {
    pub start_at: Option<Token>,
}

impl Rule for PackageStatement {
    fn name(&self) -> String {
        "TopLevelStatement".into()
    }
    fn debug_format(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }

    fn matches(&self, s: &crate::parser::Parser) -> bool {
        s.tokens
            .next_non_ws()
            .is_some_and(|(t, _)| *t == Token::PackageKeyword)
    }

    fn parse_rule(&self, p: &mut crate::parser::Parser) {
        let mut package_decl = p.start(Token::PackageDecl, None);
        let jump_to = self.jump_to();

        if jump_to <= 0 {
            p.eat(Token::Ws);
            p.expect(Token::PackageKeyword);
            p.eat(Token::Ws);
        }
        if jump_to <= 1 {
            p.expect(Token::SimpleIdent);
        }
        if jump_to <= 2 && !p.eat(Token::Period) {
            package_decl.finish(p);
            return;
        }

        loop {
            p.expect(Token::SimpleIdent);
            if !p.eat(Token::Period) {
                package_decl.finish(p);
                return;
            }
        }
    }
}

impl PackageStatement {
    fn jump_to(&self) -> i32 {
        let Some(t) = self.start_at else {
            return 0;
        };
        if t == Token::Period {
            return 2;
        }
        if t == Token::SimpleIdent {
            return 1;
        }
        trace!("TopLevelStatement: could not calculate jump_to for {}", t);
        0
    }
}
